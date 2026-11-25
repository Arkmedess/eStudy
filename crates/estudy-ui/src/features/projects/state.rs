//! Estado e transições da fatia Projetos (Kanban + Frameworks Leves + Drawer da Tarefa).

use chrono::{Datelike, NaiveDate};
use estudy_domain::{
    EisenhowerQuadrant, EntityId, MAX_FOCUS_PINNED_TASKS, MAX_TASK_CHECKLIST_ITEMS, Project,
    ProjectFramework, ProjectStatus, Task, TaskChecklistItem, TaskPriority, TaskProjectLink,
    TaskStatus, TaskTurn, validate_focus_pin_budget,
};
use std::collections::BTreeSet;

/// Evento próximo exibido na faixa superior da visão de Projetos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpcomingEventItem {
    /// Horário ou data resumida (ex.: `Hoje · 18:00`).
    pub when_label: String,
    /// Título do evento.
    pub title: String,
    /// Projeto relacionado, se houver.
    pub project_title: Option<String>,
}

/// Modo do editor de descrição Markdown no Drawer da tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptionTab {
    /// Campo de escrita em Markdown.
    Write,
    /// Visualização formatada do Markdown.
    Preview,
}

/// Estado do Drawer deslizante à direita da tarefa selecionada.
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
    /// Mensagem de feedback ou erro de regra (ex.: limite 5 em foco ou 20 checklist).
    pub notice: Option<String>,
}

/// Estado completo da fatia Projetos.
#[derive(Debug, Clone)]
pub struct ProjectsState {
    /// Projetos ativos.
    pub projects: Vec<Project>,
    /// Tarefas ativas.
    pub tasks: Vec<Task>,
    /// Próximos eventos exibidos na faixa superior.
    pub upcoming_events: Vec<UpcomingEventItem>,
    /// Projeto atualmente selecionado (`None` exibe todos).
    pub selected_project_id: Option<EntityId>,
    /// Filtro rápido por tag (`None` exibe todas).
    pub active_tag_filter: Option<String>,
    /// Indica se a barra lateral esquerda contextual (`240px`) está visível (`Ctrl+B`).
    pub sidebar_visible: bool,
    /// Ano exibido no mini-calendário mensal.
    pub calendar_year: i32,
    /// Mês exibido no mini-calendário mensal (`1..=12`).
    pub calendar_month: u32,
    /// Dia selecionado no mini-calendário.
    pub selected_day: NaiveDate,
    /// Texto rápido para criar nova tarefa no quadro.
    pub quick_task_title: String,
    /// Conjunto de tarefas selecionadas para o framework Foco Semanal.
    pub weekly_focus_ids: BTreeSet<EntityId>,
    /// Drawer lateral direito aberto, se houver uma tarefa em edição.
    pub drawer: Option<TaskDrawerState>,
    dirty_project_ids: BTreeSet<EntityId>,
    dirty_task_ids: BTreeSet<EntityId>,
    deferred_task_ids: BTreeSet<EntityId>,
}

