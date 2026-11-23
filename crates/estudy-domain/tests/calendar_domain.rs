//! Testes TDD para Calendário Unificado, recorrência, eventos multi-dia e Timeline "Hoje".

#![allow(clippy::expect_used)]

use chrono::{DateTime, Duration, NaiveDate, Utc};
use estudy_domain::{
    CalendarEvent, DomainError, EisenhowerQuadrant, EntityId, RecurrenceRule, Task, TaskPriority,
    TaskStatus, TimelineSlot, build_today_timeline, expand_calendar_items,
    format_free_window_label,
};

fn utc(year: i32, month: u32, day: u32, hour: u32, min: u32) -> DateTime<Utc> {
    let date = NaiveDate::from_ymd_opt(year, month, day).expect("data válida");
    let dt = date.and_hms_opt(hour, min, 0).expect("hora válida");
    DateTime::from_naive_utc_and_offset(dt, Utc)
}

#[test]
fn should_reject_event_when_end_is_before_start() {
    let event = CalendarEvent {
        id: EntityId::new_v7(),
        title: "Inválido".to_string(),
        description: String::new(),
        project_id: None,
        starts_at: utc(2026, 11, 22, 14, 0),
        ends_at: utc(2026, 11, 22, 13, 0),
        all_day: false,
        recurrence: None,
        color_token: "accent".to_string(),
        tags: Vec::new(),
        created_at: utc(2026, 11, 22, 8, 0),
        updated_at: utc(2026, 11, 22, 8, 0),
    };
    assert!(matches!(
        event.validate(),
        Err(DomainError::InvalidEventTimeRange { .. })
    ));
}

#[test]
fn should_expand_multiday_and_weekly_recurring_events_and_task_deadlines() {
    let weekly_event = CalendarEvent {
        id: EntityId::new_v7(),
        title: "Aula de Cálculo".to_string(),
        description: String::new(),
        project_id: None,
        starts_at: utc(2026, 11, 2, 10, 0),
        ends_at: utc(2026, 11, 2, 12, 0),
        all_day: false,
        recurrence: Some(RecurrenceRule::Weekly),
        color_token: "accent".to_string(),
        tags: vec!["estudo".to_string()],
        created_at: utc(2026, 11, 1, 8, 0),
        updated_at: utc(2026, 11, 1, 8, 0),
    };

    let multiday_event = CalendarEvent {
        id: EntityId::new_v7(),
        title: "Semana Acadêmica".to_string(),
        description: String::new(),
        project_id: None,
        starts_at: utc(2026, 11, 10, 0, 0),
        ends_at: utc(2026, 11, 12, 23, 59),
        all_day: true,
        recurrence: None,
        color_token: "warning".to_string(),
        tags: Vec::new(),
        created_at: utc(2026, 11, 1, 8, 0),
        updated_at: utc(2026, 11, 1, 8, 0),
    };

    let done_task = Task {
        id: EntityId::new_v7(),
        title: "Entregar resumo".to_string(),
        description_md: String::new(),
        status: TaskStatus::Done,
        priority: TaskPriority::Medium,
        eisenhower_quadrant: Some(EisenhowerQuadrant::Do),
        pinned_focus: false,
        start_at: None,
        due_at: Some("2026-11-11T18:00:00Z".to_string()),
        recurrence: None,
        estimated_pomos: 1,
        completed_pomos: 1,
        project_links: Vec::new(),
        tags: Vec::new(),
        checklist: Vec::new(),
        created_at: utc(2026, 11, 1, 8, 0),
        updated_at: utc(2026, 11, 1, 8, 0),
    };

    let start = NaiveDate::from_ymd_opt(2026, 11, 1).expect("início");
    let end = NaiveDate::from_ymd_opt(2026, 11, 20).expect("fim");
    let occurrences =
        expand_calendar_items(&[weekly_event, multiday_event], &[done_task], start, end);

    let weekly_hits = occurrences
        .iter()
        .filter(|occ| occ.title == "Aula de Cálculo")
        .count();
    assert_eq!(weekly_hits, 3); // dias 2, 9 e 16 de novembro

    let multiday_hits = occurrences
        .iter()
        .filter(|occ| occ.title == "Semana Acadêmica")
        .count();
    assert_eq!(multiday_hits, 3); // dias 10, 11 e 12

    let task_hit = occurrences
        .iter()
        .find(|occ| occ.is_task_deadline)
        .expect("tarefa com prazo deve aparecer");
    assert!(task_hit.is_completed);
    assert_eq!(
        task_hit.date,
        NaiveDate::from_ymd_opt(2026, 11, 11).expect("data")
    );
}

#[test]
fn should_build_today_timeline_with_unscheduled_free_windows_and_now_marker() {
    let now = utc(2026, 11, 22, 11, 30);
    let morning_block = CalendarEvent {
        id: EntityId::new_v7(),
        title: "Treino".to_string(),
        description: String::new(),
        project_id: None,
        starts_at: utc(2026, 11, 22, 8, 0),
        ends_at: utc(2026, 11, 22, 9, 0),
        all_day: false,
        recurrence: None,
        color_token: "accent".to_string(),
        tags: Vec::new(),
        created_at: now - Duration::days(1),
        updated_at: now - Duration::days(1),
    };
    let afternoon_block = CalendarEvent {
        id: EntityId::new_v7(),
        title: "Estudo dirigido".to_string(),
        description: String::new(),
        project_id: None,
        starts_at: utc(2026, 11, 22, 10, 45),
        ends_at: utc(2026, 11, 22, 12, 0),
        all_day: false,
        recurrence: None,
        color_token: "accent".to_string(),
        tags: Vec::new(),
        created_at: now - Duration::days(1),
        updated_at: now - Duration::days(1),
    };
    let all_day = CalendarEvent {
        id: EntityId::new_v7(),
        title: "Feriado Municipal".to_string(),
        description: String::new(),
        project_id: None,
        starts_at: utc(2026, 11, 22, 0, 0),
        ends_at: utc(2026, 11, 22, 23, 59),
        all_day: true,
        recurrence: None,
        color_token: "warning".to_string(),
        tags: Vec::new(),
        created_at: now - Duration::days(1),
        updated_at: now - Duration::days(1),
    };

    let timeline = build_today_timeline(&[morning_block, afternoon_block, all_day], &[], now);
    assert_eq!(timeline.now_minute, 11 * 60 + 30);
    assert_eq!(timeline.unscheduled.len(), 1);
    assert_eq!(timeline.unscheduled[0].title, "Feriado Municipal");

    assert_eq!(format_free_window_label(105), "Livre · 1h45");
    assert!(timeline.slots.iter().any(|slot| matches!(
        slot,
        TimelineSlot::Free { duration_minutes: 105, label, .. } if label == "Livre · 1h45"
    )));
}
