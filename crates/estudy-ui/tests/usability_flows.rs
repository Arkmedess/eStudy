//! Testes de usabilidade e fluxos de interação direta do eStudy (Design Vídeo 1 & Vídeo 2).

use std::sync::Arc;

use chrono::{Duration, TimeZone, Utc};
use estudy_domain::{FixedClock, NoteKind, TaskPriority, TaskStatus, TaskTurn};
use estudy_ui::command_palette::{AppRoute, PaletteMessage};
use estudy_ui::features::calendar::CalendarMessage;
use estudy_ui::features::home::{HomeDashboardData, HomeLoadState, HomeMessage};
use estudy_ui::features::projects::{DragTarget, ProjectsMessage, TasksViewMode};
use estudy_ui::features::second_brain::SecondBrainMessage;
use estudy_ui::{AppMessage, EstudyApp};
use estudy_ui_kit::{ThemeMode, ThemePreset};
use iced::widget::text_editor;

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

/// UF-01: Tela de Tarefas abre direto na visão Semana e permite concluir (○ -> ✓) e ciclar status em 1 clique no cartão.
#[test]
fn uf01_should_open_tasks_in_week_mode_and_manipulate_card_status_inline() {
    let mut app = make_app();
    assert_eq!(app.projects_state.view_mode, TasksViewMode::Week);

    let task_id = app.projects_state.tasks[1].id.clone();
    assert_eq!(app.projects_state.tasks[1].status, TaskStatus::Todo);

    // Clicar na pílula de status avança: Todo -> InProgress -> Done -> Todo
    app.update(AppMessage::Projects(ProjectsMessage::CycleTaskStatus(
        task_id.clone(),
    )));
    assert_eq!(app.projects_state.tasks[1].status, TaskStatus::InProgress);

    // Clicar no círculo ○ conclui imediatamente; clicar de novo reabre
    app.update(AppMessage::Projects(ProjectsMessage::ToggleTaskDone(
        task_id.clone(),
    )));
    assert_eq!(app.projects_state.tasks[1].status, TaskStatus::Done);

    app.update(AppMessage::Projects(ProjectsMessage::ToggleTaskDone(
        task_id,
    )));
    assert_eq!(app.projects_state.tasks[1].status, TaskStatus::Todo);
}

/// UF-02: Drag & Drop real entre 'SEM DIA' e os períodos/dias da semana.
#[test]
fn uf02_should_drag_and_drop_task_between_unscheduled_and_day_turns() {
    let mut app = make_app();
    let today = fixed_now().date_naive();
    let target_day = today + Duration::days(1);

    // Tarefa 2 inicialmente não tem data (fica na bandeja SEM DIA)
    let task_id = app.projects_state.tasks[2].id.clone();
    assert!(app.projects_state.tasks[2].start_at.is_none());

    app.update(AppMessage::Projects(ProjectsMessage::BeginDragTask(
        task_id.clone(),
    )));
    app.update(AppMessage::Projects(ProjectsMessage::HoverDropTarget(
        Some(DragTarget::DayTurn {
            date: target_day,
            turn: TaskTurn::Afternoon,
        }),
    )));
    app.update(AppMessage::Projects(ProjectsMessage::DropTask));

    assert!(app.projects_state.dragging_task_id.is_none());
    let expected_prefix = target_day.format("%Y-%m-%d").to_string();
    assert!(
        app.projects_state.tasks[2]
            .start_at
            .as_deref()
            .is_some_and(|s| s.starts_with(&expected_prefix) && s.contains("14:00"))
    );

    // Arrastar de volta para SEM DIA limpa data e horário
    app.update(AppMessage::Projects(ProjectsMessage::BeginDragTask(
        task_id,
    )));
    app.update(AppMessage::Projects(ProjectsMessage::HoverDropTarget(
        Some(DragTarget::Unscheduled),
    )));
    app.update(AppMessage::Projects(ProjectsMessage::DropTask));
    assert!(app.projects_state.tasks[2].start_at.is_none());
    assert!(app.projects_state.tasks[2].due_at.is_none());
}