/// Mensagens da fatia Projetos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectsMessage {
    /// Recolhe ou expande a barra lateral contextual (`Ctrl+B`).
    ToggleSidebar,
    /// Seleciona um projeto (`None` mostra visão agregada).
    SelectProject(Option<EntityId>),
    /// Altera o framework opcional do projeto selecionado.
    SwitchFramework(ProjectFramework),
    /// Filtra tarefas por tag.
    ToggleTagFilter(Option<String>),
    /// Navega para o mês anterior ou próximo no mini-calendário.
    ShiftCalendarMonth(i32),
    /// Seleciona um dia específico no mini-calendário.
    SelectCalendarDay(NaiveDate),
    /// Atualiza o título da barra de criação rápida de tarefa.
    QuickTaskTitleChanged(String),
    /// Cria nova tarefa no projeto/coluna padrão.
    CreateQuickTask,
    /// Cria nova tarefa diretamente em um dia e turno específicos (`Manhã / Tarde / Noite`).
    CreateQuickTaskInTurn {
        /// Data alvo no calendário.
        date: NaiveDate,
        /// Turno selecionado.
        turn: TaskTurn,
    },
    /// Inicia um bloco Pomodoro imediatamente vinculado a esta tarefa.
    StartFocusForTask(EntityId),
    /// Move uma tarefa para outra coluna do Kanban.
    MoveTaskStatus(EntityId, TaskStatus),
    /// Move uma tarefa para um quadrante na Matriz de Eisenhower.
    SetTaskQuadrant(EntityId, EisenhowerQuadrant),
    /// Inclui ou remove uma tarefa das metas de Foco Semanal.
    ToggleWeeklyFocus(EntityId),
    /// Abre o Drawer lateral da tarefa.
    OpenTaskDrawer(EntityId),
    /// Fecha o Drawer lateral.
    CloseTaskDrawer,
    /// Alterna aba `Escrever / Visualizar` na descrição Markdown.
    SetDescriptionTab(DescriptionTab),
    /// Atualiza título da tarefa aberta no Drawer.
    DrawerTitleChanged(String),
    /// Atualiza descrição Markdown da tarefa aberta no Drawer.
    DrawerDescriptionChanged(String),
    /// Atualiza horário de início da tarefa aberta.
    DrawerStartAtChanged(String),
    /// Atualiza prazo da tarefa aberta.
    DrawerDueAtChanged(String),
    /// Seleciona recorrência da tarefa aberta.
    DrawerRecurrenceChanged(Option<String>),
    /// Alterna prioridade da tarefa aberta.
    DrawerPriorityChanged(TaskPriority),
    /// Alterna fixação em "Em foco" respeitando o limite de 5.
    DrawerToggleFocusPin,
    /// Vincula ou desvincula a tarefa de um projeto (múltiplos projetos).
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
                framework: ProjectFramework::Kanban,
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

        let tasks = sample_tasks(&proj_estudy, &proj_semestre, now_utc);
        let mut weekly_focus_ids = BTreeSet::new();
        if let Some(first) = tasks.first() {
            weekly_focus_ids.insert(first.id.clone());
        }

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
                    title: "Revisar marcos da v0.2.0".to_string(),
                    project_title: Some("eStudy Desktop".to_string()),
                },
            ],
            selected_project_id: Some(proj_estudy),
            active_tag_filter: None,
            sidebar_visible: true,
            calendar_year: today.year(),
            calendar_month: today.month(),
            selected_day: today,
            quick_task_title: String::new(),
            weekly_focus_ids,
            drawer: None,
            dirty_project_ids: BTreeSet::new(),
            dirty_task_ids: BTreeSet::new(),
            deferred_task_ids: BTreeSet::new(),
        }
    }

    /// Promove edições em rascunho no Drawer para gravação pendente (chamado ao fechar/sair).
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

    /// Framework ativo para visualização da área principal.
    #[must_use]
    pub fn active_framework(&self) -> ProjectFramework {
        self.active_project()
            .map_or(ProjectFramework::Kanban, |project| project.framework)
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

    /// Retorna a tarefa aberta atualmente no Drawer lateral.
    #[must_use]
    pub fn drawer_task(&self) -> Option<&Task> {
        let drawer = self.drawer.as_ref()?;
        self.tasks.iter().find(|task| task.id == drawer.task_id)
    }

    /// Processa mensagens da fatia Projetos retornando se houve alteração de estado.
    pub fn update(&mut self, message: ProjectsMessage, now_utc: chrono::DateTime<chrono::Utc>) {
        match message {
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
            ProjectsMessage::StartFocusForTask(_) => {}
            ProjectsMessage::MoveTaskStatus(id, status) => self.move_task_status(&id, status),
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
            }
            other => self.update_drawer(other, now_utc),
        }
    }

    fn switch_framework(&mut self, framework: ProjectFramework) {
        if let Some(target_id) = self.selected_project_id.clone()
            && let Some(project) = self.projects.iter_mut().find(|proj| proj.id == target_id)
        {
            project.framework = framework;
            self.dirty_project_ids.insert(target_id);
        }
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
            format!(
                "Bloco {} ({:02}/{:02})",
                turn.label(),
                date.day(),
                date.month()
            )
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
        let timestamp = format!("{} {}", date.format("%Y-%m-%d"), turn.default_hour());
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

    fn move_task_status(&mut self, task_id: &EntityId, status: TaskStatus) {
        if let Some(task) = self.tasks.iter_mut().find(|task| task.id == *task_id) {
            task.status = status;
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

fn sample_tasks(
    proj_estudy: &EntityId,
    proj_semestre: &EntityId,
    now_utc: chrono::DateTime<chrono::Utc>,
) -> Vec<Task> {
    vec![
        Task {
            id: EntityId::new_v7(),
            title: "Kanban 4 colunas e filtros por tag".to_string(),
            description_md:
                "# Escopo\n- Colunas Backlog, A fazer, Em andamento e Concluído\n- Filtros rápidos por tag"
                    .to_string(),
            status: TaskStatus::InProgress,
            priority: TaskPriority::High,
            eisenhower_quadrant: Some(EisenhowerQuadrant::Do),
            pinned_focus: true,
            start_at: Some("2026-11-22 09:00".to_string()),
            due_at: Some("2026-11-23 18:00".to_string()),
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
                    title: "Modelar colunas Kanban".to_string(),
                    done: true,
                    sort_order: 0,
                },
                TaskChecklistItem {
                    id: EntityId::new_v7(),
                    title: "Conectar Drawer lateral".to_string(),
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
            start_at: None,
            due_at: Some("2026-11-25 23:59".to_string()),
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
            title: "Especificar paleta e tokens WCAG AA".to_string(),
            description_md: "Checklist concluída na v0.1.0.".to_string(),
            status: TaskStatus::Done,
            priority: TaskPriority::Low,
            eisenhower_quadrant: Some(EisenhowerQuadrant::Do),
            pinned_focus: false,
            start_at: None,
            due_at: None,
            recurrence: None,
            estimated_pomos: 2,
            completed_pomos: 2,
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
