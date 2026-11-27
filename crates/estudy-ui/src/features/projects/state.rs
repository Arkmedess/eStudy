//! Estado e transições da fatia Tarefas & Projetos (`Semana | Calendário | Kanban`, Drag & Drop, Menu de Contexto e Modais).

use chrono::{Datelike, Duration, NaiveDate};
use estudy_domain::{
    EisenhowerQuadrant, EntityId, MAX_FOCUS_PINNED_TASKS, MAX_TASK_CHECKLIST_ITEMS, Project,
    ProjectFramework, ProjectStatus, Task, TaskChecklistItem, TaskPriority, TaskProjectLink,
    TaskStatus, TaskTurn, classify_task_turn, validate_focus_pin_budget,
};
use iced::widget::text_editor;
use std::collections::BTreeSet;

/// Modo principal da tela `Tarefas` (`Semana | Calendário | Kanban`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TasksViewMode {
    /// Visão semanal fluida com bandeja `SEM DIA` e 4 períodos por dia (padrão).
    Week,
    /// Grade mensal de tarefas com abertura do Card do Dia ao clicar.
    Calendar,
    /// Quadro Kanban em 4 colunas.
    Kanban,
}

impl TasksViewMode {
    /// Opções exibidas no seletor segmentado principal de Tarefas.
    pub const ALL: [(Self, &'static str); 3] = [
        (Self::Week, "Semana"),
        (Self::Calendar, "Calendário"),
        (Self::Kanban, "Kanban"),
    ];
}

/// Alvo de soltura para o Drag & Drop de cartões de tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragTarget {
    /// Bandeja horizontal `SEM DIA` (remove data e horário).
    Unscheduled,
    /// Coluna de um dia e período específico (`Qualquer hora / Manhã / Tarde / Noite`).
    DayTurn {
        /// Data alvo.
        date: NaiveDate,
        /// Período do dia.
        turn: TaskTurn,
    },
    /// Coluna de status no modo Kanban.
    KanbanStatus(TaskStatus),
}

/// Submenu aberto no menu de contexto de clique direito.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextSubmenu {
    /// Escolha rápida de status.
    Status,
    /// Escolha rápida de prioridade.
    Priority,
    /// Escolha rápida de período do dia.
    Turn,
    /// Escolha rápida de data.
    Day,
}

/// Menu de contexto flutuante ancorado na posição `(x, y)` do cursor.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskContextMenu {
    /// Tarefa alvo do clique direito.
    pub task_id: EntityId,
    /// Coordenada horizontal na janela.
    pub x: f32,
    /// Coordenada vertical na janela.
    pub y: f32,
    /// Submenu expandido lateralmente, se houver.
    pub submenu: Option<ContextSubmenu>,
}

/// Modo de operação do modal central de tarefa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskModalMode {
    /// Criação de uma nova tarefa.
    Create,
    /// Edição de uma tarefa existente.
    Edit(EntityId),
}

/// Estado do modal central estruturado (`Nova tarefa` / `Editar tarefa`) com `text_editor` multi-linha real.
#[derive(Debug)]
pub struct TaskModalState {
    /// Criação ou edição.
    pub mode: TaskModalMode,
    /// Título editável.
    pub title: String,
    /// Conteúdo multi-linha real (`Enter` insere quebra de linha).
    pub notes_content: text_editor::Content,
    /// Itens da checklist em edição.
    pub checklist: Vec<TaskChecklistItem>,
    /// Campo temporário para novo item da checklist.
    pub new_checklist_title: String,
    /// Status selecionado.
    pub status: TaskStatus,
    /// Prioridade selecionada.
    pub priority: TaskPriority,
    /// Se `true`, agenda em `date` + `turn`; se `false`, fica em `SEM DIA`.
    pub has_date: bool,
    /// Data selecionada.
    pub date: NaiveDate,
    /// Período do dia selecionado.
    pub turn: TaskTurn,
    /// Projeto vinculado principal (`None` = Nenhum).
    pub project_id: Option<EntityId>,
    /// Aviso de validação opcional.
    pub notice: Option<String>,
}

impl Clone for TaskModalState {
    fn clone(&self) -> Self {
        Self {
            mode: self.mode.clone(),
            title: self.title.clone(),
            notes_content: text_editor::Content::with_text(&self.notes_content.text()),
            checklist: self.checklist.clone(),
            new_checklist_title: self.new_checklist_title.clone(),
            status: self.status,
            priority: self.priority,
            has_date: self.has_date,
            date: self.date,
            turn: self.turn,
            project_id: self.project_id.clone(),
            notice: self.notice.clone(),
        }
    }
}

/// Evento próximo exibido como contexto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpcomingEventItem {
    /// Horário ou data resumida (ex.: `Hoje · 18:00`).
    pub when_label: String,
    /// Título do evento.
    pub title: String,
    /// Projeto relacionado, se houver.
    pub project_title: Option<String>,
}

/// Modo do editor de descrição Markdown legado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptionTab {
    /// Campo de escrita em Markdown.
    Write,
    /// Visualização formatada do Markdown.
    Preview,
}

/// Estado de edição direta por ID de tarefa (mantido para compatibilidade de contratos MVU).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDrawerState {
    /// Identificador da tarefa aberta.
    pub task_id: EntityId,
    /// Aba ativa da descrição (`Escrever` ou `Visualizar`).
    pub description_tab: DescriptionTab,
    /// Texto temporário para adicionar item na checklist (`0/20`).
    pub new_checklist_title: String,
    /// Texto temporário para adicionar uma tag.
    pub new_tag_name: String,
    /// Mensagem de feedback ou erro de regra.
    pub notice: Option<String>,
}

