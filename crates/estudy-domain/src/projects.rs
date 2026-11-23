//! Entidades, enums e regras puras de Projetos, Tarefas, Checklists e Frameworks Leves.

use chrono::{DateTime, Utc};

use crate::error::DomainError;
use crate::ids::EntityId;

/// Limite máximo de itens na checklist de uma tarefa (`0/20`).
pub const MAX_TASK_CHECKLIST_ITEMS: usize = 20;

/// Limite máximo de tarefas pinadas simultaneamente em "Em foco".
pub const MAX_FOCUS_PINNED_TASKS: usize = 5;

/// Estado macro de um projeto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectStatus {
    /// Em planejamento.
    Planning,
    /// Em andamento.
    InProgress,
    /// Pausado.
    Paused,
    /// Concluído.
    Done,
}

const PROJECT_STATUS_MAP: &[(ProjectStatus, &str, &str)] = &[
    (ProjectStatus::Planning, "planning", "Planejando"),
    (ProjectStatus::InProgress, "in_progress", "Em andamento"),
    (ProjectStatus::Paused, "paused", "Pausado"),
    (ProjectStatus::Done, "done", "Concluído"),
];

impl ProjectStatus {
    /// Representação persistida no `SQLite`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        find_db_str(PROJECT_STATUS_MAP, self)
    }

    /// Rótulo legível em português.
    #[must_use]
    pub fn label(self) -> &'static str {
        find_label(PROJECT_STATUS_MAP, self)
    }

    /// Faz parse do valor persistido no banco.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        parse_from_map(PROJECT_STATUS_MAP, "status do projeto", raw)
    }
}

/// Framework opcional de visualização e organização por projeto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectFramework {
    /// Kanban de 4 colunas (padrão).
    Kanban,
    /// Marcos & Entregas com barra de progresso automática.
    Milestones,
    /// Matriz de Prioridade 2×2 (Eisenhower).
    Eisenhower,
    /// Foco Semanal (metas da semana).
    WeeklyFocus,
}

const FRAMEWORK_MAP: &[(ProjectFramework, &str, &str)] = &[
    (ProjectFramework::Kanban, "kanban", "Kanban Simples"),
    (
        ProjectFramework::Milestones,
        "milestones",
        "Marcos & Entregas",
    ),
    (
        ProjectFramework::Eisenhower,
        "eisenhower",
        "Matriz Eisenhower",
    ),
    (
        ProjectFramework::WeeklyFocus,
        "weekly_focus",
        "Foco Semanal",
    ),
];

impl ProjectFramework {
    /// Todos os 4 frameworks suportados.
    pub const ALL: [Self; 4] = [
        Self::Kanban,
        Self::Milestones,
        Self::Eisenhower,
        Self::WeeklyFocus,
    ];

    /// Representação persistida no `SQLite`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        find_db_str(FRAMEWORK_MAP, self)
    }

    /// Rótulo legível em português.
    #[must_use]
    pub fn label(self) -> &'static str {
        find_label(FRAMEWORK_MAP, self)
    }

    /// Faz parse do valor persistido no banco.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        parse_from_map(FRAMEWORK_MAP, "framework do projeto", raw)
    }
}

/// Coluna do Kanban de uma tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// Caixa de entrada / Backlog.
    Backlog,
    /// A fazer.
    Todo,
    /// Em andamento.
    InProgress,
    /// Concluído.
    Done,
}

const TASK_STATUS_MAP: &[(TaskStatus, &str, &str)] = &[
    (TaskStatus::Backlog, "backlog", "Backlog"),
    (TaskStatus::Todo, "todo", "A fazer"),
    (TaskStatus::InProgress, "in_progress", "Em andamento"),
    (TaskStatus::Done, "done", "Concluído"),
];

impl TaskStatus {
    /// As 4 colunas do Kanban em ordem.
    pub const ALL: [Self; 4] = [Self::Backlog, Self::Todo, Self::InProgress, Self::Done];

