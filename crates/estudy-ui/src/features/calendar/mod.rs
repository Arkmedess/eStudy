//! Fatia vertical Calendário Unificado e Drawer "Hoje".

pub mod state;
pub mod today_drawer;
pub mod view;

pub use state::{CalendarMessage, CalendarState};
pub use today_drawer::view_today_drawer;
pub use view::view_calendar;
