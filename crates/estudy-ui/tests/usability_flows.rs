//! Testes de usabilidade e fluxos de uso ponta-a-ponta do eStudy (`v0.6.1`).

use std::sync::Arc;

use chrono::{TimeZone, Utc};
use estudy_domain::FixedClock;
use estudy_ui::command_palette::{AppRoute, PaletteMessage};
use estudy_ui::features::habits::HabitsMessage;
use estudy_ui::features::home::{HomeDashboardData, HomeLoadState};
use estudy_ui::features::projects::ProjectsMessage;
use estudy_ui::{AppMessage, EstudyApp, QuickPopup};
use estudy_ui_kit::{ThemeMode, ThemePreset};

fn fixed_now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 25, 10, 0, 0)
        .single()
        .unwrap_or_default()
}

fn make_app() -> EstudyApp {
    let clock = Arc::new(FixedClock::new(fixed_now()));
    EstudyApp::new(
        clock,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    )
}

/// UF-01: Ação rápida sem sair da tela atual (marcar hábito via `QuickPopup` em Projetos).
#[test]
fn uf01_should_toggle_habit_in_quick_popup_without_leaving_projects() {
    let mut app = make_app();
    let today = fixed_now().date_naive();

    app.update(AppMessage::Navigate(AppRoute::Projects));
    assert_eq!(app.route, AppRoute::Projects);

    let habit_id = app.habits_state.habits[0].id.clone();
    let before = app.habits_state.habits[0].is_done_on(today);

    app.update(AppMessage::OpenQuickPopup(QuickPopup::HabitsQuick));
    assert_eq!(app.active_popup, Some(QuickPopup::HabitsQuick));

    app.update(AppMessage::Habits(HabitsMessage::ToggleHabitDate {
        habit_id: habit_id.clone(),
        date: today,
    }));
    app.update(AppMessage::CloseOverlays);

    assert_eq!(app.route, AppRoute::Projects);
    assert!(app.active_popup.is_none());
    assert_ne!(app.habits_state.habits[0].is_done_on(today), before);
}

/// UF-02: Disparo de foco em 1 clique a partir de uma tarefa preserva contexto e ativa dock.
#[test]
fn uf02_should_start_focus_from_task_and_surface_bottom_status_dock() {
    let mut app = make_app();
    app.update(AppMessage::Navigate(AppRoute::Projects));
    let task_id = app.projects_state.tasks[0].id.clone();

    app.update(AppMessage::Projects(ProjectsMessage::StartFocusForTask(
        task_id.clone(),
    )));

    assert_eq!(app.focus_state.active_task_id, Some(task_id));
    assert!(app.is_focus_ticking());
    assert!(app.has_active_status_dock());
}

/// UF-03: Pilha previsível da tecla Esc fecha exatamente uma camada por vez.
#[test]
fn uf03_should_close_overlays_one_layer_at_a_time_on_escape() {
    let mut app = make_app();

    // Empilha 4 camadas: Gaveta Hoje -> Sidebar Sobreposta -> QuickPopup -> Ctrl+K
    app.update(AppMessage::ToggleTodayDrawer);
    app.update(AppMessage::ToggleNavSidebar);
    app.update(AppMessage::OpenQuickPopup(QuickPopup::HabitsQuick));
    app.update(AppMessage::Palette(PaletteMessage::Toggle));

    assert!(app.palette.is_open);
    assert!(app.active_popup.is_some());
    assert!(app.nav_sidebar_open);
    assert!(app.calendar_state.today_drawer_open);

    // 1º Esc: fecha apenas o Ctrl+K
    app.update(AppMessage::CloseOverlays);
    assert!(!app.palette.is_open);
    assert!(app.active_popup.is_some());
    assert!(app.nav_sidebar_open);
    assert!(app.calendar_state.today_drawer_open);

    // 2º Esc: fecha apenas o QuickPopup
    app.update(AppMessage::CloseOverlays);
    assert!(app.active_popup.is_none());
    assert!(app.nav_sidebar_open);
    assert!(app.calendar_state.today_drawer_open);

    // 3º Esc: fecha apenas a sidebar sobreposta
    app.update(AppMessage::CloseOverlays);
    assert!(!app.nav_sidebar_open);
    assert!(app.calendar_state.today_drawer_open);

    // 4º Esc: fecha a Gaveta Hoje
    app.update(AppMessage::CloseOverlays);
    assert!(!app.calendar_state.today_drawer_open);
}

