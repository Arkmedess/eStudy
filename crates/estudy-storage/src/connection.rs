//! Abertura e configuração de conexões `SQLite` em modo WAL com teto de memória.

use rusqlite::Connection;
use std::path::Path;

use crate::error::StorageError;

const COMMON_PRAGMAS: &str = "
    PRAGMA foreign_keys = ON;
    PRAGMA synchronous = NORMAL;
    PRAGMA busy_timeout = 5000;
    PRAGMA cache_size = -4000;
    PRAGMA temp_store = MEMORY;
";

/// Abre uma conexão `SQLite` em arquivo aplicando os pragmas obrigatórios de integridade e RAM.
pub fn open_connection(db_path: &Path) -> Result<Connection, StorageError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| StorageError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch(COMMON_PRAGMAS)?;
    Ok(conn)
}

/// Abre um banco `SQLite` em memória com as mesmas regras de integridade para testes rápidos.
pub fn open_in_memory() -> Result<Connection, StorageError> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(COMMON_PRAGMAS)?;
    Ok(conn)
}
