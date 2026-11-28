//! Fatia Tarefas & Projetos (`Semana | Calendário | Kanban`, Drag & Drop, Menu de Contexto e Modais).

pub mod modals;
pub mod state;
pub mod view;

pub use state::{
    ContextSubmenu, DescriptionTab, DragTarget, ProjectsMessage, ProjectsState, TaskContextMenu,
    TaskDrawerState, TaskModalMode, TaskModalState, TasksViewMode, UpcomingEventItem,
    project_color_index,
};
pub use view::{CONTEXT_SIDEBAR_WIDTH, view_projects};
