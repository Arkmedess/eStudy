//! Camada de interface MVU do **eStudy**.
//!
//! Organizada em fatias verticais testáveis sem GPU, com Barra de Comando (`Ctrl+K`)
//! e Home pessoal em 3 colunas sobre o `estudy-ui-kit`.

pub mod app;
pub mod command_palette;
pub mod features;

pub use app::{AppMessage, EstudyApp, QuickPopup, UndoAction};
pub use command_palette::{
    AppRoute, CommandPaletteState, PaletteAction, PaletteCommand, PaletteMessage,
};
pub use features::calendar::{CalendarMessage, CalendarState};
pub use features::habits::{HabitsMessage, HabitsState};
pub use features::home::{
    FocusPinItem, HomeDashboardData, HomeLoadState, HomeMessage, MAX_FOCUS_PINS, MAX_PINNED_MEDALS,
    MedalBadge, MoodDayEntry, MoodLevel, MoodViewMode, PinboardState, ProfileSummary,
    ProjectCardSummary, TodayItem,
};
pub use features::projects::{
    CONTEXT_SIDEBAR_WIDTH, ContextSubmenu, DescriptionTab, DragTarget, ProjectsMessage,
    ProjectsState, TaskContextMenu, TaskDrawerState, TaskModalMode, TaskModalState, TasksViewMode,
    UpcomingEventItem,
};
pub use features::second_brain::{BrainEditorTab, SecondBrainMessage, SecondBrainState};
