//! Testes comportamentais MVU para Projetos (Kanban + Frameworks) e Drawer da Tarefa.

#![allow(clippy::expect_used)]

use chrono::DateTime;
use estudy_domain::{
    FixedClock, MAX_FOCUS_PINNED_TASKS, MAX_TASK_CHECKLIST_ITEMS, ProjectFramework, TaskStatus,
};
use estudy_ui::{
    AppMessage, AppRoute, DescriptionTab, EstudyApp, HomeDashboardData, HomeLoadState,
    ProjectsMessage,
};
use std::sync::Arc;

fn make_projects_app() -> EstudyApp {
    let now = DateTime::from_timestamp(1_760_000_000, 0).expect("timestamp válido");
    let clock = Arc::new(FixedClock::new(now));
    let mut app = EstudyApp::new(
        clock,
        HomeLoadState::Ready(Box::new(HomeDashboardData::sample_workspace("Arthur"))),
    );
    app.update(AppMessage::Navigate(AppRoute::Projects));
    app
}

#[test]
fn should_toggle_context_sidebar_filter_tags_and_switch_frameworks() {
    let mut app = make_projects_app();
    assert!(app.projects_state.sidebar_visible);

    app.update(AppMessage::ToggleContextSidebar);
    assert!(!app.projects_state.sidebar_visible);

    app.update(AppMessage::Projects(ProjectsMessage::SwitchFramework(
        ProjectFramework::Eisenhower,
    )));
    assert_eq!(
        app.projects_state.active_framework(),
        ProjectFramework::Eisenhower
    );

    app.update(AppMessage::Projects(ProjectsMessage::ToggleTagFilter(
        Some("rust".to_string()),
    )));
    let filtered = app.projects_state.filtered_tasks();
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].tags.iter().any(|tag| tag == "rust"));

    app.update(AppMessage::Projects(ProjectsMessage::SelectProject(None)));
    app.update(AppMessage::Projects(ProjectsMessage::ToggleTagFilter(None)));
    assert_eq!(app.projects_state.filtered_tasks().len(), 3);
}

#[test]
fn should_move_kanban_tasks_and_edit_drawer_checklist_markdown_and_multi_project() {
    let mut app = make_projects_app();
    let task_id = app.projects_state.tasks[1].id.clone();

    app.update(AppMessage::Projects(ProjectsMessage::MoveTaskStatus(
        task_id.clone(),
        TaskStatus::Done,
    )));
    assert_eq!(app.projects_state.tasks[1].status, TaskStatus::Done);

    app.update(AppMessage::Projects(ProjectsMessage::OpenTaskDrawer(
        task_id.clone(),
    )));
    assert!(app.projects_state.drawer_task().is_some());

    app.update(AppMessage::Projects(ProjectsMessage::SetDescriptionTab(
        DescriptionTab::Preview,
    )));
    assert_eq!(
        app.projects_state
            .drawer
            .as_ref()
            .map(|drawer| drawer.description_tab),
        Some(DescriptionTab::Preview)
    );

    let extra_project_id = app.projects_state.projects[0].id.clone();
    app.update(AppMessage::Projects(
        ProjectsMessage::DrawerToggleProjectLink(extra_project_id),
    ));
    assert_eq!(app.projects_state.tasks[1].project_links.len(), 2);

    for idx in 0..MAX_TASK_CHECKLIST_ITEMS {
        app.update(AppMessage::Projects(
            ProjectsMessage::DrawerChecklistInputChanged(format!("Item {idx}")),
        ));
        app.update(AppMessage::Projects(
            ProjectsMessage::DrawerAddChecklistItem,
        ));
    }
    assert_eq!(
        app.projects_state.tasks[1].checklist.len(),
        MAX_TASK_CHECKLIST_ITEMS
    );

    app.update(AppMessage::Projects(
        ProjectsMessage::DrawerChecklistInputChanged("Item 21".to_string()),
    ));
    app.update(AppMessage::Projects(
        ProjectsMessage::DrawerAddChecklistItem,
    ));
    assert_eq!(
        app.projects_state.tasks[1].checklist.len(),
        MAX_TASK_CHECKLIST_ITEMS
    );
    assert!(
        app.projects_state
            .drawer
            .as_ref()
            .and_then(|drawer| drawer.notice.as_deref())
            .is_some_and(|msg| msg.contains("20"))
    );

    app.update(AppMessage::CloseOverlays);
    assert!(app.projects_state.drawer.is_none());
}

#[test]
fn should_prevent_more_than_5_focus_pins_in_projects_drawer() {
    let mut app = make_projects_app();
    for idx in 1..=MAX_FOCUS_PINNED_TASKS {
        app.update(AppMessage::Projects(
            ProjectsMessage::QuickTaskTitleChanged(format!("Extra {idx}")),
        ));
        app.update(AppMessage::Projects(ProjectsMessage::CreateQuickTask));
    }

    let mut pinned = 0;
    let task_ids: Vec<_> = app
        .projects_state
        .tasks
        .iter()
        .filter(|task| !task.pinned_focus)
        .map(|task| task.id.clone())
        .collect();

    for id in task_ids {
        app.update(AppMessage::Projects(ProjectsMessage::OpenTaskDrawer(id)));
        app.update(AppMessage::Projects(ProjectsMessage::DrawerToggleFocusPin));
        if app
            .projects_state
            .drawer_task()
            .is_some_and(|t| t.pinned_focus)
        {
            pinned += 1;
        }
    }

    assert_eq!(
        app.projects_state
            .tasks
            .iter()
            .filter(|task| task.pinned_focus)
            .count(),
        MAX_FOCUS_PINNED_TASKS
    );
    assert_eq!(pinned, MAX_FOCUS_PINNED_TASKS - 1);
}

#[test]
fn should_avoid_dirty_writes_during_typing_and_flush_only_changed_task_on_close() {
    let mut app = make_projects_app();
    let _ = app.projects_state.take_dirty();

    let task_id = app.projects_state.tasks[0].id.clone();
    app.update(AppMessage::Projects(ProjectsMessage::OpenTaskDrawer(
        task_id.clone(),
    )));
    let (p0, t0) = app.projects_state.take_dirty();
    assert!(p0.is_empty() && t0.is_empty());

    for step in 0..50 {
        app.update(AppMessage::Projects(ProjectsMessage::DrawerTitleChanged(
            format!("Título em edição {step}"),
        )));
        app.update(AppMessage::Projects(
            ProjectsMessage::DrawerDescriptionChanged(format!("Descrição {step}")),
        ));
        let (p_mid, t_mid) = app.projects_state.take_dirty();
        assert!(
            p_mid.is_empty() && t_mid.is_empty(),
            "digitação contínua não deve disparar escrita imediata"
        );
    }

    app.update(AppMessage::Projects(ProjectsMessage::CloseTaskDrawer));
    let (p_end, t_end) = app.projects_state.take_dirty();
    assert!(p_end.is_empty());
    assert_eq!(t_end, vec![task_id]);
}
