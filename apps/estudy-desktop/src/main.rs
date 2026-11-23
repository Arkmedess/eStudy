//! Binário desktop principal do **eStudy** (Composition Root).

use chrono::{SecondsFormat, Utc};
use estudy_domain::{EntityId, SystemClock};
use estudy_storage::{
    AppPaths, SqliteProjectRepository, StorageError, apply_migrations, open_connection,
};
use estudy_ui::{AppMessage, EstudyApp, HomeDashboardData, HomeLoadState, ProjectsState};
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

struct DesktopShell {
    app: EstudyApp,
    conn: Option<Connection>,
}

impl DesktopShell {
    fn boot() -> Self {
        let clock = Arc::new(SystemClock);
        let now = Utc::now();
        let mut projects_state = ProjectsState::sample(now.date_naive(), now);

        match open_and_migrate() {
            Ok((conn, display_name)) => {
                if let Ok(loaded) = load_or_seed_projects(&conn, &projects_state) {
                    projects_state = loaded;
                }
                let home_state = HomeLoadState::Ready(Box::new(
                    HomeDashboardData::sample_workspace(display_name),
                ));
                Self {
                    app: EstudyApp::with_projects(clock, home_state, projects_state),
                    conn: Some(conn),
                }
            }
            Err(err) => Self {
                app: EstudyApp::with_projects(
                    clock,
                    HomeLoadState::Error(err.to_string()),
                    projects_state,
                ),
                conn: None,
            },
        }
    }

    fn update(&mut self, message: AppMessage) {
        let is_project_mutation = matches!(message, AppMessage::Projects(_));
        self.app.update(message);

        if is_project_mutation && let Some(conn) = &mut self.conn {
            for project in &self.app.projects_state.projects {
                let _ = SqliteProjectRepository::save_project(conn, project, None);
            }
            for task in &self.app.projects_state.tasks {
                let _ = SqliteProjectRepository::save_task(conn, task, None);
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<AppMessage> {
        self.app.subscription()
    }

    fn view(&self) -> iced::Element<'_, AppMessage> {
        self.app.view()
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

fn open_and_migrate() -> Result<(Connection, String), StorageError> {
    let cli_dir = parse_data_dir_arg();
    let paths = AppPaths::resolve(cli_dir.as_deref())?;
    paths.ensure_dirs()?;

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

fn main() -> iced::Result {
    iced::application(DesktopShell::boot, DesktopShell::update, DesktopShell::view)
        .title("eStudy")
        .subscription(DesktopShell::subscription)
        .window_size((1280.0, 800.0))
        .centered()
        .run()
}
