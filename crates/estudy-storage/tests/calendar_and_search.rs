//! Testes de integração para persistência de eventos de calendário e busca global `FTS5`.

#![allow(clippy::expect_used)]

use chrono::{DateTime, NaiveDate, Utc};
use estudy_domain::{
    CalendarEvent, DomainEvent, EntityId, InMemoryEventBus, Project, ProjectFramework,
    ProjectStatus, RecurrenceRule, Task, TaskPriority, TaskStatus,
};
use estudy_storage::{
    SearchEntityKind, SqliteCalendarRepository, SqliteProjectRepository, SqliteSearchRepository,
    apply_migrations, open_in_memory,
};

fn utc(year: i32, month: u32, day: u32, hour: u32, min: u32) -> DateTime<Utc> {
    let date = NaiveDate::from_ymd_opt(year, month, day).expect("data válida");
    let dt = date.and_hms_opt(hour, min, 0).expect("hora válida");
    DateTime::from_naive_utc_and_offset(dt, Utc)
}

#[test]
fn should_persist_calendar_events_with_recurrence_and_tags() {
    let mut conn = open_in_memory().expect("banco em memória");
    apply_migrations(&mut conn, None, utc(2026, 11, 22, 8, 0)).expect("migração");
    let bus = InMemoryEventBus::new();

    let event = CalendarEvent {
        id: EntityId::new_v7(),
        title: "Seminário de Rust".to_string(),
        description: "Apresentação semanal".to_string(),
        project_id: None,
        starts_at: utc(2026, 11, 22, 14, 0),
        ends_at: utc(2026, 11, 22, 15, 30),
        all_day: false,
        recurrence: Some(RecurrenceRule::Weekly),
        color_token: "accent".to_string(),
        tags: vec!["rust".to_string(), "palestra".to_string()],
        created_at: utc(2026, 11, 22, 8, 0),
        updated_at: utc(2026, 11, 22, 8, 0),
    };

    SqliteCalendarRepository::save_event(&mut conn, &event, Some(&bus)).expect("salvar evento");
    let loaded = SqliteCalendarRepository::list_events(&conn).expect("listar eventos");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].recurrence, Some(RecurrenceRule::Weekly));
    assert_eq!(loaded[0].tags.len(), 2);
    assert!(matches!(
        bus.snapshot().last(),
        Some(DomainEvent::CalendarEventSaved { .. })
    ));

    SqliteCalendarRepository::soft_delete_event(&mut conn, &event.id, utc(2026, 11, 22, 16, 0))
        .expect("soft delete");
    assert!(
        SqliteCalendarRepository::list_events(&conn)
            .expect("listar vazio")
            .is_empty()
    );
}

#[test]
fn should_search_across_fts5_notes_projects_tasks_and_events() {
    let mut conn = open_in_memory().expect("banco em memória");
    apply_migrations(&mut conn, None, utc(2026, 11, 22, 8, 0)).expect("migração");

    let proj = Project {
        id: EntityId::new_v7(),
        title: "Estudo de Termodinâmica".to_string(),
        description: String::new(),
        color_token: "accent".to_string(),
        status: ProjectStatus::InProgress,
        framework: ProjectFramework::Kanban,
        sort_order: 0,
        created_at: utc(2026, 11, 22, 8, 0),
        updated_at: utc(2026, 11, 22, 8, 0),
    };
    SqliteProjectRepository::save_project(&mut conn, &proj, None).expect("salvar proj");

    let task = Task {
        id: EntityId::new_v7(),
        title: "Resolver exercícios de termodinâmica".to_string(),
        description_md: String::new(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        eisenhower_quadrant: None,
        pinned_focus: false,
        start_at: None,
        due_at: None,
        recurrence: None,
        estimated_pomos: 1,
        completed_pomos: 0,
        project_links: Vec::new(),
        tags: Vec::new(),
        checklist: Vec::new(),
        created_at: utc(2026, 11, 22, 8, 0),
        updated_at: utc(2026, 11, 22, 8, 0),
    };
    SqliteProjectRepository::save_task(&mut conn, &task, None).expect("salvar tarefa");

    let note_id = EntityId::new_v7().to_string();
    conn.execute(
        "INSERT INTO notes (id, slug, title, kind, summary, body_md, created_at, updated_at)
         VALUES (?1, 'entropia', 'Entropia e Leis', 'subject', 'Termodinâmica clássica', 'Segunda lei da termodinâmica', '2026-11-22T08:00:00Z', '2026-11-22T08:00:00Z')",
        [&note_id],
    )
    .expect("inserir nota");

    let hits = SqliteSearchRepository::search_all(&conn, "termodinamica", 20)
        .expect("busca global unificada");
    assert!(hits.iter().any(|hit| hit.kind == SearchEntityKind::Project));
    assert!(hits.iter().any(|hit| hit.kind == SearchEntityKind::Task));
    assert!(hits.iter().any(|hit| hit.kind == SearchEntityKind::Note));

    let punctuation_safe = SqliteSearchRepository::search_all(&conn, "termo* \"(:!)", 10)
        .expect("caracteres especiais não devem quebrar FTS5");
    assert!(!punctuation_safe.is_empty());

    let pure_punctuation = SqliteSearchRepository::search_all(&conn, "\"(:!)*", 10)
        .expect("pontuação isolada deve retornar vazio sem erro");
    assert!(pure_punctuation.is_empty());
}
