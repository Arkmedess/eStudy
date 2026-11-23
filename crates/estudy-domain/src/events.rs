//! Eventos de domínio e porta de barramento pós-commit.

use std::sync::{Arc, Mutex};

use crate::ids::EntityId;

/// Eventos emitidos exclusivamente após o commit bem-sucedido de uma transação.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    /// Novo lançamento gravado e assinado no razão da carteira.
    WalletEntryRecorded {
        /// Perfil proprietário do lançamento.
        profile_id: EntityId,
        /// Número de sequência na cadeia HMAC.
        seq: i64,
        /// Variação de saldo aplicada.
        delta: i64,
    },
    /// Migração de banco de dados concluída com sucesso.
    DatabaseMigrated {
        /// Versão de destino aplicada.
        version: u32,
    },
    /// Projeto criado ou atualizado após commit.
    ProjectSaved {
        /// Identificador do projeto.
        project_id: EntityId,
    },
    /// Tarefa criada, movida ou atualizada após commit.
    TaskSaved {
        /// Identificador da tarefa.
        task_id: EntityId,
    },
    /// Evento de calendário criado ou atualizado após commit.
    CalendarEventSaved {
        /// Identificador do evento.
        event_id: EntityId,
    },
}

/// Porta para publicação desacoplada de eventos após a confirmação da persistência.
pub trait EventBusPort: Send + Sync {
    /// Publica um evento para os assinantes interessados.
    fn publish(&self, event: DomainEvent);
}

/// Implementação em memória útil para testes e inicialização enxuta.
#[derive(Debug, Default, Clone)]
pub struct InMemoryEventBus {
    published: Arc<Mutex<Vec<DomainEvent>>>,
}

impl InMemoryEventBus {
    /// Cria um barramento vazio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retorna uma cópia dos eventos publicados até o momento.
    #[must_use]
    pub fn snapshot(&self) -> Vec<DomainEvent> {
        self.published
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }
}

impl EventBusPort for InMemoryEventBus {
    fn publish(&self, event: DomainEvent) {
        if let Ok(mut guard) = self.published.lock() {
            guard.push(event);
        }
    }
}
