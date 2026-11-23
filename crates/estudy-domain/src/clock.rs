//! Porta de relógio para determinismo em testes e suporte a relógio virtual.

use chrono::{DateTime, Duration, Utc};
use std::sync::{Arc, Mutex};

/// Contrato para obtenção do instante UTC atual sem acoplar o domínio ao relógio do SO.
pub trait ClockPort: Send + Sync {
    /// Retorna a data e hora atual em UTC.
    fn now_utc(&self) -> DateTime<Utc>;
}

/// Implementação padrão que delega ao relógio real do sistema operacional.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl ClockPort for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Relógio controlável em memória para testes unitários, modo Admin e QA E2E.
#[derive(Debug, Clone)]
pub struct FixedClock {
    now: Arc<Mutex<DateTime<Utc>>>,
}

impl FixedClock {
    /// Cria um relógio fixado no instante UTC informado.
    #[must_use]
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self {
            now: Arc::new(Mutex::new(initial)),
        }
    }

    /// Avança o relógio virtual pela duração informada.
    pub fn advance(&self, delta: Duration) {
        if let Ok(mut guard) = self.now.lock() {
            *guard += delta;
        }
    }
}

impl ClockPort for FixedClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.now.lock().map_or(DateTime::UNIX_EPOCH, |guard| *guard)
    }
}
