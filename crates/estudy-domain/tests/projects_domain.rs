//! Testes TDD para regras de Projetos, Checklists (0/20), Marcos e Pin "Em foco".

#![allow(clippy::expect_used)]

use chrono::{DateTime, Utc};
use estudy_domain::{
    DomainError, EisenhowerQuadrant, EntityId, MAX_FOCUS_PINNED_TASKS, MAX_TASK_CHECKLIST_ITEMS,
    ProjectFramework, ProjectStatus, Task, TaskPriority, TaskStatus, TaskTurn, classify_task_turn,
    validate_focus_pin_budget,
};

fn sample_utc() -> DateTime<Utc> {
    DateTime::from_timestamp(1_760_000_000, 0).expect("timestamp válido")
}

fn empty_task(status: TaskStatus) -> Task {
    Task {
        id: EntityId::new_v7(),
        title: "Implementar Kanban".to_string(),
        description_md: String::new(),
        status,
        priority: TaskPriority::Medium,
        eisenhower_quadrant: Some(EisenhowerQuadrant::Do),
        pinned_focus: false,
        start_at: None,
        due_at: None,
        recurrence: None,
        estimated_pomos: 2,
        completed_pomos: 0,
        project_links: Vec::new(),
        tags: Vec::new(),
        checklist: Vec::new(),
        created_at: sample_utc(),
        updated_at: sample_utc(),
    }
}

#[test]
fn should_roundtrip_project_and_task_enums_uniformly() {
    assert_eq!(
        ProjectStatus::parse(ProjectStatus::InProgress.as_str()).expect("status válido"),
        ProjectStatus::InProgress
    );
    assert_eq!(
        ProjectFramework::parse(ProjectFramework::Eisenhower.as_str()).expect("framework válido"),
        ProjectFramework::Eisenhower
    );
    assert_eq!(
        TaskStatus::parse(TaskStatus::Backlog.as_str()).expect("coluna válida"),
        TaskStatus::Backlog
    );
    assert_eq!(
        TaskPriority::parse(TaskPriority::Urgent.as_str()).expect("prioridade válida"),
        TaskPriority::Urgent
    );
    assert_eq!(
        EisenhowerQuadrant::parse(EisenhowerQuadrant::Schedule.as_str()).expect("quadrante válido"),
        EisenhowerQuadrant::Schedule
    );
}

#[test]
fn should_enforce_0_to_20_checklist_limit_and_report_progress() {
    let mut task = empty_task(TaskStatus::Todo);
    for idx in 0..MAX_TASK_CHECKLIST_ITEMS {
        task.add_checklist_item(format!("Passo {idx}"))
            .expect("deve aceitar até 20 itens");
    }

    let overflow = task
        .add_checklist_item("Item 21")
        .expect_err("21º item deve ser bloqueado");
    assert_eq!(
        overflow,
        DomainError::ChecklistLimitReached {
            max: MAX_TASK_CHECKLIST_ITEMS
        }
    );

    task.checklist[0].done = true;
    task.checklist[1].done = true;
    assert_eq!(task.checklist_progress(), (2, 20));
}

#[test]
fn should_compute_milestone_progress_and_enforce_focus_pin_budget() {
    let tasks = vec![
        empty_task(TaskStatus::Done),
        empty_task(TaskStatus::InProgress),
        empty_task(TaskStatus::Done),
    ];
    assert_eq!(Task::milestone_progress(&tasks), (2, 3));

    assert!(validate_focus_pin_budget(MAX_FOCUS_PINNED_TASKS, true, true).is_ok());
    assert!(validate_focus_pin_budget(MAX_FOCUS_PINNED_TASKS - 1, true, false).is_ok());
    let err = validate_focus_pin_budget(MAX_FOCUS_PINNED_TASKS, true, false)
        .expect_err("6º pin em foco deve falhar");
    assert_eq!(
        err,
        DomainError::FocusPinLimitReached {
            max: MAX_FOCUS_PINNED_TASKS
        }
    );
}

#[test]
fn should_roundtrip_weekly_turns_framework_and_classify_task_turns() {
    assert_eq!(
        ProjectFramework::parse(ProjectFramework::WeeklyTurns.as_str()).expect("framework válido"),
        ProjectFramework::WeeklyTurns
    );
    assert_eq!(
        classify_task_turn(Some("2026-11-24 08:30"), None, TaskPriority::Low),
        TaskTurn::Morning
    );
    assert_eq!(
        classify_task_turn(None, Some("2026-11-24T14:00"), TaskPriority::High),
        TaskTurn::Afternoon
    );
    assert_eq!(
        classify_task_turn(Some("20:00"), None, TaskPriority::Medium),
        TaskTurn::Evening
    );
    assert_eq!(
        classify_task_turn(None, None, TaskPriority::Urgent),
        TaskTurn::Morning
    );
    assert_eq!(
        classify_task_turn(None, None, TaskPriority::Medium),
        TaskTurn::Afternoon
    );
    assert_eq!(
        classify_task_turn(None, None, TaskPriority::Low),
        TaskTurn::Evening
    );
}
