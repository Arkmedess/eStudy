//! Fatia vertical Projetos (Kanban 4 colunas, Frameworks Leves e Drawer da Tarefa).

pub mod state;
pub mod view;

pub use state::{
    DescriptionTab, ProjectsMessage, ProjectsState, TaskDrawerState, UpcomingEventItem,
};
pub use view::{CONTEXT_SIDEBAR_WIDTH, view_projects};
