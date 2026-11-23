//! Testes de integração para persistência de Projetos, Tarefas, Checklists e Tags no `SQLite`.

#![allow(clippy::expect_used)]

use chrono::{DateTime, Utc};
use estudy_domain::{
    DomainError, DomainEvent, EisenhowerQuadrant, EntityId, InMemoryEventBus,
    MAX_FOCUS_PINNED_TASKS, Project, ProjectFramework, ProjectStatus, Task, TaskChecklistItem,
    TaskPriority, TaskProjectLink, TaskStatus,
};
use estudy_storage::{SqliteProjectRepository, StorageError, apply_migrations, open_in_memory};

fn now_utc(offset: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(1_760_000_000 + offset, 0).expect("timestamp válido")
}

#[test]
fn should_persist_projects_multi_project_tasks_tags_and_checklist() {
    let mut conn = open_in_memory().expect("banco em memória");
    apply_migrations(&mut conn, None, now_utc(0)).expect("migração v1");
    let bus = InMemoryEventBus::new();

    let proj_a = Project {
        id: EntityId::new_v7(),
        title: "eStudy Desktop".to_string(),
        description: "Aplicativo em Rust".to_string(),
        color_token: "accent".to_string(),
        status: ProjectStatus::InProgress,
        framework: ProjectFramework::Kanban,
        sort_order: 0,
        created_at: now_utc(1),
        updated_at: now_utc(1),
    };
    let proj_b = Project {
        id: EntityId::new_v7(),
        title: "TCC".to_string(),
        description: String::new(),
        color_token: "warning".to_string(),
        status: ProjectStatus::Planning,
        framework: ProjectFramework::Milestones,
        sort_order: 1,
        created_at: now_utc(2),
        updated_at: now_utc(2),
    };

    SqliteProjectRepository::save_project(&mut conn, &proj_a, Some(&bus)).expect("salvar proj A");
    SqliteProjectRepository::save_project(&mut conn, &proj_b, Some(&bus)).expect("salvar proj B");
    SqliteProjectRepository::update_project_framework(
        &mut conn,
        &proj_a.id,
        ProjectFramework::Eisenhower,
        now_utc(3),
    )
    .expect("atualizar framework");

    let projects = SqliteProjectRepository::list_projects(&conn).expect("listar projetos");
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].framework, ProjectFramework::Eisenhower);

    let task = Task {
        id: EntityId::new_v7(),
        title: "Construir Drawer da Tarefa".to_string(),
        description_md: "# Detalhes\n- Suporte a **Markdown**".to_string(),
        status: TaskStatus::InProgress,
        priority: TaskPriority::High,
        eisenhower_quadrant: Some(EisenhowerQuadrant::Do),
        pinned_focus: true,
        start_at: Some("2026-11-22T09:00:00Z".to_string()),
        due_at: Some("2026-11-23T18:00:00Z".to_string()),
        recurrence: Some("weekly".to_string()),
        estimated_pomos: 3,
        completed_pomos: 1,
        project_links: vec![
            TaskProjectLink {
                project_id: proj_a.id.clone(),
                milestone_name: Some("Entrega v0.2.0".to_string()),
            },
            TaskProjectLink {
                project_id: proj_b.id.clone(),
                milestone_name: None,
            },
        ],
        tags: vec!["rust".to_string(), "ui".to_string()],
        checklist: vec![
            TaskChecklistItem {
                id: EntityId::new_v7(),
                title: "Criar layout lateral".to_string(),
                done: true,
                sort_order: 0,
            },
            TaskChecklistItem {
                id: EntityId::new_v7(),
                title: "Ligar preview Markdown".to_string(),
                done: false,
                sort_order: 1,
            },
        ],
        created_at: now_utc(4),
        updated_at: now_utc(4),
    };

    SqliteProjectRepository::save_task(&mut conn, &task, Some(&bus)).expect("salvar tarefa");
    let loaded = SqliteProjectRepository::list_tasks(&conn).expect("listar tarefas");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].project_links.len(), 2);
    assert_eq!(loaded[0].tags.len(), 2);
    assert_eq!(loaded[0].checklist.len(), 2);
    assert_eq!(loaded[0].checklist_progress(), (1, 2));
    assert!(loaded[0].pinned_focus);
    assert!(matches!(
        bus.snapshot().last(),
        Some(DomainEvent::TaskSaved { .. })
    ));

    SqliteProjectRepository::soft_delete_task(&mut conn, &task.id, now_utc(5))
        .expect("soft delete");
    assert!(
        SqliteProjectRepository::list_tasks(&conn)
            .expect("listar após soft delete")
            .is_empty()
    );
}

#[test]
fn should_enforce_focus_pin_budget_of_5_in_sqlite_transactions() {
    let mut conn = open_in_memory().expect("banco em memória");
    apply_migrations(&mut conn, None, now_utc(0)).expect("migração v1");

    for idx in 0..MAX_FOCUS_PINNED_TASKS {
        let task = Task {
            id: EntityId::new_v7(),
            title: format!("Pin {idx}"),
            description_md: String::new(),
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            eisenhower_quadrant: None,
            pinned_focus: true,
            start_at: None,
            due_at: None,
            recurrence: None,
            estimated_pomos: 1,
            completed_pomos: 0,
            project_links: Vec::new(),
            tags: Vec::new(),
            checklist: Vec::new(),
            created_at: now_utc(1),
            updated_at: now_utc(1),
        };
        SqliteProjectRepository::save_task(&mut conn, &task, None).expect("até 5 pins ok");
    }

    let extra = Task {
        id: EntityId::new_v7(),
        title: "Pin 6".to_string(),
        description_md: String::new(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        eisenhower_quadrant: None,
        pinned_focus: true,
        start_at: None,
        due_at: None,
        recurrence: None,
        estimated_pomos: 1,
        completed_pomos: 0,
        project_links: Vec::new(),
        tags: Vec::new(),
        checklist: Vec::new(),
        created_at: now_utc(2),
        updated_at: now_utc(2),
    };

    let err = SqliteProjectRepository::save_task(&mut conn, &extra, None)
        .expect_err("6º pin no banco deve falhar");
    assert!(matches!(
        err,
        StorageError::Domain(DomainError::FocusPinLimitReached { max: 5 })
    ));
}