    /// Representação persistida no `SQLite`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        find_db_str(TASK_STATUS_MAP, self)
    }

    /// Rótulo da coluna em português.
    #[must_use]
    pub fn label(self) -> &'static str {
        find_label(TASK_STATUS_MAP, self)
    }

    /// Faz parse do valor persistido no banco.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        parse_from_map(TASK_STATUS_MAP, "status da tarefa", raw)
    }
}

/// Nível de prioridade de uma tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskPriority {
    /// Baixa.
    Low,
    /// Média.
    Medium,
    /// Alta.
    High,
    /// Urgente.
    Urgent,
}

const PRIORITY_MAP: &[(TaskPriority, &str, &str)] = &[
    (TaskPriority::Low, "low", "Baixa"),
    (TaskPriority::Medium, "medium", "Média"),
    (TaskPriority::High, "high", "Alta"),
    (TaskPriority::Urgent, "urgent", "Urgente"),
];

impl TaskPriority {
    /// Todas as prioridades disponíveis.
    pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Urgent];

    /// Representação persistida no `SQLite`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        find_db_str(PRIORITY_MAP, self)
    }

    /// Rótulo legível em português.
    #[must_use]
    pub fn label(self) -> &'static str {
        find_label(PRIORITY_MAP, self)
    }

    /// Faz parse do valor persistido no banco.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        parse_from_map(PRIORITY_MAP, "prioridade da tarefa", raw)
    }
}

/// Quadrante da Matriz de Eisenhower (`Urgente × Importante`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EisenhowerQuadrant {
    /// Urgente e importante — fazer agora.
    Do,
    /// Importante, não urgente — agendar.
    Schedule,
    /// Urgente, menos importante — delegar.
    Delegate,
    /// Nem urgente nem importante — eliminar/adiar.
    Eliminate,
}

const EISENHOWER_MAP: &[(EisenhowerQuadrant, &str, &str)] = &[
    (EisenhowerQuadrant::Do, "do", "Fazer agora"),
    (EisenhowerQuadrant::Schedule, "schedule", "Agendar"),
    (EisenhowerQuadrant::Delegate, "delegate", "Delegar"),
    (EisenhowerQuadrant::Eliminate, "eliminate", "Eliminar"),
];

impl EisenhowerQuadrant {
    /// Os 4 quadrantes na ordem visual 2×2.
    pub const ALL: [Self; 4] = [Self::Do, Self::Schedule, Self::Delegate, Self::Eliminate];

    /// Representação persistida no `SQLite`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        find_db_str(EISENHOWER_MAP, self)
    }

    /// Rótulo legível em português.
    #[must_use]
    pub fn label(self) -> &'static str {
        find_label(EISENHOWER_MAP, self)
    }

    /// Faz parse do valor persistido no banco.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        parse_from_map(EISENHOWER_MAP, "quadrante de Eisenhower", raw)
    }
}

/// Projeto do usuário com escolha de framework leve opcional.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Identificador `UUIDv7`.
    pub id: EntityId,
    /// Nome do projeto.
    pub title: String,
    /// Descrição opcional.
    pub description: String,
    /// Token visual de cor.
    pub color_token: String,
    /// Estado macro.
    pub status: ProjectStatus,
    /// Framework de organização selecionado para este projeto.
    pub framework: ProjectFramework,
    /// Ordem de exibição.
    pub sort_order: i32,
    /// Criação em UTC.
    pub created_at: DateTime<Utc>,
    /// Última atualização em UTC.
    pub updated_at: DateTime<Utc>,
}

/// Item da checklist (`0/20`) vinculada a uma tarefa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskChecklistItem {
    /// Identificador `UUIDv7`.
    pub id: EntityId,
    /// Texto do item.
    pub title: String,
    /// Indica se foi concluído.
    pub done: bool,
    /// Ordem dentro da tarefa.
    pub sort_order: i32,
}

/// Vínculo de uma tarefa a um projeto (permite múltiplos projetos e marco opcional).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskProjectLink {
    /// Identificador do projeto.
    pub project_id: EntityId,
    /// Nome opcional do marco (*Milestone*) dentro daquele projeto.
    pub milestone_name: Option<String>,
}

