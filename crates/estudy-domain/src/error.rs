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
}