/// Estado completo da fatia Tarefas & Projetos.
#[derive(Debug, Clone)]
pub struct ProjectsState {
    /// Projetos ativos.
    pub projects: Vec<Project>,
    /// Tarefas ativas.
    pub tasks: Vec<Task>,
    /// Próximos eventos.
    pub upcoming_events: Vec<UpcomingEventItem>,
    /// Projeto atualmente filtrado (`None` exibe todos).
    pub selected_project_id: Option<EntityId>,
    /// Filtro rápido por tag (`None` exibe todas).
    pub active_tag_filter: Option<String>,
    /// Flag de visibilidade contextual legada.
    pub sidebar_visible: bool,
    /// Ano exibido no calendário mensal de tarefas.
    pub calendar_year: i32,
    /// Mês exibido no calendário mensal de tarefas (`1..=12`).
    pub calendar_month: u32,
    /// Dia de referência selecionado.
    pub selected_day: NaiveDate,
    /// Texto rápido para criar nova tarefa.
    pub quick_task_title: String,
    /// Conjunto de tarefas selecionadas para Foco Semanal.
    pub weekly_focus_ids: BTreeSet<EntityId>,
    /// Estado de edição contextual por tarefa.
    pub drawer: Option<TaskDrawerState>,
    /// Modo de visualização principal (`Semana | Calendário | Kanban`).
    pub view_mode: TasksViewMode,
    /// Segunda-feira de referência da semana exibida.
    pub week_start: NaiveDate,
    /// Tarefa atualmente sendo arrastada no Drag & Drop.
    pub dragging_task_id: Option<EntityId>,
    /// Zona de soltura atualmente destacada sob o cursor durante o arraste.
    pub hovered_drop_target: Option<DragTarget>,
    /// Última coordenada conhecida do cursor `(x, y)` na janela.
    pub cursor_pos: (f32, f32),
    /// Menu de contexto aberto no clique direito sobre um cartão.
    pub context_menu: Option<TaskContextMenu>,
    /// Modal central estruturado de criação/edição de tarefa.
    pub task_modal: Option<TaskModalState>,
    /// Card do Dia aberto ao clicar numa célula do modo Calendário.
    pub day_peek_date: Option<NaiveDate>,
    dirty_project_ids: BTreeSet<EntityId>,
    dirty_task_ids: BTreeSet<EntityId>,
    deleted_task_ids: BTreeSet<EntityId>,
    deferred_task_ids: BTreeSet<EntityId>,
}

/// Mensagens da fatia Projetos / Tarefas.
#[derive(Debug, Clone)]
pub enum ProjectsMessage {
    /// Alterna entre `Semana | Calendário | Kanban`.
    SetTasksViewMode(TasksViewMode),
    /// Avança ou retrocede a semana exibida (`-1` ou `+1`).
    ShiftWeek(i64),
    /// Conclui ou reabre uma tarefa em 1 clique direto no cartão (`○ -> ✓`).
    ToggleTaskDone(EntityId),
    /// Cicla o status direto na pílula do cartão (`A fazer -> Em andamento -> Concluído`).
    CycleTaskStatus(EntityId),
    /// Inicia o arraste de um cartão de tarefa.
    BeginDragTask(EntityId),
    /// Atualiza a zona de soltura sob o cursor durante o arraste.
    HoverDropTarget(Option<DragTarget>),
    /// Solta a tarefa arrastada na zona destacada atual.
    DropTask,
    /// Cancela o arraste ativo sem mover a tarefa.
    CancelDrag,
    /// Atualiza a posição do cursor na janela `(x, y)`.
    CursorMoved(f32, f32),
    /// Abre o menu de contexto na posição atual do cursor para a tarefa.
    OpenContextMenu(EntityId),
    /// Fecha o menu de contexto.
    CloseContextMenu,
    /// Abre um submenu lateral dentro do menu de contexto.
    SetContextSubmenu(Option<ContextSubmenu>),
    /// Altera prioridade diretamente pelo menu de contexto.
    ContextSetPriority(EntityId, TaskPriority),
    /// Altera período do dia diretamente pelo menu de contexto.
    ContextSetTurn(EntityId, TaskTurn),
    /// Reagenda rapidamente pelo menu de contexto (`None` = Sem dia, `Some(dias)` a partir de `selected_day`).
    ContextSetDayOffset(EntityId, Option<i64>),
    /// Duplica uma tarefa existente.
    DuplicateTask(EntityId),
    /// Exclui (soft-delete) uma tarefa existente.
    DeleteTask(EntityId),
    /// Abre o modal central para criar nova tarefa (com data e período opcionais pré-preenchidos).
    OpenCreateTaskModal {
        /// Data inicial sugerida (`None` = Sem dia).
        date: Option<NaiveDate>,
        /// Período inicial sugerido.
        turn: TaskTurn,
    },
    /// Abre o modal central para editar uma tarefa existente.
    OpenEditTaskModal(EntityId),
    /// Fecha o modal central sem salvar alterações pendentes de criação.
    CloseTaskModal,
    /// Confirma e salva os dados do modal central de tarefa.
    SaveTaskModal,
    /// Atualiza o título no modal central.
    ModalTitleChanged(String),
    /// Processa ação multi-linha (`Enter`, digitação, seleção, colar) no `text_editor` de notas.
    ModalNotesAction(text_editor::Action),
    /// Atualiza o input de novo item da checklist no modal.
    ModalChecklistInputChanged(String),
    /// Adiciona novo item à checklist no modal.
    ModalAddChecklistItem,
    /// Alterna conclusão de item da checklist no modal.
    ModalToggleChecklistItem(EntityId),
    /// Remove item da checklist no modal.
    ModalRemoveChecklistItem(EntityId),
    /// Altera status no modal.
    ModalSetStatus(TaskStatus),
    /// Altera prioridade no modal.
    ModalSetPriority(TaskPriority),
    /// Liga/desliga agendamento com data (`Tem dia`).
    ModalToggleHasDate,
    /// Define a data agendada no modal.
    ModalSetDate(NaiveDate),
    /// Define o período agendado no modal.
    ModalSetTurn(TaskTurn),
    /// Define o projeto vinculado no modal.
    ModalSetProject(Option<EntityId>),
    /// Abre o Card do Dia ao clicar em uma data do calendário mensal.
    OpenDayPeek(NaiveDate),
    /// Fecha o Card do Dia.
    CloseDayPeek,
    /// Recolhe ou expande a barra contextual legada.
    ToggleSidebar,
    /// Seleciona um projeto (`None` mostra visão agregada).
    SelectProject(Option<EntityId>),
    /// Altera o framework opcional do projeto selecionado.
    SwitchFramework(ProjectFramework),
    /// Filtra tarefas por tag.
    ToggleTagFilter(Option<String>),
    /// Navega para o mês anterior ou próximo no calendário.
    ShiftCalendarMonth(i32),
    /// Seleciona um dia específico no calendário.
    SelectCalendarDay(NaiveDate),
    /// Atualiza o título da barra de criação rápida de tarefa.
    QuickTaskTitleChanged(String),
    /// Cria nova tarefa no projeto/coluna padrão.
    CreateQuickTask,
    /// Cria nova tarefa diretamente em um dia e turno específicos.
    CreateQuickTaskInTurn {
        /// Data alvo no calendário.
        date: NaiveDate,
        /// Turno selecionado.
        turn: TaskTurn,
    },
    /// Inicia um bloco Pomodoro imediatamente vinculado a esta tarefa.
    StartFocusForTask(EntityId),
    /// Move uma tarefa para outra coluna de status.
    MoveTaskStatus(EntityId, TaskStatus),
    /// Move uma tarefa para um quadrante na Matriz de Eisenhower.
    SetTaskQuadrant(EntityId, EisenhowerQuadrant),
    /// Inclui ou remove uma tarefa das metas de Foco Semanal.
    ToggleWeeklyFocus(EntityId),
    /// Abre a edição por ID da tarefa (e sincroniza o modal central).
    OpenTaskDrawer(EntityId),
    /// Fecha a edição da tarefa.
    CloseTaskDrawer,
    /// Alterna aba `Escrever / Visualizar` na descrição Markdown.
    SetDescriptionTab(DescriptionTab),
    /// Atualiza título da tarefa em edição.
    DrawerTitleChanged(String),
    /// Atualiza descrição Markdown da tarefa em edição.
    DrawerDescriptionChanged(String),
    /// Atualiza horário de início da tarefa em edição.
    DrawerStartAtChanged(String),
    /// Atualiza prazo da tarefa em edição.
    DrawerDueAtChanged(String),
    /// Seleciona recorrência da tarefa em edição.
    DrawerRecurrenceChanged(Option<String>),
    /// Alterna prioridade da tarefa em edição.
    DrawerPriorityChanged(TaskPriority),
    /// Alterna fixação em "Em foco" respeitando o limite de 5.
    DrawerToggleFocusPin,
    /// Vincula ou desvincula a tarefa de um projeto.
    DrawerToggleProjectLink(EntityId),
    /// Atualiza texto do novo item de checklist.
    DrawerChecklistInputChanged(String),
    /// Adiciona item à checklist (`0/20`).
    DrawerAddChecklistItem,
    /// Marca ou desmarca um item da checklist.
    DrawerToggleChecklistItem(EntityId),
    /// Remove um item da checklist.
    DrawerRemoveChecklistItem(EntityId),
    /// Atualiza texto da nova tag.
    DrawerTagInputChanged(String),
    /// Adiciona tag à tarefa aberta.
    DrawerAddTag,
    /// Remove tag da tarefa aberta.
    DrawerRemoveTag(String),
}