/// Tarefa completa manipulada pelo Kanban, frameworks e Drawer lateral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// Identificador `UUIDv7`.
    pub id: EntityId,
    /// Título da tarefa.
    pub title: String,
    /// Descrição em Markdown (`Escrever / Visualizar`).
    pub description_md: String,
    /// Coluna Kanban atual.
    pub status: TaskStatus,
    /// Prioridade.
    pub priority: TaskPriority,
    /// Quadrante opcional na Matriz de Eisenhower.
    pub eisenhower_quadrant: Option<EisenhowerQuadrant>,
    /// Fixada em "Em foco" (teto global de 5 itens).
    pub pinned_focus: bool,
    /// Início opcional (ISO-8601 / RFC 3339).
    pub start_at: Option<String>,
    /// Prazo opcional (ISO-8601 / RFC 3339).
    pub due_at: Option<String>,
    /// Regra opcional de recorrência em JSON/texto.
    pub recurrence: Option<String>,
    /// Pomodoros estimados.
    pub estimated_pomos: i32,
    /// Pomodoros concluídos.
    pub completed_pomos: i32,
    /// Projetos vinculados (suporta múltiplos projetos).
    pub project_links: Vec<TaskProjectLink>,
    /// Nomes das tags associadas.
    pub tags: Vec<String>,
    /// Itens da checklist (`0/20`).
    pub checklist: Vec<TaskChecklistItem>,
    /// Timestamp de criação.
    pub created_at: DateTime<Utc>,
    /// Timestamp de atualização.
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// Valida e adiciona um item na checklist respeitando o limite máximo de 20 itens.
    pub fn add_checklist_item(&mut self, title: impl Into<String>) -> Result<(), DomainError> {
        if self.checklist.len() >= MAX_TASK_CHECKLIST_ITEMS {
            return Err(DomainError::ChecklistLimitReached {
                max: MAX_TASK_CHECKLIST_ITEMS,
            });
        }
        let sort_order = i32::try_from(self.checklist.len()).unwrap_or(i32::MAX);
        self.checklist.push(TaskChecklistItem {
            id: EntityId::new_v7(),
            title: title.into(),
            done: false,
            sort_order,
        });
        Ok(())
    }

    /// Retorna `(concluídos, total)` da checklist da tarefa.
    #[must_use]
    pub fn checklist_progress(&self) -> (usize, usize) {
        let done = self.checklist.iter().filter(|item| item.done).count();
        (done, self.checklist.len())
    }

    /// Calcula o progresso `(concluídas, total)` para um grupo de tarefas de um marco.
    #[must_use]
    pub fn milestone_progress(tasks: &[Self]) -> (usize, usize) {
        let done = tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Done)
            .count();
        (done, tasks.len())
    }
}

/// Garante o teto global de 5 tarefas fixadas em "Em foco".
pub fn validate_focus_pin_budget(
    current_pinned_count: usize,
    wants_pin: bool,
    already_pinned: bool,
) -> Result<(), DomainError> {
    if wants_pin && !already_pinned && current_pinned_count >= MAX_FOCUS_PINNED_TASKS {
        return Err(DomainError::FocusPinLimitReached {
            max: MAX_FOCUS_PINNED_TASKS,
        });
    }
    Ok(())
}

fn find_db_str<T: PartialEq + Copy>(
    table: &[(T, &'static str, &'static str)],
    value: T,
) -> &'static str {
    table
        .iter()
        .find(|(item, _, _)| *item == value)
        .map_or("", |(_, db_str, _)| *db_str)
}

fn find_label<T: PartialEq + Copy>(
    table: &[(T, &'static str, &'static str)],
    value: T,
) -> &'static str {
    table
        .iter()
        .find(|(item, _, _)| *item == value)
        .map_or("", |(_, _, label)| *label)
}

fn parse_from_map<T: Copy>(
    table: &[(T, &str, &str)],
    field: &'static str,
    raw: &str,
) -> Result<T, DomainError> {
    let normalized = raw.trim();
    table
        .iter()
        .find(|(_, db_str, _)| *db_str == normalized)
        .map(|(item, _, _)| *item)
        .ok_or_else(|| DomainError::InvalidProjectValue {
            field,
            value: normalized.to_string(),
        })
}
