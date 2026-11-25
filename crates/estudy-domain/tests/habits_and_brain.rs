//! Testes TDD para domínio de Hábitos e Segundo Cérebro.

#![allow(clippy::expect_used)]

use chrono::{DateTime, NaiveDate};
use estudy_domain::{
    EntityId, Habit, KnowledgeNote, NoteKind, days_in_month, overall_monthly_adherence_pct,
};
use std::collections::BTreeSet;

#[test]
fn should_toggle_habits_compute_streak_and_monthly_adherence() {
    let today = NaiveDate::from_ymd_opt(2026, 11, 10).expect("data válida");
    let mut habit = Habit {
        id: EntityId::new_v7(),
        title: "Leitura".to_string(),
        subtitle: "20 min antes de dormir".to_string(),
        color_index: 0,
        weekly_goal: 6,
        completed_dates: BTreeSet::new(),
    };

    for day in [6, 7, 8, 9, 10] {
        let d = NaiveDate::from_ymd_opt(2026, 11, day).expect("data válida");
        assert!(habit.toggle_on(d));
    }

    assert_eq!(habit.current_streak(today), 5);
    assert_eq!(habit.completions_in_month(2026, 11), 5);
    assert_eq!(habit.monthly_adherence_pct(2026, 11, today), 50);
    assert_eq!(days_in_month(2026, 11), 30);

    // Desmarcar hoje preserva sequência de ontem (4 dias)
    assert!(!habit.toggle_on(today));
    assert_eq!(habit.current_streak(today), 4);
    assert_eq!(overall_monthly_adherence_pct(&[habit], 2026, 11, today), 40);
}

#[test]
fn should_filter_second_brain_notes_and_build_clean_excerpt() {
    let now = DateTime::from_timestamp(1_760_000_000, 0).expect("timestamp válido");
    let note = KnowledgeNote {
        id: EntityId::new_v7(),
        title: "Arquitetura Local-First em Rust".to_string(),
        kind: NoteKind::Reference,
        body_md: "Linha 1 sobre SQLite WAL.\n\nLinha 2 sobre memória enxuta.".to_string(),
        source_url: Some("https://example.org/local-first".to_string()),
        tags: vec!["rust".to_string(), "arquitetura".to_string()],
        pinned: true,
        created_at: now,
        updated_at: now,
    };

    assert!(note.matches_filter(Some(NoteKind::Reference), Some("RUST"), "sqlite"));
    assert!(!note.matches_filter(Some(NoteKind::Idea), None, ""));
    assert!(!note.matches_filter(None, Some("design"), ""));

    let excerpt = note.preview_excerpt(30);
    assert!(excerpt.ends_with('…'));
    assert!(excerpt.chars().count() <= 30);
}
