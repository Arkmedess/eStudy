//! Testes de fluxo MVU para Pomodoro, Modo Foco em tela cheia, captura pós-sessão e DSP.

#![allow(clippy::expect_used)]

use chrono::{TimeZone, Utc};
use estudy_domain::{DspTrack, EntityId, FixedClock, FocusModality, FocusPhase, PomodoroPreset};
use estudy_ui::command_palette::PaletteMessage;
use estudy_ui::features::focus::FocusMessage;
use estudy_ui::features::home::{HomeDashboardData, HomeLoadState};
use estudy_ui::{AppMessage, EstudyApp};
use std::sync::Arc;

#[test]
fn should_run_pomodoro_lifecycle_fullscreen_esc_and_post_session_capture() {
    let clock = Arc::new(FixedClock::new(
        Utc.with_ymd_and_hms(2026, 11, 23, 14, 0, 0)
            .single()
            .expect("horário válido"),
    ));
    let mut app = EstudyApp::new(
        clock,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    );

    assert!(!app.focus_state.fullscreen_active);
    assert!(!app.is_focus_ticking());

    // Entrada rápida pela barra superior abre Modo Foco em tela cheia.
    app.update(AppMessage::QuickFocusShortcut);
    assert!(app.focus_state.fullscreen_active);

    // Esc sai da tela cheia sem descartar o estado do timer.
    app.update(AppMessage::Focus(FocusMessage::Start));
    assert_eq!(app.focus_state.timer.phase, FocusPhase::Focusing);
    assert!(app.is_focus_ticking());
    assert!(app.focus_state.audio_playing);

    app.update(AppMessage::CloseOverlays);
    assert!(!app.focus_state.fullscreen_active);
    assert!(app.is_focus_ticking());

    // Comando > focar 50m seleciona bloco de 50 minutos e entra em Modo Foco.
    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "> focar 50m".to_string(),
    )));
    app.update(AppMessage::Palette(PaletteMessage::ExecuteSelected));
    assert!(app.focus_state.fullscreen_active);
    assert_eq!(app.focus_state.timer.preset, PomodoroPreset::Deep50_10);

    // Seleção de faixa DSP e volume sem erro.
    app.update(AppMessage::Focus(FocusMessage::SelectDspTrack(
        DspTrack::WarmDrone,
    )));
    app.update(AppMessage::Focus(FocusMessage::SetVolumePercent(45)));
    assert_eq!(app.focus_state.dsp_track, DspTrack::WarmDrone);
    assert_eq!(app.focus_state.volume_percent, 45);

    // Avança os 50 minutos até completar e abrir captura pós-sessão.
    app.update(AppMessage::Focus(FocusMessage::Start));
    app.update(AppMessage::Focus(FocusMessage::TickSeconds(50 * 60)));
    assert_eq!(app.focus_state.timer.phase, FocusPhase::Completed);
    assert!(!app.focus_state.audio_playing);
    assert!(app.focus_state.capture.open);

    app.update(AppMessage::Focus(FocusMessage::SelectModality(
        FocusModality::Questions,
    )));
    app.update(AppMessage::Focus(FocusMessage::CaptureNoteChanged(
        "Revisado algoritmo de Dijkstra".to_string(),
    )));
    app.update(AppMessage::Focus(FocusMessage::QuestionsTotalChanged(
        "10".to_string(),
    )));
    app.update(AppMessage::Focus(FocusMessage::QuestionsRightChanged(
        "8".to_string(),
    )));
    app.update(AppMessage::Focus(FocusMessage::SavePostSession));

    assert_eq!(app.focus_state.pending_sessions.len(), 1);
    let saved = &app.focus_state.pending_sessions[0];
    assert_eq!(saved.duration_minutes, 50);
    assert_eq!(saved.modality, FocusModality::Questions);
    assert_eq!(saved.questions_total, 10);
    assert_eq!(saved.questions_right, 8);
    assert_eq!(app.focus_state.today_focused_minutes, 50);
    assert!(!app.focus_state.capture.open);
}

#[test]
fn should_start_focus_linked_to_pin_from_home() {
    let clock = Arc::new(FixedClock::new(
        Utc.with_ymd_and_hms(2026, 11, 23, 14, 0, 0)
            .single()
            .expect("horário válido"),
    ));
    let mut app = EstudyApp::new(
        clock,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    );
    let task_id = EntityId::new_v7();
    app.update(AppMessage::Focus(FocusMessage::OpenWithTarget {
        title: "Estudar Sistemas".to_string(),
        task_id: Some(task_id.clone()),
        preset: PomodoroPreset::Classic25_5,
    }));

    assert!(app.focus_state.fullscreen_active);
    assert_eq!(
        app.focus_state.active_target_title.as_deref(),
        Some("Estudar Sistemas")
    );
    assert_eq!(app.focus_state.active_task_id, Some(task_id));
}
