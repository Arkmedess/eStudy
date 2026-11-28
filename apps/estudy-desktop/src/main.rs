//! Binário desktop principal do **eStudy** (Composition Root).

mod screenshot_cli;

use chrono::{SecondsFormat, Utc};
use estudy_audio::AudioController;
use estudy_domain::{CalendarViewMode, EntityId, PomodoroPreset, SystemClock, TaskTurn};
use estudy_storage::{
    AppPaths, SearchEntityKind, SqliteCalendarRepository, SqliteFocusRepository,
    SqliteHabitsRepository, SqliteProjectRepository, SqliteSearchRepository,
    SqliteSecondBrainRepository, StorageError, apply_migrations, open_connection,
};
use estudy_ui::features::focus::FocusMessage;
use estudy_ui::{
    AppMessage, AppRoute, CalendarMessage, CalendarState, EstudyApp, HomeDashboardData,
    HomeLoadState, PaletteAction, PaletteCommand, PaletteMessage, ProjectsMessage, ProjectsState,
    SecondBrainMessage, TasksViewMode,
};
use estudy_ui_kit::{FONT_SANS, Icon, embedded_fonts};
use rusqlite::Connection;
use screenshot_cli::{ScreenshotOptions, parse_screenshot_args, save_screenshot_png};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const LOCAL_WALLET_SECRET: &[u8] = b"estudy-local-wallet-v1";

#[derive(Debug, Clone)]
enum ShellMessage {
    App(AppMessage),
    TriggerScreenshot,
    ScreenshotCaptured(iced::window::Screenshot),
}

struct DesktopShell {
    app: EstudyApp,
    conn: Option<Connection>,
    profile_id: Option<EntityId>,
    audio: Option<AudioController>,
    last_focus_tick: Option<Instant>,
    last_fts_query: Option<String>,
    _log_guard: Option<WorkerGuard>,
    screenshot_opts: Option<ScreenshotOptions>,
    screenshot_requested: bool,
}

impl Drop for DesktopShell {
    fn drop(&mut self) {
        self.app.projects_state.flush_deferred();
        self.persist_dirty_changes();
    }
}

fn apply_screenshot_overrides(app: &mut EstudyApp, opts: &ScreenshotOptions) {
    app.route = opts.route;
    app.loadout.theme_preset = opts.preset;
    app.loadout.theme_mode = opts.mode;
    app.nav_sidebar_open = opts.sidebar_open;
    app.window_size = (opts.width, opts.height);

    let Some(subview) = opts.subview.as_deref() else {
        return;
    };

    let now = Utc::now();
    let today = now.date_naive();
    match subview {
        "week" => {
            app.projects_state
                .update(ProjectsMessage::SetTasksViewMode(TasksViewMode::Week), now);
        }
        "calendar" => match opts.route {
            AppRoute::Projects => app.projects_state.update(
                ProjectsMessage::SetTasksViewMode(TasksViewMode::Calendar),
                now,
            ),
            AppRoute::Calendar => app
                .calendar_state
                .update(CalendarMessage::SetViewMode(CalendarViewMode::Month), now),
            _ => {}
        },
        "kanban" => {
            app.projects_state.update(
                ProjectsMessage::SetTasksViewMode(TasksViewMode::Kanban),
                now,
            );
        }
        "list" => {
            app.calendar_state
                .update(CalendarMessage::SetViewMode(CalendarViewMode::Day), now);
        }
        "task-modal" => {
            let msg = app.projects_state.tasks.first().map_or(
                ProjectsMessage::OpenCreateTaskModal {
                    date: Some(today),
                    turn: TaskTurn::Morning,
                },
                |task| ProjectsMessage::OpenEditTaskModal(task.id.clone()),
            );
            app.projects_state.update(msg, now);
        }
        "day-peek" => {
            app.calendar_state
                .update(CalendarMessage::OpenDayPeek(today), now);
        }
        "note-modal" => {
            let msg = app
                .second_brain_state
                .notes
                .first()
                .map_or(SecondBrainMessage::CreateNote, |note| {
                    SecondBrainMessage::SelectNote(note.id.clone())
                });
            app.second_brain_state.update(msg, now);
        }
        "today-drawer" => {
            app.calendar_state.today_drawer_open = true;
        }
        "palette" => {
            app.update(AppMessage::Palette(PaletteMessage::Toggle));
        }
        "focus-running" => {
            app.update(AppMessage::Focus(FocusMessage::SelectPreset(
                PomodoroPreset::Classic25_5,
            )));
            app.update(AppMessage::Focus(FocusMessage::Start));
        }
        _ => {}
    }
}

