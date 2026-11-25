//! Núcleo de domínio puro do **eStudy**.
//!
//! Este crate não possui dependências de I/O, banco de dados ou interface gráfica.
//! Concentra identificadores ordenáveis (`EntityId`), abstração de relógio (`ClockPort`),
//! barramento de eventos (`EventBusPort`) e a cadeia criptográfica do razão da carteira.

pub mod calendar;
pub mod clock;
pub mod error;
pub mod events;
pub mod focus;
pub mod habits;
pub mod ids;
pub mod projects;
pub mod second_brain;
pub mod wallet;

pub use calendar::{
    CalendarEvent, CalendarOccurrence, CalendarViewMode, RecurrenceRule, TimelineSlot,
    TodayTimeline, build_today_timeline, expand_calendar_items, format_free_window_label,
};
pub use clock::{ClockPort, FixedClock, SystemClock};
pub use error::DomainError;
pub use events::{DomainEvent, EventBusPort, InMemoryEventBus};
pub use focus::{
    DspTrack, FocusModality, FocusPhase, FocusSessionDraft, MINIMUM_REWARDABLE_FOCUS_MINUTES,
    PomodoroPreset, PomodoroTimer, ValidatedFocusSession, calculate_focus_coins,
};
pub use habits::{Habit, days_in_month, overall_monthly_adherence_pct};
pub use ids::EntityId;
pub use projects::{
    EisenhowerQuadrant, MAX_FOCUS_PINNED_TASKS, MAX_TASK_CHECKLIST_ITEMS, Project,
    ProjectFramework, ProjectStatus, Task, TaskChecklistItem, TaskPriority, TaskProjectLink,
    TaskStatus, TaskTurn, classify_task_turn, validate_focus_pin_budget,
};
pub use second_brain::{KnowledgeNote, NoteKind};
pub use wallet::{
    DAILY_COIN_CAP, GENESIS_PREV_HASH, LedgerEntryType, SignedWalletEntry, WalletChain,
    WalletEntryDraft,
};