impl ProjectsState {
    /// Cria um estado inicial de demonstração alinhado à data atual.
    #[must_use]
    pub fn sample(today: NaiveDate, now_utc: chrono::DateTime<chrono::Utc>) -> Self {
        let proj_estudy = EntityId::new_v7();
        let proj_semestre = EntityId::new_v7();

        let projects = vec![
            Project {
                id: proj_estudy.clone(),
                title: "eStudy Desktop".to_string(),
                description: "Aplicativo local-first em Rust e Iced".to_string(),
                color_token: "accent".to_string(),
                status: ProjectStatus::InProgress,
                framework: ProjectFramework::WeeklyTurns,
                sort_order: 0,
                created_at: now_utc,
                updated_at: now_utc,
            },
            Project {
                id: proj_semestre.clone(),
                title: "Semestre Engenharia".to_string(),
                description: "Provas, listas e entregas da faculdade".to_string(),
                color_token: "warning".to_string(),
                status: ProjectStatus::InProgress,
                framework: ProjectFramework::Milestones,
                sort_order: 1,
                created_at: now_utc,
                updated_at: now_utc,
            },
        ];

        let tasks = sample_tasks(&proj_estudy, &proj_semestre, today, now_utc);
        let mut weekly_focus_ids = BTreeSet::new();
        if let Some(first) = tasks.first() {
            weekly_focus_ids.insert(first.id.clone());
        }
        let monday_offset = i64::from(today.weekday().num_days_from_monday());
        let week_start = today - Duration::days(monday_offset);

        Self {
            projects,
            tasks,
            upcoming_events: vec![
                UpcomingEventItem {
                    when_label: "Hoje · 19:00".to_string(),
                    title: "Bloco de revisão Cálculo".to_string(),
                    project_title: Some("Semestre Engenharia".to_string()),
                },
                UpcomingEventItem {
                    when_label: "Amanhã · 10:00".to_string(),
                    title: "Revisar marcos da entrega".to_string(),
                    project_title: Some("eStudy Desktop".to_string()),
                },
            ],
            selected_project_id: None,
            active_tag_filter: None,
            sidebar_visible: true,
            calendar_year: today.year(),
            calendar_month: today.month(),
            selected_day: today,
            quick_task_title: String::new(),
            weekly_focus_ids,
            drawer: None,
            view_mode: TasksViewMode::Week,
            week_start,
            dragging_task_id: None,
            hovered_drop_target: None,
            cursor_pos: (260.0, 200.0),
            context_menu: None,
            task_modal: None,
            day_peek_date: None,
            dirty_project_ids: BTreeSet::new(),
            dirty_task_ids: BTreeSet::new(),
            deleted_task_ids: BTreeSet::new(),
            deferred_task_ids: BTreeSet::new(),
        }
    }

    /// Informa se há arraste ou menu contextual precisando de eventos de mouse da janela.
    #[must_use]
    pub const fn needs_pointer_tracking(&self) -> bool {
        self.dragging_task_id.is_some() || self.context_menu.is_some()
    }

    /// Promove edições em rascunho no Drawer para gravação pendente.
    pub fn flush_deferred(&mut self) {
        self.dirty_task_ids.append(&mut self.deferred_task_ids);
    }

    /// Extrai e limpa os IDs de projetos e tarefas que precisam ser persistidos.
    pub fn take_dirty(&mut self) -> (Vec<EntityId>, Vec<EntityId>) {
        let projects = std::mem::take(&mut self.dirty_project_ids)
            .into_iter()
            .collect();
        let tasks = std::mem::take(&mut self.dirty_task_ids)
            .into_iter()
            .collect();
        (projects, tasks)
    }