/// UF-03: Menu de Contexto no cursor (clique direito) altera prioridade, duplica e exclui com rastreio `SQLite`.
#[test]
fn uf03_should_open_context_menu_at_cursor_duplicate_set_priority_and_soft_delete() {
    let mut app = make_app();
    let task_id = app.projects_state.tasks[0].id.clone();
    let initial_len = app.projects_state.tasks.len();

    app.update(AppMessage::Projects(ProjectsMessage::CursorMoved(
        340.0, 220.0,
    )));
    app.update(AppMessage::Projects(ProjectsMessage::OpenContextMenu(
        task_id.clone(),
    )));
    assert!(app.projects_state.context_menu.is_some());
    if let Some(menu) = app.projects_state.context_menu.as_ref() {
        assert!((menu.x - 340.0).abs() < f32::EPSILON);
        assert!((menu.y - 220.0).abs() < f32::EPSILON);
    }

    app.update(AppMessage::Projects(ProjectsMessage::ContextSetPriority(
        task_id.clone(),
        TaskPriority::Low,
    )));
    assert_eq!(app.projects_state.tasks[0].priority, TaskPriority::Low);
    assert!(app.projects_state.context_menu.is_none());

    app.update(AppMessage::Projects(ProjectsMessage::DuplicateTask(
        task_id.clone(),
    )));
    assert_eq!(app.projects_state.tasks.len(), initial_len + 1);

    app.update(AppMessage::Projects(ProjectsMessage::DeleteTask(
        task_id.clone(),
    )));
    assert_eq!(app.projects_state.tasks.len(), initial_len);
    let deleted = app.projects_state.take_deleted_tasks();
    assert_eq!(deleted, vec![task_id]);
}

/// UF-04: Modal central de Tarefa suporta notas multi-linha reais (`Enter` insere `\n`), checklist e Card do Dia.
#[test]
fn uf04_should_create_task_via_multiline_modal_and_open_calendar_day_peek() {
    let mut app = make_app();
    let today = fixed_now().date_naive();
    let before = app.projects_state.tasks.len();

    app.update(AppMessage::Projects(ProjectsMessage::OpenCreateTaskModal {
        date: Some(today),
        turn: TaskTurn::Anytime,
    }));
    assert!(app.projects_state.task_modal.is_some());

    app.update(AppMessage::Projects(ProjectsMessage::ModalTitleChanged(
        "Editar o vídeo do canal".to_string(),
    )));
    app.update(AppMessage::Projects(ProjectsMessage::ModalNotesAction(
        text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new("Linha 1".to_string()))),
    )));
    app.update(AppMessage::Projects(ProjectsMessage::ModalNotesAction(
        text_editor::Action::Edit(text_editor::Edit::Enter),
    )));
    app.update(AppMessage::Projects(ProjectsMessage::ModalNotesAction(
        text_editor::Action::Edit(text_editor::Edit::Paste(Arc::new("Linha 2".to_string()))),
    )));
    app.update(AppMessage::Projects(
        ProjectsMessage::ModalChecklistInputChanged("Cortar os trechos".to_string()),
    ));
    app.update(AppMessage::Projects(ProjectsMessage::ModalAddChecklistItem));
    app.update(AppMessage::Projects(ProjectsMessage::ModalSetPriority(
        TaskPriority::High,
    )));
    app.update(AppMessage::Projects(ProjectsMessage::SaveTaskModal));

    assert!(app.projects_state.task_modal.is_none());
    assert_eq!(app.projects_state.tasks.len(), before + 1);
    assert!(app.projects_state.tasks.last().is_some());
    if let Some(created) = app.projects_state.tasks.last() {
        assert_eq!(created.title, "Editar o vídeo do canal");
        assert_eq!(created.description_md, "Linha 1\nLinha 2");
        assert_eq!(created.checklist.len(), 1);
        assert_eq!(created.priority, TaskPriority::High);
    }

    // Modo Calendário em Tarefas abre o Card do Dia ao clicar na data
    app.update(AppMessage::Projects(ProjectsMessage::OpenDayPeek(today)));
    assert_eq!(app.projects_state.day_peek_date, Some(today));
}