/// UF-04: Sidebar sobreposta inicia fechada (sem empurrar), fecha ao navegar e evita dupla sidebar.
#[test]
fn uf04_should_collapse_overlay_sidebar_when_navigating_or_opening_context_sidebar() {
    let mut app = make_app();
    assert!(!app.nav_sidebar_open);

    app.update(AppMessage::ToggleNavSidebar);
    assert!(app.nav_sidebar_open);

    // Navegar fecha a sidebar sobreposta automaticamente para devolver 100% da área útil.
    app.update(AppMessage::Navigate(AppRoute::Projects));
    assert_eq!(app.route, AppRoute::Projects);
    assert!(!app.nav_sidebar_open);

    // Abrir sidebar sobreposta e acionar Ctrl+B em Projetos fecha a sobreposta (sem dupla sidebar).
    app.update(AppMessage::ToggleNavSidebar);
    assert!(app.nav_sidebar_open);
    app.update(AppMessage::ToggleContextSidebar);
    assert!(!app.nav_sidebar_open);
}

/// UF-05: Navegação primária expõe somente rotas funcionais reais (zero dead-ends).
#[test]
fn uf05_should_expose_only_live_routes_in_primary_navigation_without_dead_ends() {
    let primary = EstudyApp::primary_nav_routes();
    assert_eq!(
        primary,
        &[
            AppRoute::Home,
            AppRoute::Projects,
            AppRoute::Habits,
            AppRoute::SecondBrain,
            AppRoute::Calendar,
        ]
    );
    // Garante ausência de placeholders na navegação primária.
    assert!(!primary.contains(&AppRoute::Studies));
    assert!(!primary.contains(&AppRoute::Bookshelf));
    assert!(!primary.contains(&AppRoute::Finance));
    assert!(!primary.contains(&AppRoute::GardenAndPets));
    assert!(!primary.contains(&AppRoute::Shop));
    assert!(!primary.contains(&AppRoute::Focus));
}

/// UF-06: Seleção direta de tema em Configurações + captura global via Ctrl+K.
#[test]
fn uf06_should_select_theme_preset_in_settings_and_execute_global_command_palette() {
    let mut app = make_app();
    app.update(AppMessage::Navigate(AppRoute::Settings));
    assert_eq!(app.route, AppRoute::Settings);

    app.update(AppMessage::SelectThemePreset(ThemePreset::VelvetSakura));
    app.update(AppMessage::ToggleThemeMode);
    assert_eq!(app.loadout.theme_preset, ThemePreset::VelvetSakura);
    assert_eq!(app.loadout.theme_mode, ThemeMode::Dark);

    let before_tasks = app.projects_state.tasks.len();
    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "> nova tarefa Revisar arquitetura UX".to_string(),
    )));
    app.update(AppMessage::Palette(PaletteMessage::ExecuteSelected));

    assert!(!app.palette.is_open);
    assert_eq!(app.projects_state.tasks.len(), before_tasks + 1);
}

/// UF-07: Clicar em um projeto na Home abre `QuickPopup::ProjectOverview` sem sair da Home.
#[test]
fn uf07_should_preview_project_popup_from_home_without_leaving_route() {
    use estudy_ui::features::home::HomeMessage;

    let mut app = make_app();
    let title = app.projects_state.projects[0].title.clone();
    let expected_id = app.projects_state.projects[0].id.clone();

    app.update(AppMessage::Home(HomeMessage::PreviewProject(title)));
    assert_eq!(app.route, AppRoute::Home);
    assert_eq!(
        app.active_popup,
        Some(QuickPopup::ProjectOverview(expected_id))
    );
}
