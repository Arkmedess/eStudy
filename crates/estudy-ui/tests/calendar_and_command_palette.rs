//! Testes comportamentais MVU para Calendário Unificado, Drawer "Hoje" e `Ctrl+K` Completo.

#![allow(clippy::expect_used)]

use chrono::DateTime;
use estudy_domain::{CalendarViewMode, FixedClock};
use estudy_ui::{
    AppMessage, AppRoute, CalendarMessage, EstudyApp, HomeDashboardData, HomeLoadState,
    HomeMessage, PaletteMessage,
};
use std::sync::Arc;

fn make_app() -> EstudyApp {
    let now = DateTime::from_timestamp(1_760_000_000, 0).expect("timestamp válido");
    let clock = Arc::new(FixedClock::new(now));
    EstudyApp::new(
        clock,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    )
}

#[test]
fn should_open_today_timeline_and_week_calendar_from_home() {
    let mut app = make_app();
    assert!(!app.calendar_state.today_drawer_open);

    app.update(AppMessage::Home(HomeMessage::OpenTodayTimeline));
    assert!(app.calendar_state.today_drawer_open);

    app.update(AppMessage::CloseOverlays);
    assert!(!app.calendar_state.today_drawer_open);

    app.update(AppMessage::Home(HomeMessage::OpenCalendarWeek));
    assert_eq!(app.route, AppRoute::Calendar);
    assert_eq!(app.calendar_state.view_mode, CalendarViewMode::Week);

    app.update(AppMessage::ToggleContextSidebar);
    assert!(!app.calendar_state.sidebar_visible);
}

#[test]
fn should_filter_calendar_by_project_and_create_events() {
    let mut app = make_app();
    app.update(AppMessage::Navigate(AppRoute::Calendar));
    app.update(AppMessage::Calendar(CalendarMessage::SetViewMode(
        CalendarViewMode::Month,
    )));

    let initial_count = app.calendar_state.events.len();
    app.update(AppMessage::Calendar(CalendarMessage::ToggleCreateForm));
    app.update(AppMessage::Calendar(CalendarMessage::NewTitleChanged(
        "Prova de Física".to_string(),
    )));
    app.update(AppMessage::Calendar(CalendarMessage::CreateEvent));
    assert_eq!(app.calendar_state.events.len(), initial_count + 1);

    let first_proj_id = app.projects_state.projects[0].id.clone();
    app.update(AppMessage::Calendar(CalendarMessage::ToggleProjectFilter(
        first_proj_id.clone(),
    )));
    let filtered = app
        .calendar_state
        .visible_occurrences(&app.projects_state.tasks);
    assert!(
        filtered
            .iter()
            .all(|occ| occ.project_id.as_ref() == Some(&first_proj_id))
    );
}

#[test]
fn should_execute_inline_command_palette_actions_and_global_search() {
    let mut app = make_app();
    let initial_tasks = app.projects_state.tasks.len();

    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "> nova tarefa Ler capítulo 5".to_string(),
    )));
    app.update(AppMessage::Palette(PaletteMessage::ExecuteSelected));
    assert_eq!(app.route, AppRoute::Projects);
    assert_eq!(app.projects_state.tasks.len(), initial_tasks + 1);

    let initial_events = app.calendar_state.events.len();
    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "> novo evento Mentoria".to_string(),
    )));
    app.update(AppMessage::Palette(PaletteMessage::ExecuteSelected));
    assert_eq!(app.route, AppRoute::Calendar);
    assert_eq!(app.calendar_state.events.len(), initial_events + 1);

    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "Cálculo Vetorial".to_string(),
    )));
    let hits = app.palette.filtered_commands();
    assert!(!hits.is_empty());
    app.update(AppMessage::Palette(PaletteMessage::ExecuteSelected));
    assert_eq!(app.route, AppRoute::Projects);
    assert!(app.projects_state.drawer_task().is_some());
}
