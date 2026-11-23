//! Binário desktop principal do **eStudy** (Composition Root).

use chrono::{SecondsFormat, Utc};
use estudy_domain::{EntityId, SystemClock};
use estudy_storage::{
    AppPaths, SearchEntityKind, SqliteCalendarRepository, SqliteProjectRepository,
    SqliteSearchRepository, StorageError, apply_migrations, open_connection,
};
use estudy_ui::{
    AppMessage, AppRoute, CalendarState, EstudyApp, HomeDashboardData, HomeLoadState,
    PaletteAction, PaletteCommand, PaletteMessage, ProjectsState,
};
use estudy_ui_kit::Icon;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct DesktopShell {
    app: EstudyApp,
    conn: Option<Connection>,
    last_fts_query: Option<String>,
    _log_guard: Option<WorkerGuard>,
}

impl Drop for DesktopShell {
    fn drop(&mut self) {
        self.app.projects_state.flush_deferred();
        self.persist_dirty_changes();
    }
}

impl DesktopShell {
    fn boot() -> Self {
        let boot_started = Instant::now();
        let clock = Arc::new(SystemClock);
        let now = Utc::now();
        let mut projects_state = ProjectsState::sample(now.date_naive(), now);
        let mut calendar_state =
            CalendarState::sample(now.date_naive(), now, &projects_state.projects);

        let cli_dir = parse_data_dir_arg();
        let paths_res = AppPaths::resolve(cli_dir.as_deref()).and_then(|paths| {
            paths.ensure_dirs()?;
            Ok(paths)
        });

        let log_guard = paths_res
            .as_ref()
            .ok()
            .and_then(|paths| init_structured_logging(&paths.logs_dir()));

        match paths_res.and_then(|paths| open_and_migrate_with_paths(&paths)) {
            Ok((conn, display_name)) => {
                if let Ok(loaded) = load_or_seed_projects(&conn, &projects_state) {
                    projects_state = loaded;
                }
                if let Ok(loaded_cal) = load_or_seed_calendar(&conn, &calendar_state) {
                    calendar_state = loaded_cal;
                }
                let home_state = HomeLoadState::Ready(Box::new(
                    HomeDashboardData::sample_workspace(display_name),
                ));
                let boot_ms = u64::try_from(boot_started.elapsed().as_millis()).unwrap_or(u64::MAX);
                tracing::info!(
                    target: "estudy.desktop",
                    service_version = env!("CARGO_PKG_VERSION"),
                    boot_ms,
                    renderer = if cfg!(feature = "gpu") { "wgpu" } else { "tiny-skia" },
                    "aplicativo desktop inicializado"
                );
                Self {
                    app: EstudyApp::with_state(clock, home_state, projects_state, calendar_state),
                    conn: Some(conn),
                    last_fts_query: None,
                    _log_guard: log_guard,
                }
            }
            Err(err) => {
                tracing::error!(
                    target: "estudy.desktop",
                    error = %err,
                    "falha ao inicializar banco local"
                );
                Self {
                    app: EstudyApp::with_state(
                        clock,
                        HomeLoadState::Error(err.to_string()),
                        projects_state,
                        calendar_state,
                    ),
                    conn: None,
                    last_fts_query: None,
                    _log_guard: log_guard,
                }
            }
        }
    }

    fn update(&mut self, message: AppMessage) {
        let palette_query = match &message {
            AppMessage::Palette(PaletteMessage::QueryChanged(query)) => Some(query.clone()),
            AppMessage::Palette(PaletteMessage::Close | PaletteMessage::Toggle) => {
                self.last_fts_query = None;
                None
            }
            _ => None,
        };

        self.app.update(message);

        if let Some(query) = palette_query {
            self.run_fts_palette_search(&query);
        }

        self.persist_dirty_changes();
    }

    fn persist_dirty_changes(&mut self) {
        let Some(conn) = &mut self.conn else {
            return;
        };

        let (dirty_projects, dirty_tasks) = self.app.projects_state.take_dirty();
        let dirty_events = self.app.calendar_state.take_dirty_events();
        if dirty_projects.is_empty() && dirty_tasks.is_empty() && dirty_events.is_empty() {
            return;
        }

        let started = Instant::now();
        let mut persisted = 0_usize;

        for proj_id in &dirty_projects {
            if let Some(project) = self
                .app
                .projects_state
                .projects
                .iter()
                .find(|item| item.id == *proj_id)
                && SqliteProjectRepository::save_project(conn, project, None).is_ok()
            {
                persisted += 1;
            }
        }

        for task_id in &dirty_tasks {
            if let Some(task) = self
                .app
                .projects_state
                .tasks
                .iter()
                .find(|item| item.id == *task_id)
                && SqliteProjectRepository::save_task(conn, task, None).is_ok()
            {
                persisted += 1;
            }
        }

        for event_id in &dirty_events {
            if let Some(event) = self
                .app
                .calendar_state
                .events
                .iter()
                .find(|item| item.id == *event_id)
                && SqliteCalendarRepository::save_event(conn, event, None).is_ok()
            {
                persisted += 1;
            }
        }

        let elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
        tracing::debug!(
            target: "estudy.desktop",
            dirty_projects = dirty_projects.len(),
            dirty_tasks = dirty_tasks.len(),
            dirty_events = dirty_events.len(),
            persisted,
            elapsed_us,
            "persistência seletiva concluída"
        );
    }