/// UF-05: Home navega direto ao projeto clicado (sem popup redundante) e aciona criações rápidas no Hero.
#[test]
fn uf05_should_navigate_directly_to_project_from_home_and_trigger_hero_quick_actions() {
    let mut app = make_app();
    let proj_title = app.projects_state.projects[1].title.clone();
    let proj_id = app.projects_state.projects[1].id.clone();

    app.update(AppMessage::Home(HomeMessage::PreviewProject(proj_title)));
    assert_eq!(app.route, AppRoute::Projects);
    assert_eq!(app.projects_state.selected_project_id, Some(proj_id));
    assert!(app.active_popup.is_none());

    app.update(AppMessage::Navigate(AppRoute::Home));
    app.update(AppMessage::Home(HomeMessage::QuickCreateTask));
    assert!(app.projects_state.task_modal.is_some());
    app.update(AppMessage::CloseOverlays);
    assert!(app.projects_state.task_modal.is_none());

    app.update(AppMessage::Home(HomeMessage::QuickCreateNote));
    assert_eq!(app.route, AppRoute::SecondBrain);
    assert!(app.second_brain_state.note_modal_open);
}

/// UF-06: Segundo Cérebro edita notas multi-linha reais (`Enter` insere `\n`) em modal amplo e Ajustes troca temas.
#[test]
fn uf06_should_edit_multiline_note_in_second_brain_modal_and_switch_themes() {
    let mut app = make_app();
    app.update(AppMessage::Navigate(AppRoute::SecondBrain));
    app.update(AppMessage::SecondBrain(SecondBrainMessage::CreateNote));
    assert!(app.second_brain_state.note_modal_open);

    app.update(AppMessage::SecondBrain(
        SecondBrainMessage::BodyEditorAction(text_editor::Action::Edit(text_editor::Edit::Paste(
            Arc::new("Ideia A".to_string()),
        ))),
    ));
    app.update(AppMessage::SecondBrain(
        SecondBrainMessage::BodyEditorAction(text_editor::Action::Edit(text_editor::Edit::Enter)),
    ));
    app.update(AppMessage::SecondBrain(
        SecondBrainMessage::BodyEditorAction(text_editor::Action::Edit(text_editor::Edit::Paste(
            Arc::new("Ideia B".to_string()),
        ))),
    ));

    assert!(app.second_brain_state.selected_note().is_some());
    if let Some(note) = app.second_brain_state.selected_note() {
        assert_eq!(note.body_md, "Ideia A\nIdeia B");
    }

    app.update(AppMessage::Navigate(AppRoute::Settings));
    app.update(AppMessage::SelectThemePreset(ThemePreset::VelvetSakura));
    app.update(AppMessage::ToggleThemeMode);
    assert_eq!(app.loadout.theme_preset, ThemePreset::VelvetSakura);
    assert_eq!(app.loadout.theme_mode, ThemeMode::Dark);
}

/// UF-07: Pilha previsível de Esc fecha exatamente uma camada por vez (`Palette` -> `ContextMenu` -> `TaskModal` -> `DayPeek`).
#[test]
fn uf07_should_close_interactive_layers_one_at_a_time_on_escape() {
    let mut app = make_app();
    let today = fixed_now().date_naive();
    let task_id = app.projects_state.tasks[0].id.clone();

    app.update(AppMessage::Projects(ProjectsMessage::OpenDayPeek(today)));
    app.update(AppMessage::Projects(ProjectsMessage::OpenCreateTaskModal {
        date: Some(today),
        turn: TaskTurn::Morning,
    }));
    app.update(AppMessage::Projects(ProjectsMessage::OpenContextMenu(
        task_id,
    )));
    app.update(AppMessage::Palette(PaletteMessage::Toggle));

    assert!(app.palette.is_open);
    assert!(app.projects_state.context_menu.is_some());
    assert!(app.projects_state.task_modal.is_some());
    assert!(app.projects_state.day_peek_date.is_some());

    app.update(AppMessage::CloseOverlays);
    assert!(!app.palette.is_open);
    assert!(app.projects_state.context_menu.is_some());

    app.update(AppMessage::CloseOverlays);
    assert!(app.projects_state.context_menu.is_none());
    assert!(app.projects_state.task_modal.is_some());

    app.update(AppMessage::CloseOverlays);
    assert!(app.projects_state.task_modal.is_none());
    assert!(app.projects_state.day_peek_date.is_some());

    app.update(AppMessage::CloseOverlays);
    assert!(app.projects_state.day_peek_date.is_none());
}

