//! Fatia vertical do Segundo Cérebro (`Ideias, notas, referências e links`).

mod state;
mod view;

pub use state::{BrainEditorTab, SecondBrainMessage, SecondBrainState};
pub use view::{view_note_editor_modal, view_second_brain};
