//! Erros explícitos da camada de persistência `SQLite` e diretórios locais.

use estudy_domain::DomainError;
use std::path::PathBuf;
use thiserror::Error;

/// Erros retornados pelas operações de armazenamento do eStudy.
#[derive(Debug, Error)]
pub enum StorageError {
    /// Não foi possível resolver os diretórios padrão do sistema operacional.
    #[error("não foi possível determinar os diretórios padrão do usuário para o eStudy")]
    UnavailableProjectDirs,

    /// Erro de sistema de arquivos ao criar pastas, backups ou snapshots.
    #[error("erro de I/O em '{path}': {source}")]
    Io {
        /// Caminho envolvido na falha.
        path: PathBuf,
        /// Causa subjacente.
        #[source]
        source: std::io::Error,
    },

    /// Erro retornado pelo `SQLite` / `rusqlite`.
    #[error("erro SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Falha de integridade após migração ou verificação do banco.
    #[error("verificação de integridade do banco falhou: {0}")]
    IntegrityCheckFailed(String),

    /// Erro propagado das regras de domínio (ex.: adulteração HMAC ou saldo insuficiente).
    #[error(transparent)]
    Domain(#[from] DomainError),
}