/// UF-08: Criar novo projeto na UI, filtrar tarefas por ele e rastrear dirty para `SQLite`.
#[test]
fn uf08_should_create_new_project_filter_by_it_and_mark_dirty() {
    let mut app = make_app();
    let _ = app.projects_state.take_dirty();
    let initial_projects = app.projects_state.projects.len();

    app.update(AppMessage::Navigate(AppRoute::Projects));
    app.update(AppMessage::Projects(
        ProjectsMessage::ToggleCreateProjectModal,
    ));
    assert!(app.projects_state.create_project_modal_open);

    app.update(AppMessage::Projects(
        ProjectsMessage::NewProjectTitleChanged("Pesquisa Rust".to_string()),
    ));
    app.update(AppMessage::Projects(
        ProjectsMessage::NewProjectColorSelected(4),
    ));
    app.update(AppMessage::Projects(ProjectsMessage::CreateProject));

    assert!(!app.projects_state.create_project_modal_open);
    assert_eq!(app.projects_state.projects.len(), initial_projects + 1);
    let created = app.projects_state.projects.last();
    assert!(created.is_some());
    let Some(created) = created else {
        return;
    };
    assert_eq!(created.title, "Pesquisa Rust");
    let created_id = created.id.clone();
    assert_eq!(
        app.projects_state.selected_project_id,
        Some(created_id.clone())
    );

    let (dirty_projects, _) = app.projects_state.take_dirty();
    assert_eq!(dirty_projects, vec![created_id]);
}

/// UF-09: Clicar em dia na Agenda abre o Card do Dia, edita/exclui evento e Desfazer restaura o evento.
#[test]
fn uf09_should_open_agenda_day_peek_edit_delete_and_undo_event() {
    let mut app = make_app();
    let today = fixed_now().date_naive();
    app.update(AppMessage::Navigate(AppRoute::Calendar));

    app.update(AppMessage::Calendar(CalendarMessage::OpenDayPeek(today)));
    assert!(app.calendar_state.day_peek_open);
    assert_eq!(app.calendar_state.cursor_date, today);

    let event_id = app.calendar_state.events[0].id.clone();
    app.update(AppMessage::Calendar(CalendarMessage::EditEvent(
        event_id.clone(),
    )));
    assert!(app.calendar_state.create_draft.open);
    app.update(AppMessage::Calendar(CalendarMessage::NewTitleChanged(
        "Revisão Atualizada".to_string(),
    )));
    app.update(AppMessage::Calendar(CalendarMessage::CreateEvent));
    assert!(!app.calendar_state.create_draft.open);
    assert_eq!(app.calendar_state.events[0].title, "Revisão Atualizada");

    let count_before = app.calendar_state.events.len();
    app.update(AppMessage::Calendar(CalendarMessage::DeleteEvent(event_id)));
    assert_eq!(app.calendar_state.events.len(), count_before - 1);
    assert!(app.undo_action.is_some());

    app.update(AppMessage::UndoLastAction);
    assert_eq!(app.calendar_state.events.len(), count_before);
    assert!(app.undo_action.is_none());
}

