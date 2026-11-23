//! Binário desktop principal do **eStudy** (Composition Root).

use chrono::{SecondsFormat, Utc};
use estudy_domain::{EntityId, SystemClock};
use estudy_storage::{AppPaths, StorageError, apply_migrations, open_connection};
use estudy_ui::{EstudyApp, HomeDashboardData, HomeLoadState};
use std::path::PathBuf;
use std::sync::Arc;

fn parse_data_dir_arg() -> Option<PathBuf> {
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--data-dir" {
            return args.next().map(PathBuf::from);
        }
    }
    None
}

fn ensure_default_profile(conn: &rusqlite::Connection) -> Result<String, StorageError> {
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

fn bootstrap_home_state() -> HomeLoadState {
    let cli_dir = parse_data_dir_arg();
    let paths = match AppPaths::resolve(cli_dir.as_deref()) {
        Ok(paths) => paths,
        Err(err) => return HomeLoadState::Error(err.to_string()),
    };

    if let Err(err) = paths.ensure_dirs() {
        return HomeLoadState::Error(err.to_string());
    }

    let mut conn = match open_connection(&paths.db_path()) {
        Ok(conn) => conn,
        Err(err) => return HomeLoadState::Error(err.to_string()),
    };

    if let Err(err) = apply_migrations(&mut conn, Some(&paths.backups_dir()), Utc::now()) {
        return HomeLoadState::Error(err.to_string());
    }

    match ensure_default_profile(&conn) {
        Ok(name) => HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace(name))),
        Err(err) => HomeLoadState::Error(err.to_string()),
    }
}

fn boot_app() -> EstudyApp {
    let clock = Arc::new(SystemClock);
    let home_state = bootstrap_home_state();
    EstudyApp::new(clock, home_state)
}

fn main() -> iced::Result {
    iced::application(boot_app, EstudyApp::update, EstudyApp::view)
        .title("eStudy")
        .subscription(EstudyApp::subscription)
        .window_size((1280.0, 800.0))
        .centered()
        .run()
}