    fn run_fts_palette_search(&mut self, query: &str) {
        let trimmed = query.trim();
        if trimmed.starts_with('>') || trimmed.chars().count() < 2 {
            return;
        }
        if self.last_fts_query.as_deref() == Some(trimmed) {
            return;
        }
        let Some(conn) = &self.conn else {
            return;
        };

        self.last_fts_query = Some(trimmed.to_string());
        if let Ok(hits) = SqliteSearchRepository::search_all(conn, trimmed, 15) {
            let fts_entries: Vec<PaletteCommand> = hits
                .into_iter()
                .filter(|hit| {
                    matches!(
                        hit.kind,
                        SearchEntityKind::Note | SearchEntityKind::Subject | SearchEntityKind::Book
                    )
                })
                .map(|hit| {
                    let (icon, route) = match hit.kind {
                        SearchEntityKind::Note => (Icon::Code2, AppRoute::Home),
                        SearchEntityKind::Subject => (Icon::GraduationCap, AppRoute::Studies),
                        SearchEntityKind::Book => (Icon::BookOpen, AppRoute::Bookshelf),
                        SearchEntityKind::Project => (Icon::FolderKanban, AppRoute::Projects),
                        SearchEntityKind::Task => (Icon::CheckSquare, AppRoute::Projects),
                        SearchEntityKind::Event => (Icon::Calendar, AppRoute::Calendar),
                    };
                    PaletteCommand {
                        id: format!("fts:{}", hit.id),
                        label: hit.title,
                        hint: hit.subtitle,
                        shortcut: None,
                        icon,
                        action: PaletteAction::Navigate(route),
                    }
                })
                .collect();

            self.app
                .palette
                .search_entries
                .retain(|entry| !entry.id.starts_with("fts:"));
            self.app.palette.search_entries.extend(fts_entries);
        }
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        self.app.subscription()
    }

    fn view(&self) -> iced::Element<'_, AppMessage> {
        self.app.view()
    }
}

fn init_structured_logging(logs_dir: &Path) -> Option<WorkerGuard> {
    let appender = tracing_appender::rolling::daily(logs_dir, "estudy.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_target(true)
        .with_current_span(false);

    if tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .try_init()
        .is_ok()
    {
        Some(guard)
    } else {
        None
    }
}

fn parse_data_dir_arg() -> Option<PathBuf> {
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--data-dir" {
            return args.next().map(PathBuf::from);
        }
    }
    None
}

fn open_and_migrate_with_paths(paths: &AppPaths) -> Result<(Connection, String), StorageError> {
    let mut conn = open_connection(&paths.db_path())?;
    apply_migrations(&mut conn, Some(&paths.backups_dir()), Utc::now())?;
    let name = ensure_default_profile(&conn)?;
    Ok((conn, name))
}

fn ensure_default_profile(conn: &Connection) -> Result<String, StorageError> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT display_name FROM local_profiles WHERE deleted_at IS NULL ORDER BY created_at ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    if let Some(name) = existing {
        return Ok(name);
    }

    let id = EntityId::new_v7().to_string();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let default_name = "Estudante".to_string();
    conn.execute(
        "INSERT INTO local_profiles (id, display_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, default_name, now, now],
    )?;
    Ok(default_name)
}

fn load_or_seed_projects(
    conn: &Connection,
    fallback: &ProjectsState,
) -> Result<ProjectsState, StorageError> {
    let projects = SqliteProjectRepository::list_projects(conn)?;
    let tasks = SqliteProjectRepository::list_tasks(conn)?;

    if projects.is_empty() && tasks.is_empty() {
        return Ok(fallback.clone());
    }

    let mut state = fallback.clone();
    state.selected_project_id = projects.first().map(|proj| proj.id.clone());
    state.projects = projects;
    state.tasks = tasks;
    Ok(state)
}

fn load_or_seed_calendar(
    conn: &Connection,
    fallback: &CalendarState,
) -> Result<CalendarState, StorageError> {
    let events = SqliteCalendarRepository::list_events(conn)?;
    if events.is_empty() {
        return Ok(fallback.clone());
    }

    let mut state = fallback.clone();
    state.events = events;
    Ok(state)
}

fn main() -> iced::Result {
    iced::application(DesktopShell::boot, DesktopShell::update, DesktopShell::view)
        .title("eStudy")
        .subscription(DesktopShell::subscription)
        .window_size((1280.0, 800.0))
        .centered()
        .run()
}