/// UF-10: Excluir tarefa permite Desfazer e Drag & Drop respeita threshold de 6px sem bloquear clique.
#[test]
fn uf10_should_undo_deleted_task_and_respect_drag_threshold() {
    let mut app = make_app();
    let task_id = app.projects_state.tasks[0].id.clone();
    let initial_len = app.projects_state.tasks.len();

    // Movimento < 6px não dispara drag
    app.update(AppMessage::Projects(ProjectsMessage::CursorMoved(
        100.0, 100.0,
    )));
    app.update(AppMessage::Projects(ProjectsMessage::PointerDownTask(
        task_id.clone(),
    )));
    app.update(AppMessage::Projects(ProjectsMessage::CursorMoved(
        103.0, 102.0,
    )));
    assert!(app.projects_state.dragging_task_id.is_none());

    // Movimento >= 6px ativa drag real
    app.update(AppMessage::Projects(ProjectsMessage::CursorMoved(
        112.0, 108.0,
    )));
    assert_eq!(app.projects_state.dragging_task_id, Some(task_id.clone()));
    app.update(AppMessage::Projects(ProjectsMessage::DropTask));
    assert!(app.projects_state.dragging_task_id.is_none());
    assert!(app.projects_state.pending_drag_task.is_none());

    // Excluir tarefa e restaurar via Desfazer
    app.update(AppMessage::Projects(ProjectsMessage::DeleteTask(
        task_id.clone(),
    )));
    assert_eq!(app.projects_state.tasks.len(), initial_len - 1);
    assert!(app.undo_action.is_some());

    app.update(AppMessage::UndoLastAction);
    assert_eq!(app.projects_state.tasks.len(), initial_len);
    assert!(app.projects_state.tasks.iter().any(|t| t.id == task_id));
}

/// UF-11: Segundo Cérebro remove tag, edita URL de Link e Ctrl+K busca notas e hábitos.
#[test]
fn uf11_should_manage_tags_and_url_in_second_brain_and_search_in_palette() {
    let mut app = make_app();
    app.update(AppMessage::Navigate(AppRoute::SecondBrain));

    let link_note_opt = app
        .second_brain_state
        .notes
        .iter()
        .find(|n| n.kind == NoteKind::Link)
        .map(|n| n.id.clone());
    assert!(link_note_opt.is_some());
    let Some(link_note_id) = link_note_opt else {
        return;
    };

    app.update(AppMessage::SecondBrain(SecondBrainMessage::SelectNote(
        link_note_id,
    )));
    app.update(AppMessage::SecondBrain(
        SecondBrainMessage::SourceUrlChanged("https://iced.rs".to_string()),
    ));
    app.update(AppMessage::SecondBrain(SecondBrainMessage::RemoveTag(
        "iced".to_string(),
    )));

    let selected = app.second_brain_state.selected_note();
    assert!(selected.is_some());
    let Some(selected) = selected else {
        return;
    };
    assert_eq!(selected.source_url.as_deref(), Some("https://iced.rs"));
    assert!(!selected.tags.iter().any(|t| t == "iced"));

    // Ctrl+K indexa notas e hábitos
    app.update(AppMessage::Palette(PaletteMessage::Toggle));
    app.update(AppMessage::Palette(PaletteMessage::QueryChanged(
        "Revisão Espaçada".to_string(),
    )));
    let hits = app.palette.filtered_commands();
    assert!(hits.iter().any(|c| c.id.starts_with("note:")));
    assert!(hits.iter().any(|c| c.id.starts_with("habit:")));
}

/// UF-12: Redimensionar janela atualiza layout responsivo, editar nome atualiza Home e Esc fecha camadas.
#[test]
fn uf12_should_track_window_resize_update_profile_name_and_close_new_layers_with_esc() {
    let mut app = make_app();
    app.update(AppMessage::WindowResized(948.0, 1064.0));
    assert!((app.window_size.0 - 948.0).abs() < f32::EPSILON);

    app.update(AppMessage::DisplayNameChanged("Karolayne".to_string()));
    app.update(AppMessage::SaveDisplayName);
    if let HomeLoadState::Ready(data) = &app.home_state {
        assert_eq!(data.profile.display_name, "Karolayne");
    }

    app.update(AppMessage::Projects(
        ProjectsMessage::ToggleCreateProjectModal,
    ));
    app.update(AppMessage::Calendar(CalendarMessage::OpenDayPeek(
        fixed_now().date_naive(),
    )));
    assert!(app.projects_state.create_project_modal_open);
    assert!(app.calendar_state.day_peek_open);

    app.update(AppMessage::CloseOverlays);
    assert!(!app.projects_state.create_project_modal_open);
    assert!(app.calendar_state.day_peek_open);

    app.update(AppMessage::CloseOverlays);
    assert!(!app.calendar_state.day_peek_open);
}
