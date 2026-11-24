//! Erros explícitos de regras de negócio e integridade de domínio.

use thiserror::Error;

/// Erros produzidos pelas regras puras do domínio do eStudy.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    /// Identificador `UUIDv7` inválido.
    #[error("identificador inválido: {0}")]
    InvalidEntityId(String),

    /// Tipo de lançamento desconhecido no razão da carteira.
    #[error("tipo de lançamento desconhecido: {0}")]
    InvalidLedgerEntryType(String),

    /// Quantia inválida para o tipo de lançamento informado.
    #[error("quantia inválida ({amount}) para lançamento do tipo '{entry_type}'")]
    InvalidWalletAmount {
        /// Tipo do lançamento.
        entry_type: &'static str,
        /// Valor informado.
        amount: i64,
    },

    /// Saldo insuficiente para concluir uma compra na loja.
    #[error("saldo insuficiente: disponível {available}, necessário {required}")]
    InsufficientBalance {
        /// Saldo atual verificado.
        available: i64,
        /// Valor exigido pela operação.
        required: i64,
    },

    /// Quebra de sequência ou assinatura na cadeia HMAC do razão da carteira.
    #[error("adulteração detectada no razão da carteira na sequência {seq}: {reason}")]
    WalletChainTampered {
        /// Número de sequência onde a verificação falhou.
        seq: i64,
        /// Motivo específico da divergência.
        reason: String,
    },

    /// Falha ao derivar chave HMAC ou inicializar o algoritmo criptográfico.
    #[error("falha criptográfica interna: {0}")]
    CryptoFailure(String),

    /// Limite máximo de itens na checklist da tarefa atingido (`0/20`).
    #[error("a checklist da tarefa atingiu o limite máximo de {max} itens")]
    ChecklistLimitReached {
        /// Limite máximo permitido.
        max: usize,
    },

    /// Limite máximo de tarefas fixadas em "Em foco" atingido (`5`).
    #[error("o limite máximo de {max} itens em foco já foi atingido")]
    FocusPinLimitReached {
        /// Limite máximo permitido.
        max: usize,
    },

    /// Texto inválido ao converter status, prioridade, framework ou quadrante.
    #[error("valor inválido para {field}: {value}")]
    InvalidProjectValue {
        /// Campo avaliado.
        field: &'static str,
        /// Valor recebido.
        value: String,
    },

    /// Intervalo de datas ou horários inválido em um evento de calendário.
    #[error(
        "intervalo inválido no evento: início ({starts_at}) deve ser anterior ou igual ao fim ({ends_at})"
    )]
    InvalidEventTimeRange {
        /// Início informado.
        starts_at: String,
        /// Fim informado.
        ends_at: String,
    },

    /// Regra violada ao validar ou registrar uma sessão Pomodoro/Foco.
    #[error("sessão de foco inválida: {0}")]
    InvalidFocusSession(String),
}