    /// Extrai e limpa os IDs de tarefas removidas (soft-delete) para persistência no `SQLite`.
    pub fn take_deleted_tasks(&mut self) -> Vec<EntityId> {
        std::mem::take(&mut self.deleted_task_ids)
            .into_iter()
            .collect()
    }

    /// Marca os projetos e tarefas atuais como pendentes para gravação inicial (seed).
    pub fn mark_all_dirty(&mut self) {
        for proj in &self.projects {
            self.dirty_project_ids.insert(proj.id.clone());
        }
        for task in &self.tasks {
            self.dirty_task_ids.insert(task.id.clone());
        }
    }

    /// Retorna o projeto ativo atualmente (ou o primeiro da lista).
    #[must_use]
    pub fn active_project(&self) -> Option<&Project> {
        match &self.selected_project_id {
            Some(id) => self.projects.iter().find(|project| project.id == *id),
            None => self.projects.first(),
        }
    }

    /// Framework ativo para visualização de projeto.
    #[must_use]
    pub fn active_framework(&self) -> ProjectFramework {
        self.active_project()
            .map_or(ProjectFramework::WeeklyTurns, |project| project.framework)
    }

    /// Lista todas as tags únicas ordenadas alfabeticamente.
    #[must_use]
    pub fn available_tags(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        for task in &self.tasks {
            for tag in &task.tags {
                set.insert(tag.clone());
            }
        }
        set.into_iter().collect()
    }

