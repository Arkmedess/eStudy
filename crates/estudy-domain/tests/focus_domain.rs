//! Testes unitários das regras puras de Pomodoro, Modo Foco e faixas DSP.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{TimeZone, Utc};
use estudy_domain::{
    DomainError, DspTrack, EntityId, FocusModality, FocusPhase, FocusSessionDraft,
    MINIMUM_REWARDABLE_FOCUS_MINUTES, PomodoroPreset, PomodoroTimer, calculate_focus_coins,
};

#[test]
fn should_advance_and_pause_pomodoro_timer_from_monotonic_seconds() {
    let mut timer = PomodoroTimer::new(PomodoroPreset::Classic25_5);
    assert_eq!(timer.phase, FocusPhase::Idle);
    assert_eq!(timer.remaining_seconds(), 25 * 60);

    timer.start_focus();
    assert_eq!(timer.phase, FocusPhase::Focusing);

    let completed = timer.advance_seconds(600);
    assert!(!completed);
    assert_eq!(timer.remaining_seconds(), 15 * 60);
    assert!((timer.progress_ratio() - 0.4).abs() < 0.01);

    timer.pause();
    assert_eq!(timer.phase, FocusPhase::Paused);
    assert!(!timer.advance_seconds(300));
    assert_eq!(timer.remaining_seconds(), 15 * 60);

    timer.resume();
    let completed_now = timer.advance_seconds(900);
    assert!(completed_now);
    assert_eq!(timer.phase, FocusPhase::Completed);
    assert_eq!(timer.focused_minutes_rounded(), 25);

    timer.start_break();
    assert_eq!(timer.phase, FocusPhase::ShortBreak);
    assert_eq!(timer.remaining_seconds(), 5 * 60);
}

#[test]
fn should_calculate_anti_farm_focus_coins_with_15_minute_minimum() {
    assert_eq!(MINIMUM_REWARDABLE_FOCUS_MINUTES, 15);
    assert_eq!(calculate_focus_coins(14), 0);
    assert_eq!(calculate_focus_coins(15), 3);
    assert_eq!(calculate_focus_coins(25), 5);
    assert_eq!(calculate_focus_coins(50), 10);
}

#[test]
fn should_validate_focus_session_draft_and_dsp_tracks() {
    let start = Utc.with_ymd_and_hms(2026, 11, 23, 14, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 11, 23, 14, 25, 0).unwrap();

    let invalid_duration = FocusSessionDraft {
        id: EntityId::new_v7(),
        modality: FocusModality::Focus,
        duration_minutes: 0,
        questions_total: 0,
        questions_right: 0,
        note_capture: None,
        task_id: None,
        started_at: start,
        ended_at: end,
    }
    .validate();
    assert!(matches!(
        invalid_duration,
        Err(DomainError::InvalidFocusSession(_))
    ));

    let invalid_questions = FocusSessionDraft {
        id: EntityId::new_v7(),
        modality: FocusModality::Questions,
        duration_minutes: 25,
        questions_total: 10,
        questions_right: 12,
        note_capture: Some("Revisar árvore B+".to_string()),
        task_id: None,
        started_at: start,
        ended_at: end,
    }
    .validate();
    assert!(matches!(
        invalid_questions,
        Err(DomainError::InvalidFocusSession(_))
    ));

    let valid = FocusSessionDraft {
        id: EntityId::new_v7(),
        modality: FocusModality::Focus,
        duration_minutes: 25,
        questions_total: 12,
        questions_right: 10,
        note_capture: Some("Concluído capítulo 3".to_string()),
        task_id: None,
        started_at: start,
        ended_at: end,
    }
    .validate()
    .expect("sessão válida");
    assert_eq!(valid.awarded_coins, 5);

    assert_eq!(DspTrack::ALL.len(), 5);
    for track in DspTrack::ALL {
        assert_eq!(DspTrack::parse(track.as_str()).unwrap(), track);
    }
}
