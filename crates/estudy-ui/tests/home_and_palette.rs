//! Testes comportamentais MVU da Home em 3 colunas e Barra de Comando (`Ctrl+K`).

#![allow(clippy::expect_used)]

use chrono::DateTime;
use estudy_domain::FixedClock;
use estudy_ui::{
    AppMessage, AppRoute, CommandPaletteState, EstudyApp, FocusPinItem, HomeDashboardData,
    HomeLoadState, HomeMessage, MAX_FOCUS_PINS, MoodLevel, MoodViewMode, PaletteMessage,
};
use estudy_ui_kit::{BoardFrameStyle, ThemeMode, ThemePreset};
use std::sync::Arc;

fn make_app(hour_utc: i64, state: HomeLoadState) -> EstudyApp {
    let base = DateTime::from_timestamp(1_760_000_000, 0).expect("timestamp válido");
    let day_start = base - chrono::Duration::seconds(base.timestamp().rem_euclid(86_400));
    let clock = Arc::new(FixedClock::new(
        day_start + chrono::Duration::hours(hour_utc),
    ));
    EstudyApp::new(clock, state)
}

#[test]
fn should_greet_according_to_injected_clock_without_reading_system_time() {
    let morning = make_app(9, HomeLoadState::Loading);
    assert_eq!(morning.greeting(), "Bom dia");

    let afternoon = make_app(15, HomeLoadState::Loading);
    assert_eq!(afternoon.greeting(), "Boa tarde");

    let night = make_app(21, HomeLoadState::Loading);
    assert_eq!(night.greeting(), "Boa noite");
}

#[test]
fn should_filter_and_execute_command_palette_actions() {
    let mut app = make_app(
        10,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    );

    assert!(!app.palette.is_open);
    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    assert!(app.palette.is_open);

    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "focar 25m".to_string(),
    )));
    assert_eq!(app.palette.filtered_commands().len(), 1);

    app.update(AppMessage::Palette(PaletteMessage::ExecuteSelected));
    assert!(!app.palette.is_open);
    assert!(
        app.status_banner
            .as_deref()
            .is_some_and(|banner| banner.contains("25 min"))
    );

    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "estudos".to_string(),
    )));
    app.update(AppMessage::Palette(PaletteMessage::ExecuteSelected));
    assert_eq!(app.route, AppRoute::Studies);
}

#[test]
fn should_wrap_command_palette_selection_and_handle_empty_matches() {
    let mut palette = CommandPaletteState::new();
    palette.open();

    let total = palette.filtered_commands().len();
    assert!(total >= 10);

    palette.select_prev();
    assert_eq!(palette.selected_index, total - 1);

    palette.select_next();
    assert_eq!(palette.selected_index, 0);

    palette.set_query("termo-inexistente-xyz".to_string());
    assert!(palette.filtered_commands().is_empty());
    palette.select_next();
    assert_eq!(palette.selected_index, 0);
}

#[test]
fn should_enforce_max_5_focus_pins_and_update_mood_and_frame() {
    let mut data = HomeDashboardData::empty("Arthur");
    assert!(data.is_empty_workspace());

    for idx in 0..MAX_FOCUS_PINS {
        let added = data.pin_focus_item(FocusPinItem {
            id: format!("pin-{idx}"),
            title: format!("Item {idx}"),
            context_label: "Projeto".to_string(),
            completed_pomos: 0,
            estimated_pomos: 1,
        });
        assert!(added);
    }
    let overflow = data.pin_focus_item(FocusPinItem {
        id: "pin-extra".to_string(),
        title: "Excedente".to_string(),
        context_label: "Projeto".to_string(),
        completed_pomos: 0,
        estimated_pomos: 1,
    });
    assert!(!overflow);
    assert_eq!(data.focus_items.len(), MAX_FOCUS_PINS);

    let mut app = make_app(14, HomeLoadState::Ready(Box::new(data)));
    app.update(AppMessage::Home(HomeMessage::SelectTodayMood(
        MoodLevel::Great,
    )));
    app.update(AppMessage::Home(HomeMessage::SelectBoardFrame(
        BoardFrameStyle::Polaroid,
    )));
    app.update(AppMessage::CycleThemePreset);
    app.update(AppMessage::ToggleThemeMode);

    assert_eq!(app.loadout.board_frame, BoardFrameStyle::Polaroid);
    assert_eq!(app.loadout.theme_preset, ThemePreset::EstudyMenta);
    assert_eq!(app.loadout.theme_mode, ThemeMode::Dark);
    assert!(matches!(
        &app.home_state,
        HomeLoadState::Ready(home)
            if home.mood_week.last().and_then(|day| day.level) == Some(MoodLevel::Great)
    ));
}

#[test]
fn should_recover_from_error_state_and_seed_from_empty_state() {
    let mut err_app = make_app(10, HomeLoadState::Error("Falha simulada".to_string()));
    err_app.update(AppMessage::Home(HomeMessage::RetryLoad));
    assert!(matches!(err_app.home_state, HomeLoadState::Ready(_)));

    let mut empty_app = make_app(
        10,
        HomeLoadState::Ready(Box::new(HomeDashboardData::empty("Arthur"))),
    );
    empty_app.update(AppMessage::Home(HomeMessage::SeedStarterWorkspace));
    assert!(matches!(
        &empty_app.home_state,
        HomeLoadState::Ready(home) if !home.is_empty_workspace()
    ));
}

#[test]
fn should_toggle_modules_menu_notifications_and_mood_year_view() {
    let mut app = make_app(
        10,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    );

    assert!(!app.modules_menu_open);
    assert!(!app.notifications_open);

    app.update(AppMessage::ToggleModulesMenu);
    assert!(app.modules_menu_open);
    assert!(!app.notifications_open);

    app.update(AppMessage::ToggleNotifications);
    assert!(app.notifications_open);
    assert!(!app.modules_menu_open);

    app.update(AppMessage::Navigate(AppRoute::Shop));
    assert_eq!(app.route, AppRoute::Shop);
    assert!(!app.notifications_open);

    app.update(AppMessage::Home(HomeMessage::ToggleMoodViewMode));
    assert!(matches!(
        &app.home_state,
        HomeLoadState::Ready(home) if home.mood_view == MoodViewMode::Year
    ));

    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "+10p".to_string(),
    )));
    assert_eq!(app.palette.filtered_commands().len(), 1);
}