impl DesktopShell {
    fn boot_sample_screenshot(
        clock: Arc<dyn estudy_domain::ClockPort>,
        projects_state: ProjectsState,
        calendar_state: CalendarState,
        opts: ScreenshotOptions,
    ) -> Self {
        let home_state = HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace(
            "Estudante".to_string(),
        )));
        let mut app = EstudyApp::with_state(clock, home_state, projects_state, calendar_state);
        apply_screenshot_overrides(&mut app, &opts);
        Self {
            app,
            conn: None,
            profile_id: None,
            audio: None,
            last_focus_tick: None,
            last_fts_query: None,
            _log_guard: None,
            screenshot_opts: Some(opts),
            screenshot_requested: false,
        }
    }

    fn boot(screenshot_opts: Option<ScreenshotOptions>) -> Self {
        let boot_started = Instant::now();
        let clock = Arc::new(SystemClock);
        let now = Utc::now();
        let projects_state = ProjectsState::sample(now.date_naive(), now);
        let calendar_state = CalendarState::sample(now.date_naive(), now, &projects_state.projects);

        let cli_dir = parse_data_dir_arg();
        if let Some(opts) = screenshot_opts.as_ref().filter(|_| cli_dir.is_none()) {
            return Self::boot_sample_screenshot(
                clock,
                projects_state,
                calendar_state,
                opts.clone(),
            );
        }

        let paths_res = AppPaths::resolve(cli_dir.as_deref()).and_then(|paths| {
            paths.ensure_dirs()?;
            Ok(paths)
        });

        let log_guard = paths_res
            .as_ref()
            .ok()
            .and_then(|paths| init_structured_logging(&paths.logs_dir()));

        match paths_res.and_then(|paths| open_and_migrate_with_paths(&paths)) {
            Ok((conn, profile_id, display_name)) => Self::boot_loaded_shell(
                clock,
                projects_state,
                calendar_state,
                conn,
                profile_id,
                display_name,
                boot_started,
                log_guard,
                screenshot_opts,
            ),
            Err(err) => {
                tracing::error!(
                    target: "estudy.desktop",
                    error = %err,
                    "falha ao inicializar banco local"
                );
                let mut app = EstudyApp::with_state(
                    clock,
                    HomeLoadState::Error(err.to_string()),
                    projects_state,
                    calendar_state,
                );
                if let Some(opts) = &screenshot_opts {
                    apply_screenshot_overrides(&mut app, opts);
                }
                Self {
                    app,
                    conn: None,
                    profile_id: None,
                    audio: None,
                    last_focus_tick: None,
                    last_fts_query: None,
                    _log_guard: log_guard,
                    screenshot_opts,
                    screenshot_requested: false,
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn boot_loaded_shell(
        clock: Arc<dyn estudy_domain::ClockPort>,
        mut projects_state: ProjectsState,
        mut calendar_state: CalendarState,
        conn: Connection,
        profile_id: EntityId,
        display_name: String,
        boot_started: Instant,
        log_guard: Option<WorkerGuard>,
        screenshot_opts: Option<ScreenshotOptions>,
    ) -> Self {
        let today = clock.now_utc().date_naive();
        if let Ok(loaded) = load_or_seed_projects(&conn, &projects_state) {
            projects_state = loaded;
        }
        if let Ok(loaded_cal) = load_or_seed_calendar(&conn, &calendar_state) {
            calendar_state = loaded_cal;
        }
        let today_focus = SqliteFocusRepository::today_focus_minutes(&conn, today).unwrap_or(0);
        let home_state =
            HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace(display_name)));
        let mut app = EstudyApp::with_state(clock, home_state, projects_state, calendar_state);
        if let Ok(habits) = SqliteHabitsRepository::list_habits(&conn)
            && !habits.is_empty()
        {
            app.habits_state.habits = habits;
        }
        if let Ok(notes) = SqliteSecondBrainRepository::list_notes(&conn)
            && !notes.is_empty()
        {
            app.second_brain_state.selected_note_id = notes.first().map(|n| n.id.clone());
            app.second_brain_state.notes = notes;
        }
        app.focus_state.today_focused_minutes = today_focus;
        let boot_ms = u64::try_from(boot_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            target: "estudy.desktop",
            service_version = env!("CARGO_PKG_VERSION"),
            boot_ms,
            renderer = if cfg!(feature = "gpu") { "wgpu" } else { "tiny-skia" },
            "aplicativo desktop inicializado"
        );
        if let Some(opts) = &screenshot_opts {
            apply_screenshot_overrides(&mut app, opts);
        }
        Self {
            app,
            conn: Some(conn),
            profile_id: Some(profile_id),
            audio: None,
            last_focus_tick: None,
            last_fts_query: None,
            _log_guard: log_guard,
            screenshot_opts,
            screenshot_requested: false,
        }
    }

    fn update(&mut self, message: ShellMessage) -> iced::Task<ShellMessage> {
        let mut message = match message {
            ShellMessage::App(app_msg) => app_msg,
            ShellMessage::TriggerScreenshot => {
                if self.screenshot_requested {
                    return iced::Task::none();
                }
                self.screenshot_requested = true;
                return iced::window::oldest().and_then(|id| {
                    iced::window::screenshot(id).map(ShellMessage::ScreenshotCaptured)
                });
            }
            ShellMessage::ScreenshotCaptured(shot) => {
                if let Some(opts) = &self.screenshot_opts
                    && let Err(err) = save_screenshot_png(&shot, &opts.output_path)
                {
                    eprintln!("erro ao salvar screenshot: {err}");
                }
                return iced::exit();
            }
        };

        if let AppMessage::Focus(FocusMessage::TickSeconds(fallback_secs)) = &message {
            let now_mono = Instant::now();
            let elapsed_secs = self
                .last_focus_tick
                .map_or(*fallback_secs, |prev| {
                    u32::try_from(now_mono.saturating_duration_since(prev).as_secs()).unwrap_or(1)
                })
                .max(1);
            self.last_focus_tick = Some(now_mono);
            message = AppMessage::Focus(FocusMessage::TickSeconds(elapsed_secs));
        } else if !self.app.is_focus_ticking() {
            self.last_focus_tick = None;
        }

        let palette_query = match &message {
            AppMessage::Palette(PaletteMessage::QueryChanged(query)) => Some(query.clone()),
            AppMessage::Palette(PaletteMessage::Close | PaletteMessage::Toggle) => {
                self.last_fts_query = None;
                None
            }
            _ => None,
        };

        self.app.update(message);
        self.sync_audio_engine();

        if self.app.is_focus_ticking() && self.last_focus_tick.is_none() {
            self.last_focus_tick = Some(Instant::now());
        } else if !self.app.is_focus_ticking() {
            self.last_focus_tick = None;
        }

        if let Some(query) = palette_query {
            self.run_fts_palette_search(&query);
        }

        self.persist_dirty_changes();
        iced::Task::none()
    }

    fn sync_audio_engine(&mut self) {
        let focus = &self.app.focus_state;
        if focus.audio_playing && self.audio.is_none() {
            self.audio = Some(AudioController::try_start());
        }
        if let Some(audio) = &self.audio {
            audio.apply(
                focus.dsp_track,
                focus.volume_percent,
                focus.muted,
                focus.audio_playing,
            );
        }
    }

    fn persist_habits_and_notes(
        conn: &mut Connection,
        habits: &[estudy_domain::Habit],
        notes: &[estudy_domain::KnowledgeNote],
        dirty_habits: &[EntityId],
        deleted_habits: &[EntityId],
        dirty_notes: &[EntityId],
        deleted_notes: &[EntityId],
    ) -> usize {
        let now_utc = Utc::now();
        let mut persisted = 0_usize;
        for habit_id in dirty_habits {
            if let Some(habit) = habits.iter().find(|item| item.id == *habit_id)
                && SqliteHabitsRepository::save_habit(conn, habit, now_utc).is_ok()
            {
                persisted += 1;
            }
        }
        for habit_id in deleted_habits {
            if SqliteHabitsRepository::delete_habit(conn, habit_id, now_utc).is_ok() {
                persisted += 1;
            }
        }
        for note_id in dirty_notes {
            if let Some(note) = notes.iter().find(|item| item.id == *note_id)
                && SqliteSecondBrainRepository::save_note(conn, note).is_ok()
            {
                persisted += 1;
            }
        }
        for note_id in deleted_notes {
            if SqliteSecondBrainRepository::delete_note(conn, note_id, now_utc).is_ok() {
                persisted += 1;
            }
        }
        persisted
    }

    fn persist_dirty_changes(&mut self) {
        let Some(conn) = &mut self.conn else {
            return;
        };

        let (dirty_projects, dirty_tasks) = self.app.projects_state.take_dirty();
        let deleted_tasks = self.app.projects_state.take_deleted_tasks();
        let dirty_events = self.app.calendar_state.take_dirty_events();
        let deleted_events = self.app.calendar_state.take_deleted_events();
        let (dirty_habits, deleted_habits) = self.app.habits_state.take_dirty();
        let (dirty_notes, deleted_notes) = self.app.second_brain_state.take_dirty();
        let pending_focus = self.app.focus_state.take_pending_sessions();
        if dirty_projects.is_empty()
            && dirty_tasks.is_empty()
            && deleted_tasks.is_empty()
            && dirty_events.is_empty()
            && deleted_events.is_empty()
            && dirty_habits.is_empty()
            && deleted_habits.is_empty()
            && dirty_notes.is_empty()
            && deleted_notes.is_empty()
            && pending_focus.is_empty()
        {
            return;
        }

        let started = Instant::now();
        let mut persisted = Self::persist_habits_and_notes(
            conn,
            &self.app.habits_state.habits,
            &self.app.second_brain_state.notes,
            &dirty_habits,
            &deleted_habits,
            &dirty_notes,
            &deleted_notes,
        );

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

        let now_del = Utc::now();
        for task_id in &deleted_tasks {
            if SqliteProjectRepository::soft_delete_task(conn, task_id, now_del).is_ok() {
                persisted += 1;
            }
        }

        persisted += Self::persist_calendar_changes(
            conn,
            &self.app.calendar_state.events,
            &dirty_events,
            &deleted_events,
            now_del,
        );

        if let Some(profile_id) = &self.profile_id {
            for draft in pending_focus {
                if SqliteFocusRepository::record_session(
                    conn,
                    LOCAL_WALLET_SECRET,
                    profile_id,
                    draft,
                    None,
                )
                .is_ok()
                {
                    persisted += 1;
                }
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

    fn persist_calendar_changes(
        conn: &mut Connection,
        events: &[estudy_domain::CalendarEvent],
        dirty_events: &[EntityId],
        deleted_events: &[EntityId],
        deleted_at: chrono::DateTime<Utc>,
    ) -> usize {
        let mut count = 0usize;
        for event_id in dirty_events {
            if let Some(event) = events.iter().find(|item| item.id == *event_id)
                && SqliteCalendarRepository::save_event(conn, event, None).is_ok()
            {
                count += 1;
            }
        }
        for event_id in deleted_events {
            if SqliteCalendarRepository::soft_delete_event(conn, event_id, deleted_at).is_ok() {
                count += 1;
            }
        }
        count
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

    fn subscription(&self) -> iced::Subscription<ShellMessage> {
        let base = self.app.subscription().map(ShellMessage::App);
        let mut subs = vec![base];
        if self.app.is_focus_ticking() {
            subs.push(
                iced::time::every(Duration::from_secs(1))
                    .map(|_| ShellMessage::App(AppMessage::Focus(FocusMessage::TickSeconds(1)))),
            );
        }
        if self.screenshot_opts.is_some() && !self.screenshot_requested {
            subs.push(
                iced::time::every(Duration::from_millis(250))
                    .map(|_| ShellMessage::TriggerScreenshot),
            );
        }
        iced::Subscription::batch(subs)
    }

    fn view(&self) -> iced::Element<'_, ShellMessage> {
        self.app.view().map(ShellMessage::App)
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

fn open_and_migrate_with_paths(
    paths: &AppPaths,
) -> Result<(Connection, EntityId, String), StorageError> {
    let mut conn = open_connection(&paths.db_path())?;
    apply_migrations(&mut conn, Some(&paths.backups_dir()), Utc::now())?;
    let (profile_id, name) = ensure_default_profile(&conn)?;
    Ok((conn, profile_id, name))
}

fn ensure_default_profile(conn: &Connection) -> Result<(EntityId, String), StorageError> {
    let existing: Option<(String, String)> = conn
        .query_row(
            "SELECT id, display_name FROM local_profiles WHERE deleted_at IS NULL ORDER BY created_at ASC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    if let Some((raw_id, name)) = existing
        && let Ok(parsed_id) = EntityId::parse(&raw_id)
    {
        return Ok((parsed_id, name));
    }

    let entity_id = EntityId::new_v7();
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let default_name = "Estudante".to_string();
    conn.execute(
        "INSERT INTO local_profiles (id, display_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![entity_id.to_string(), default_name, now, now],
    )?;
    Ok((entity_id, default_name))
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
    let screenshot_opts = parse_screenshot_args(std::env::args_os().skip(1));
    let boot_opts = screenshot_opts.clone();

    let mut app = iced::application(
        move || DesktopShell::boot(boot_opts.clone()),
        DesktopShell::update,
        DesktopShell::view,
    )
    .title("eStudy")
    .default_font(FONT_SANS)
    .subscription(DesktopShell::subscription);

    if let Some(opts) = &screenshot_opts {
        let size = iced::Size::new(opts.width, opts.height);
        app = app.window(iced::window::Settings {
            size,
            min_size: Some(size),
            max_size: Some(size),
            resizable: false,
            position: iced::window::Position::Centered,
            ..iced::window::Settings::default()
        });
    } else {
        app = app.window_size((1360.0, 840.0)).centered();
    }

    for font_bytes in embedded_fonts() {
        app = app.font(font_bytes);
    }

    app.run()
}