    /// Retorna as tarefas filtradas por projeto e tag selecionados.
    #[must_use]
    pub fn filtered_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| {
                let project_ok = match &self.selected_project_id {
                    Some(proj_id) => task
                        .project_links
                        .iter()
                        .any(|link| link.project_id == *proj_id),
                    None => true,
                };
                let tag_ok = match &self.active_tag_filter {
                    Some(tag) => task
                        .tags
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(tag)),
                    None => true,
                };
                project_ok && tag_ok
            })
            .collect()
    }

    /// Retorna a tarefa aberta atualmente no Drawer legado.
    #[must_use]
    pub fn drawer_task(&self) -> Option<&Task> {
        let drawer = self.drawer.as_ref()?;
        self.tasks.iter().find(|task| task.id == drawer.task_id)
    }

    /// Retorna o título do primeiro projeto vinculado à tarefa, se houver.
    #[must_use]
    pub fn task_project_info(&self, task: &Task) -> Option<(usize, &Project)> {
        let link = task.project_links.first()?;
        self.projects
            .iter()
            .enumerate()
            .find(|(_, p)| p.id == link.project_id)
    }

    /// Processa mensagens da fatia Tarefas & Projetos.
    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: ProjectsMessage, now_utc: chrono::DateTime<chrono::Utc>) {
        match message {
            ProjectsMessage::SetTasksViewMode(mode) => {
                self.view_mode = mode;
                self.context_menu = None;
            }
            ProjectsMessage::ShiftWeek(delta_weeks) => {
                self.week_start += Duration::weeks(delta_weeks);
                self.selected_day = self.week_start;
            }
            ProjectsMessage::ToggleTaskDone(id) => self.toggle_task_done(&id, now_utc),
            ProjectsMessage::CycleTaskStatus(id) => self.cycle_task_status(&id, now_utc),
            ProjectsMessage::BeginDragTask(id) => {
                self.context_menu = None;
                self.dragging_task_id = Some(id);
            }
            ProjectsMessage::HoverDropTarget(target) => {
                if self.dragging_task_id.is_some() {
                    self.hovered_drop_target = target;
                }
            }
            ProjectsMessage::DropTask => self.finish_drop(now_utc),
            ProjectsMessage::CancelDrag => {
                self.dragging_task_id = None;
                self.hovered_drop_target = None;
            }
            ProjectsMessage::CursorMoved(x, y) => {
                self.cursor_pos = (x, y);
            }
            ProjectsMessage::OpenContextMenu(task_id) => {
                let (x, y) = self.cursor_pos;
                self.dragging_task_id = None;
                self.context_menu = Some(TaskContextMenu {
                    task_id,
                    x,
                    y,
                    submenu: None,
                });
            }
            ProjectsMessage::CloseContextMenu | ProjectsMessage::StartFocusForTask(_) => {
                self.context_menu = None;
            }
            ProjectsMessage::SetContextSubmenu(submenu) => {
                if let Some(menu) = &mut self.context_menu {
                    menu.submenu = submenu;
                }
            }
            ProjectsMessage::ContextSetPriority(id, prio) => {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                    task.priority = prio;
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(id);
                }
                self.context_menu = None;
            }
            ProjectsMessage::ContextSetTurn(id, turn) => {
                let fallback_day = self.selected_day;
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                    let day = extract_task_date(task).unwrap_or(fallback_day);
                    let stamp = format_date_and_turn(day, turn);
                    task.start_at = Some(stamp.clone());
                    task.due_at = Some(stamp);
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(id);
                }
                self.context_menu = None;
            }
            ProjectsMessage::ContextSetDayOffset(id, maybe_offset) => {
                let base_day = self.selected_day;
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                    if let Some(offset) = maybe_offset {
                        let target_day = base_day + Duration::days(offset);
                        let turn = classify_existing_or_anytime(task);
                        let stamp = format_date_and_turn(target_day, turn);
                        task.start_at = Some(stamp.clone());
                        task.due_at = Some(stamp);
                    } else {
                        task.start_at = None;
                        task.due_at = None;
                    }
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(id);
                }
                self.context_menu = None;
            }
            ProjectsMessage::DuplicateTask(id) => self.duplicate_task(&id, now_utc),
            ProjectsMessage::DeleteTask(id) => self.delete_task(&id),
            ProjectsMessage::OpenCreateTaskModal { date, turn } => {
                self.context_menu = None;
                self.open_create_modal(date, turn);
            }
            ProjectsMessage::OpenEditTaskModal(id) => {
                self.context_menu = None;
                self.open_edit_modal(&id);
            }
            ProjectsMessage::CloseTaskModal => {
                self.task_modal = None;
            }
            ProjectsMessage::SaveTaskModal => self.save_task_modal(now_utc),
            ProjectsMessage::OpenDayPeek(date) => {
                self.day_peek_date = Some(date);
                self.selected_day = date;
            }
            ProjectsMessage::CloseDayPeek => {
                self.day_peek_date = None;
            }
            ProjectsMessage::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            ProjectsMessage::SelectProject(id) => self.selected_project_id = id,
            ProjectsMessage::SwitchFramework(framework) => self.switch_framework(framework),
            ProjectsMessage::ToggleTagFilter(tag) => self.active_tag_filter = tag,
            ProjectsMessage::ShiftCalendarMonth(delta) => self.shift_month(delta),
            ProjectsMessage::SelectCalendarDay(day) => self.selected_day = day,
            ProjectsMessage::QuickTaskTitleChanged(title) => self.quick_task_title = title,
            ProjectsMessage::CreateQuickTask => self.create_quick_task(now_utc),
            ProjectsMessage::CreateQuickTaskInTurn { date, turn } => {
                self.create_quick_task_in_turn(date, turn, now_utc);
            }
            ProjectsMessage::MoveTaskStatus(id, status) => {
                self.move_task_status(&id, status, now_utc);
                self.context_menu = None;
            }
            ProjectsMessage::SetTaskQuadrant(id, quadrant) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == id) {
                    task.eisenhower_quadrant = Some(quadrant);
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(id);
                }
            }
            ProjectsMessage::ToggleWeeklyFocus(id) => {
                if !self.weekly_focus_ids.remove(&id) {
                    self.weekly_focus_ids.insert(id);
                }
            }
            ProjectsMessage::OpenTaskDrawer(task_id) => {
                self.flush_deferred();
                self.open_edit_modal(&task_id);
                self.drawer = Some(TaskDrawerState {
                    task_id,
                    description_tab: DescriptionTab::Write,
                    new_checklist_title: String::new(),
                    new_tag_name: String::new(),
                    notice: None,
                });
            }
            ProjectsMessage::CloseTaskDrawer => {
                self.flush_deferred();
                self.drawer = None;
                self.task_modal = None;
            }
            other => {
                self.update_modal_or_drawer(other, now_utc);
            }
        }
    }

    fn toggle_task_done(&mut self, task_id: &EntityId, now_utc: chrono::DateTime<chrono::Utc>) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
            task.status = if task.status == TaskStatus::Done {
                TaskStatus::Todo
            } else {
                TaskStatus::Done
            };
            task.updated_at = now_utc;
            self.dirty_task_ids.insert(task_id.clone());
        }
        self.context_menu = None;
    }

    fn cycle_task_status(&mut self, task_id: &EntityId, now_utc: chrono::DateTime<chrono::Utc>) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
            task.status = match task.status {
                TaskStatus::Backlog | TaskStatus::Todo => TaskStatus::InProgress,
                TaskStatus::InProgress => TaskStatus::Done,
                TaskStatus::Done => TaskStatus::Todo,
            };
            task.updated_at = now_utc;
            self.dirty_task_ids.insert(task_id.clone());
        }
    }

    fn finish_drop(&mut self, now_utc: chrono::DateTime<chrono::Utc>) {
        let Some(task_id) = self.dragging_task_id.take() else {
            self.hovered_drop_target = None;
            return;
        };
        let Some(target) = self.hovered_drop_target.take() else {
            return;
        };
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            match target {
                DragTarget::Unscheduled => {
                    task.start_at = None;
                    task.due_at = None;
                }
                DragTarget::DayTurn { date, turn } => {
                    let stamp = format_date_and_turn(date, turn);
                    task.start_at = Some(stamp.clone());
                    task.due_at = Some(stamp);
                }
                DragTarget::KanbanStatus(status) => {
                    task.status = status;
                }
            }
            task.updated_at = now_utc;
            self.dirty_task_ids.insert(task_id);
        }
    }

    fn duplicate_task(&mut self, task_id: &EntityId, now_utc: chrono::DateTime<chrono::Utc>) {
        if let Some(source) = self.tasks.iter().find(|t| t.id == *task_id).cloned() {
            let new_id = EntityId::new_v7();
            let checklist = source
                .checklist
                .into_iter()
                .map(|mut item| {
                    item.id = EntityId::new_v7();
                    item
                })
                .collect();
            self.tasks.push(Task {
                id: new_id.clone(),
                title: format!("{} (cópia)", source.title),
                checklist,
                created_at: now_utc,
                updated_at: now_utc,
                ..source
            });
            self.dirty_task_ids.insert(new_id);
        }
        self.context_menu = None;
    }

    fn delete_task(&mut self, task_id: &EntityId) {
        let before = self.tasks.len();
        self.tasks.retain(|t| t.id != *task_id);
        if self.tasks.len() < before {
            self.dirty_task_ids.remove(task_id);
            self.deferred_task_ids.remove(task_id);
            self.deleted_task_ids.insert(task_id.clone());
        }
        if self.drawer.as_ref().is_some_and(|d| &d.task_id == task_id) {
            self.drawer = None;
        }
        self.context_menu = None;
    }

    fn open_create_modal(&mut self, date: Option<NaiveDate>, turn: TaskTurn) {
        let default_proj = self
            .selected_project_id
            .clone()
            .or_else(|| self.projects.first().map(|p| p.id.clone()));
        self.task_modal = Some(TaskModalState {
            mode: TaskModalMode::Create,
            title: String::new(),
            notes_content: text_editor::Content::new(),
            checklist: Vec::new(),
            new_checklist_title: String::new(),
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            has_date: date.is_some(),
            date: date.unwrap_or(self.selected_day),
            turn,
            project_id: default_proj,
            notice: None,
        });
    }

    fn open_edit_modal(&mut self, task_id: &EntityId) {
        let Some(task) = self.tasks.iter().find(|t| t.id == *task_id) else {
            return;
        };
        let parsed_date = extract_task_date(task);
        let turn = classify_existing_or_anytime(task);
        let project_id = task.project_links.first().map(|l| l.project_id.clone());
        self.task_modal = Some(TaskModalState {
            mode: TaskModalMode::Edit(task.id.clone()),
            title: task.title.clone(),
            notes_content: text_editor::Content::with_text(&task.description_md),
            checklist: task.checklist.clone(),
            new_checklist_title: String::new(),
            status: task.status,
            priority: task.priority,
            has_date: parsed_date.is_some(),
            date: parsed_date.unwrap_or(self.selected_day),
            turn,
            project_id,
            notice: None,
        });
    }

    fn save_task_modal(&mut self, now_utc: chrono::DateTime<chrono::Utc>) {
        let Some(modal) = self.task_modal.clone() else {
            return;
        };
        let clean_title = modal.title.trim().to_string();
        if clean_title.is_empty() {
            if let Some(m) = &mut self.task_modal {
                m.notice = Some("Digite o que precisa ser feito antes de salvar.".to_string());
            }
            return;
        }
        let stamp = modal
            .has_date
            .then(|| format_date_and_turn(modal.date, modal.turn));
        let raw_notes = modal.notes_content.text();
        let notes_md = raw_notes.trim_end_matches('\n').to_string();
        let links = modal
            .project_id
            .map(|project_id| {
                vec![TaskProjectLink {
                    project_id,
                    milestone_name: None,
                }]
            })
            .unwrap_or_default();

        match modal.mode {
            TaskModalMode::Create => {
                let id = EntityId::new_v7();
                self.tasks.push(Task {
                    id: id.clone(),
                    title: clean_title,
                    description_md: notes_md,
                    status: modal.status,
                    priority: modal.priority,
                    eisenhower_quadrant: None,
                    pinned_focus: false,
                    start_at: stamp.clone(),
                    due_at: stamp,
                    recurrence: None,
                    estimated_pomos: 1,
                    completed_pomos: 0,
                    project_links: links,
                    tags: Vec::new(),
                    checklist: modal.checklist,
                    created_at: now_utc,
                    updated_at: now_utc,
                });
                self.dirty_task_ids.insert(id);
            }
            TaskModalMode::Edit(id) => {
                if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                    task.title = clean_title;
                    task.description_md = notes_md;
                    task.status = modal.status;
                    task.priority = modal.priority;
                    task.start_at.clone_from(&stamp);
                    task.due_at = stamp;
                    task.project_links = links;
                    task.checklist = modal.checklist;
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(id);
                }
            }
        }
        self.task_modal = None;
        self.drawer = None;
    }

    fn update_modal_or_drawer(
        &mut self,
        message: ProjectsMessage,
        now_utc: chrono::DateTime<chrono::Utc>,
    ) {
        match message {
            ProjectsMessage::ModalTitleChanged(val) => {
                if let Some(m) = &mut self.task_modal {
                    m.title = val;
                    m.notice = None;
                }
            }
            ProjectsMessage::ModalNotesAction(action) => {
                if let Some(m) = &mut self.task_modal {
                    m.notes_content.perform(action);
                }
            }
            ProjectsMessage::ModalChecklistInputChanged(val) => {
                if let Some(m) = &mut self.task_modal {
                    m.new_checklist_title = val;
                }
            }
            ProjectsMessage::ModalAddChecklistItem => {
                if let Some(m) = &mut self.task_modal {
                    let item_title = m.new_checklist_title.trim().to_string();
                    if item_title.is_empty() {
                        return;
                    }
                    if m.checklist.len() >= MAX_TASK_CHECKLIST_ITEMS {
                        m.notice = Some(format!(
                            "Limite máximo de {MAX_TASK_CHECKLIST_ITEMS} itens na checklist."
                        ));
                        return;
                    }
                    let order = i32::try_from(m.checklist.len()).unwrap_or(0);
                    m.checklist.push(TaskChecklistItem {
                        id: EntityId::new_v7(),
                        title: item_title,
                        done: false,
                        sort_order: order,
                    });
                    m.new_checklist_title.clear();
                }
            }
            ProjectsMessage::ModalToggleChecklistItem(item_id) => {
                if let Some(m) = &mut self.task_modal
                    && let Some(item) = m.checklist.iter_mut().find(|i| i.id == item_id)
                {
                    item.done = !item.done;
                }
            }
            ProjectsMessage::ModalRemoveChecklistItem(item_id) => {
                if let Some(m) = &mut self.task_modal {
                    m.checklist.retain(|i| i.id != item_id);
                }
            }
            ProjectsMessage::ModalSetStatus(status) => {
                if let Some(m) = &mut self.task_modal {
                    m.status = status;
                }
            }
            ProjectsMessage::ModalSetPriority(prio) => {
                if let Some(m) = &mut self.task_modal {
                    m.priority = prio;
                }
            }
            ProjectsMessage::ModalToggleHasDate => {
                if let Some(m) = &mut self.task_modal {
                    m.has_date = !m.has_date;
                }
            }
            ProjectsMessage::ModalSetDate(date) => {
                if let Some(m) = &mut self.task_modal {
                    m.date = date;
                    m.has_date = true;
                }
            }
            ProjectsMessage::ModalSetTurn(turn) => {
                if let Some(m) = &mut self.task_modal {
                    m.turn = turn;
                }
            }
            ProjectsMessage::ModalSetProject(proj_id) => {
                if let Some(m) = &mut self.task_modal {
                    m.project_id = proj_id;
                }
            }
            other => self.update_drawer(other, now_utc),
        }
    }

    fn switch_framework(&mut self, framework: ProjectFramework) {
        let target_id = self
            .selected_project_id
            .clone()
            .or_else(|| self.projects.first().map(|p| p.id.clone()));
        if let Some(id) = target_id
            && let Some(project) = self.projects.iter_mut().find(|proj| proj.id == id)
        {
            project.framework = framework;
            self.dirty_project_ids.insert(id);
        }
        self.view_mode = match framework {
            ProjectFramework::WeeklyTurns => TasksViewMode::Week,
            ProjectFramework::Kanban
            | ProjectFramework::Milestones
            | ProjectFramework::Eisenhower
            | ProjectFramework::WeeklyFocus => TasksViewMode::Kanban,
        };
    }

    fn shift_month(&mut self, delta: i32) {
        let total_months =
            self.calendar_year * 12 + (self.calendar_month.cast_signed() - 1) + delta;
        self.calendar_year = total_months.div_euclid(12);
        self.calendar_month = u32::try_from(total_months.rem_euclid(12) + 1).unwrap_or(1);
    }

    fn create_quick_task(&mut self, now_utc: chrono::DateTime<chrono::Utc>) {
        let title = self.quick_task_title.trim().to_string();
        if title.is_empty() {
            return;
        }
        let links = self
            .selected_project_id
            .clone()
            .or_else(|| self.projects.first().map(|proj| proj.id.clone()))
            .map(|project_id| {
                vec![TaskProjectLink {
                    project_id,
                    milestone_name: None,
                }]
            })
            .unwrap_or_default();

        let id = EntityId::new_v7();
        self.tasks.push(Task {
            id: id.clone(),
            title,
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
            project_links: links,
            tags: Vec::new(),
            checklist: Vec::new(),
            created_at: now_utc,
            updated_at: now_utc,
        });
        self.dirty_task_ids.insert(id);
        self.quick_task_title.clear();
    }

    fn create_quick_task_in_turn(
        &mut self,
        date: NaiveDate,
        turn: TaskTurn,
        now_utc: chrono::DateTime<chrono::Utc>,
    ) {
        let title = if self.quick_task_title.trim().is_empty() {
            self.open_create_modal(Some(date), turn);
            return;
        } else {
            self.quick_task_title.trim().to_string()
        };
        let links = self
            .selected_project_id
            .clone()
            .or_else(|| self.projects.first().map(|proj| proj.id.clone()))
            .map(|project_id| {
                vec![TaskProjectLink {
                    project_id,
                    milestone_name: None,
                }]
            })
            .unwrap_or_default();

        let id = EntityId::new_v7();
        let timestamp = format_date_and_turn(date, turn);
        self.tasks.push(Task {
            id: id.clone(),
            title,
            description_md: String::new(),
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            eisenhower_quadrant: None,
            pinned_focus: false,
            start_at: Some(timestamp.clone()),
            due_at: Some(timestamp),
            recurrence: None,
            estimated_pomos: 1,
            completed_pomos: 0,
            project_links: links,
            tags: Vec::new(),
            checklist: Vec::new(),
            created_at: now_utc,
            updated_at: now_utc,
        });
        self.dirty_task_ids.insert(id);
        self.quick_task_title.clear();
    }

    fn move_task_status(
        &mut self,
        task_id: &EntityId,
        status: TaskStatus,
        now_utc: chrono::DateTime<chrono::Utc>,
    ) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == *task_id) {
            task.status = status;
            task.updated_at = now_utc;
            self.dirty_task_ids.insert(task_id.clone());
        }
    }

    fn update_drawer(&mut self, message: ProjectsMessage, now_utc: chrono::DateTime<chrono::Utc>) {
        let Some(drawer) = &mut self.drawer else {
            return;
        };
        let target_id = drawer.task_id.clone();

        match message {
            ProjectsMessage::SetDescriptionTab(tab) => drawer.description_tab = tab,
            ProjectsMessage::DrawerChecklistInputChanged(val) => {
                drawer.new_checklist_title = val;
            }
            ProjectsMessage::DrawerTagInputChanged(val) => drawer.new_tag_name = val,
            ProjectsMessage::DrawerTitleChanged(title) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.title = title;
                    task.updated_at = now_utc;
                    self.deferred_task_ids.insert(target_id);
                }
            }
            ProjectsMessage::DrawerDescriptionChanged(desc) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.description_md = desc;
                    task.updated_at = now_utc;
                    self.deferred_task_ids.insert(target_id);
                }
            }
            ProjectsMessage::DrawerStartAtChanged(val) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.start_at = (!val.trim().is_empty()).then_some(val);
                    task.updated_at = now_utc;
                    self.deferred_task_ids.insert(target_id);
                }
            }
            ProjectsMessage::DrawerDueAtChanged(val) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.due_at = (!val.trim().is_empty()).then_some(val);
                    task.updated_at = now_utc;
                    self.deferred_task_ids.insert(target_id);
                }
            }
            ProjectsMessage::DrawerRecurrenceChanged(rec) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.recurrence = rec;
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(target_id);
                }
            }
            ProjectsMessage::DrawerPriorityChanged(priority) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.priority = priority;
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(target_id);
                }
            }
            ProjectsMessage::DrawerToggleFocusPin => self.toggle_drawer_focus_pin(&target_id),
            ProjectsMessage::DrawerToggleProjectLink(project_id) => {
                self.toggle_drawer_project_link(&target_id, project_id);
            }
            ProjectsMessage::DrawerAddChecklistItem => self.add_drawer_checklist_item(&target_id),
            ProjectsMessage::DrawerToggleChecklistItem(item_id) => {
                self.toggle_drawer_checklist_item(&target_id, &item_id);
            }
            ProjectsMessage::DrawerRemoveChecklistItem(item_id) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.checklist.retain(|item| item.id != item_id);
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(target_id);
                }
            }
            ProjectsMessage::DrawerAddTag => self.add_drawer_tag(&target_id),
            ProjectsMessage::DrawerRemoveTag(tag) => {
                if let Some(task) = self.tasks.iter_mut().find(|task| task.id == target_id) {
                    task.tags.retain(|item| item != &tag);
                    task.updated_at = now_utc;
                    self.dirty_task_ids.insert(target_id);
                }
            }
            _ => {}
        }
    }

    fn toggle_drawer_focus_pin(&mut self, task_id: &EntityId) {
        let pinned_count = self.tasks.iter().filter(|task| task.pinned_focus).count();
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == *task_id) else {
            return;
        };

        let next_pin = !task.pinned_focus;
        match validate_focus_pin_budget(pinned_count, next_pin, task.pinned_focus) {
            Ok(()) => {
                task.pinned_focus = next_pin;
                self.dirty_task_ids.insert(task_id.clone());
                if let Some(drawer) = &mut self.drawer {
                    drawer.notice = None;
                }
            }
            Err(_) => {
                if let Some(drawer) = &mut self.drawer {
                    drawer.notice = Some(format!(
                        "Limite de {MAX_FOCUS_PINNED_TASKS} itens em foco já atingido."
                    ));
                }
            }
        }
    }

    fn toggle_drawer_project_link(&mut self, task_id: &EntityId, project_id: EntityId) {
        let Some(task) = self.tasks.iter_mut().find(|task| task.id == *task_id) else {
            return;
        };
        if let Some(pos) = task
            .project_links
            .iter()
            .position(|link| link.project_id == project_id)
        {
            task.project_links.remove(pos);
        } else {
            task.project_links.push(TaskProjectLink {
                project_id,
                milestone_name: None,
            });
        }
        self.dirty_task_ids.insert(task_id.clone());
    }

    fn add_drawer_checklist_item(&mut self, task_id: &EntityId) {
        let Some(drawer) = &mut self.drawer else {
            return;
        };
        let title = drawer.new_checklist_title.trim().to_string();
        if title.is_empty() {
            return;
        }

        let Some(task) = self.tasks.iter_mut().find(|task| task.id == *task_id) else {
            return;
        };

        match task.add_checklist_item(title) {
            Ok(()) => {
                drawer.new_checklist_title.clear();
                drawer.notice = None;
                self.dirty_task_ids.insert(task_id.clone());
            }
            Err(_) => {
                drawer.notice = Some(format!(
                    "Limite máximo de {MAX_TASK_CHECKLIST_ITEMS} itens na checklist atingido."
                ));
            }
        }
    }

    fn toggle_drawer_checklist_item(&mut self, task_id: &EntityId, item_id: &EntityId) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == *task_id)
            && let Some(item) = task.checklist.iter_mut().find(|item| item.id == *item_id)
        {
            item.done = !item.done;
            self.dirty_task_ids.insert(task_id.clone());
        }
    }

    fn add_drawer_tag(&mut self, task_id: &EntityId) {
        let Some(drawer) = &mut self.drawer else {
            return;
        };
        let tag = drawer.new_tag_name.trim().to_string();
        if tag.is_empty() {
            return;
        }
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == *task_id)
            && !task
                .tags
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&tag))
        {
            task.tags.push(tag);
            self.dirty_task_ids.insert(task_id.clone());
        }
        drawer.new_tag_name.clear();
    }
}

