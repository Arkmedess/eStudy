//! Binário desktop principal do **eStudy** (Composition Root).

use chrono::Utc;
use estudy_domain::SystemClock;
use estudy_storage::{AppPaths, apply_migrations, open_connection};
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

    HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Estudante")))
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
