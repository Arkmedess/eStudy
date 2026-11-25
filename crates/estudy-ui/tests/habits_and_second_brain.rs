//! Testes comportamentais MVU para Hábitos, Segundo Cérebro e ações inline na Home.

#![allow(clippy::expect_used)]

use chrono::DateTime;
use estudy_domain::{FixedClock, NoteKind};
use estudy_ui::{
    AppMessage, AppRoute, BrainEditorTab, EstudyApp, HabitsMessage, HomeDashboardData,
    HomeLoadState, QuickPopup, SecondBrainMessage,
};
use std::sync::Arc;

fn make_app() -> EstudyApp {
    let base = DateTime::from_timestamp(1_760_000_000, 0).expect("timestamp válido");
    let clock = Arc::new(FixedClock::new(base));
    EstudyApp::new(
        clock,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    )
}

#[test]
fn should_toggle_habits_create_new_habit_and_shift_week_month() {
    let mut app = make_app();
    let today = app.habits_state.week_start;
    let initial_count = app.habits_state.habits.len();
    let first_id = app.habits_state.habits[0].id.clone();
    let was_done = app.habits_state.habits[0].is_done_on(today);

    app.update(AppMessage::Habits(HabitsMessage::ToggleHabitDate {
        habit_id: first_id,
        date: today,
    }));
    assert_eq!(app.habits_state.habits[0].is_done_on(today), !was_done);

    app.update(AppMessage::Habits(HabitsMessage::NewTitleChanged(
        "Meditar 10m".to_string(),
    )));
    app.update(AppMessage::Habits(HabitsMessage::CreateHabit));
    assert_eq!(app.habits_state.habits.len(), initial_count + 1);
    assert_eq!(
        app.habits_state.habits.last().map(|h| h.title.as_str()),
        Some("Meditar 10m")
    );

    let prev_month = app.habits_state.view_month;
    app.update(AppMessage::Habits(HabitsMessage::ShiftMonth(1)));
    assert_ne!(app.habits_state.view_month, prev_month);
}

#[test]
fn should_create_filter_edit_and_open_second_brain_notes_from_home() {
    let mut app = make_app();
    let initial_notes = app.second_brain_state.notes.len();

    // Selecionar nota direto da Home abre o Popup de leitura rápida sem sair da Home
    let first_note_id = app.second_brain_state.notes[0].id.clone();
    assert_eq!(app.route, AppRoute::Home);
    app.update(AppMessage::SecondBrain(SecondBrainMessage::SelectNote(
        first_note_id.clone(),
    )));
    assert_eq!(app.route, AppRoute::Home);
    assert_eq!(app.active_popup, Some(QuickPopup::Note(first_note_id)));

    // Abrir outros popups rápidos e fechar com CloseOverlays
    app.update(AppMessage::OpenQuickPopup(QuickPopup::HabitsQuick));
    assert_eq!(app.active_popup, Some(QuickPopup::HabitsQuick));
    app.update(AppMessage::CloseOverlays);
    assert_eq!(app.active_popup, None);

    app.update(AppMessage::Navigate(AppRoute::SecondBrain));
    assert_eq!(app.route, AppRoute::SecondBrain);

    // Criar nova nota abre em modo escrita e permite editar título, corpo, tipo e tags
    app.update(AppMessage::SecondBrain(SecondBrainMessage::CreateNote));
    assert_eq!(app.second_brain_state.notes.len(), initial_notes + 1);
    assert_eq!(app.second_brain_state.editor_tab, BrainEditorTab::Write);

    app.update(AppMessage::SecondBrain(SecondBrainMessage::TitleChanged(
        "Resumo de Rust".to_string(),
    )));
    app.update(AppMessage::SecondBrain(SecondBrainMessage::KindChanged(
        NoteKind::Idea,
    )));
    app.update(AppMessage::SecondBrain(
        SecondBrainMessage::NewTagInputChanged("#sistemas".to_string()),
    ));
    app.update(AppMessage::SecondBrain(SecondBrainMessage::AddTag));

    let selected = app
        .second_brain_state
        .selected_note()
        .expect("nota selecionada");
    assert_eq!(selected.title, "Resumo de Rust");
    assert_eq!(selected.kind, NoteKind::Idea);
    assert!(selected.tags.iter().any(|t| t == "sistemas"));

    app.update(AppMessage::SecondBrain(SecondBrainMessage::SetKindFilter(
        Some(NoteKind::Idea),
    )));
    assert!(
        app.second_brain_state
            .filtered_notes()
            .iter()
            .all(|n| n.kind == NoteKind::Idea)
    );
}