/// Extrai a data (`YYYY-MM-DD`) de `start_at` ou `due_at` de uma tarefa, se agendada.
#[must_use]
pub fn extract_task_date(task: &Task) -> Option<NaiveDate> {
    let raw = task.start_at.as_deref().or(task.due_at.as_deref())?.trim();
    if raw.len() < 10 {
        return None;
    }
    NaiveDate::parse_from_str(&raw[..10], "%Y-%m-%d").ok()
}

fn classify_existing_or_anytime(task: &Task) -> TaskTurn {
    if task.start_at.is_some() || task.due_at.is_some() {
        classify_task_turn(
            task.start_at.as_deref(),
            task.due_at.as_deref(),
            task.priority,
        )
    } else {
        TaskTurn::Anytime
    }
}

fn format_date_and_turn(date: NaiveDate, turn: TaskTurn) -> String {
    let hour = turn.default_hour();
    if hour.is_empty() {
        date.format("%Y-%m-%d").to_string()
    } else {
        format!("{} {hour}", date.format("%Y-%m-%d"))
    }
}

fn sample_tasks(
    proj_estudy: &EntityId,
    proj_semestre: &EntityId,
    today: NaiveDate,
    now_utc: chrono::DateTime<chrono::Utc>,
) -> Vec<Task> {
    let today_str = today.format("%Y-%m-%d").to_string();
    let tomorrow_str = (today + Duration::days(1)).format("%Y-%m-%d").to_string();
    vec![
        Task {
            id: EntityId::new_v7(),
            title: "Kanban 4 colunas e filtros por tag".to_string(),
            description_md:
                "# Escopo\n- Visão Semana, Calendário e Kanban\n- Drag & Drop fluido entre turnos"
                    .to_string(),
            status: TaskStatus::InProgress,
            priority: TaskPriority::High,
            eisenhower_quadrant: Some(EisenhowerQuadrant::Do),
            pinned_focus: true,
            start_at: Some(format!("{today_str} 09:00")),
            due_at: Some(format!("{today_str} 18:00")),
            recurrence: None,
            estimated_pomos: 3,
            completed_pomos: 1,
            project_links: vec![TaskProjectLink {
                project_id: proj_estudy.clone(),
                milestone_name: Some("Marco v0.2.0".to_string()),
            }],
            tags: vec!["rust".to_string(), "ui".to_string()],
            checklist: vec![
                TaskChecklistItem {
                    id: EntityId::new_v7(),
                    title: "Modelar colunas e turnos".to_string(),
                    done: true,
                    sort_order: 0,
                },
                TaskChecklistItem {
                    id: EntityId::new_v7(),
                    title: "Conectar Modal multi-linha".to_string(),
                    done: false,
                    sort_order: 1,
                },
            ],
            created_at: now_utc,
            updated_at: now_utc,
        },
        Task {
            id: EntityId::new_v7(),
            title: "Lista 3 de Cálculo Vetorial".to_string(),
            description_md: "Resolver exercícios ímpares da seção 14.4.".to_string(),
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            eisenhower_quadrant: Some(EisenhowerQuadrant::Schedule),
            pinned_focus: false,
            start_at: Some(tomorrow_str.clone()),
            due_at: Some(tomorrow_str),
            recurrence: Some("semanal".to_string()),
            estimated_pomos: 2,
            completed_pomos: 0,
            project_links: vec![TaskProjectLink {
                project_id: proj_semestre.clone(),
                milestone_name: Some("Prova P2".to_string()),
            }],
            tags: vec!["calculo".to_string()],
            checklist: Vec::new(),
            created_at: now_utc,
            updated_at: now_utc,
        },
        Task {
            id: EntityId::new_v7(),
            title: "Organizar arquivos de projetos antigos".to_string(),
            description_md: "Revisar pastas do semestre anterior.".to_string(),
            status: TaskStatus::Todo,
            priority: TaskPriority::Low,
            eisenhower_quadrant: Some(EisenhowerQuadrant::Do),
            pinned_focus: false,
            start_at: None,
            due_at: None,
            recurrence: None,
            estimated_pomos: 2,
            completed_pomos: 0,
            project_links: vec![TaskProjectLink {
                project_id: proj_estudy.clone(),
                milestone_name: Some("Marco v0.1.0".to_string()),
            }],
            tags: vec!["design".to_string()],
            checklist: Vec::new(),
            created_at: now_utc,
            updated_at: now_utc,
        },
    ]
}
