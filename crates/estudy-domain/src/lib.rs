//! Núcleo de domínio puro do **eStudy**.
//!
//! Este crate não possui dependências de I/O, banco de dados ou interface gráfica.
//! Concentra identificadores ordenáveis (`EntityId`), abstração de relógio (`ClockPort`),
//! barramento de eventos (`EventBusPort`) e a cadeia criptográfica do razão da carteira.

pub mod clock;
pub mod error;
pub mod events;
pub mod ids;
pub mod wallet;

pub use clock::{ClockPort, FixedClock, SystemClock};
pub use error::DomainError;
pub use events::{DomainEvent, EventBusPort, InMemoryEventBus};
pub use ids::EntityId;
pub use wallet::{
    DAILY_COIN_CAP, GENESIS_PREV_HASH, LedgerEntryType, SignedWalletEntry, WalletChain,
    WalletEntryDraft,
};
