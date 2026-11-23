//! Executor de migrações versionadas com backup prévio e validação pós-migração.

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::error::StorageError;
use crate::snapshot::{MAX_PRE_MIGRATION_BACKUPS, create_pre_migration_backup};

/// Versão atual do schema relacional baseline (`v0.0.1`).
pub const LATEST_SCHEMA_VERSION: u32 = 1;

struct Migration {
    version: u32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial_v1",
    sql: include_str!("../migrations/0001_initial_v1.sql"),
}];

/// Resultado resumido de uma execução de migrações.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    /// Versões recém-aplicadas nesta execução.
    pub applied_versions: Vec<u32>,
    /// Versão final ativa no banco após a execução.
    pub current_version: u32,
}

/// Aplica migrações pendentes dentro de transação explícita e valida integridade ao final.
pub fn apply_migrations(
    conn: &mut Connection,
    backups_dir: Option<&Path>,
    now: DateTime<Utc>,
) -> Result<MigrationReport, StorageError> {
    let current_version = read_current_version(conn)?;
    let pending: Vec<&Migration> = MIGRATIONS
        .iter()
        .filter(|migration| migration.version > current_version)
        .collect();

    if pending.is_empty() {
        return Ok(MigrationReport {
            applied_versions: Vec::new(),
            current_version,
        });
    }

    if let (Some(dir), Some(target)) = (backups_dir, pending.last())
        && current_version > 0
    {
        create_pre_migration_backup(conn, dir, target.version, now, MAX_PRE_MIGRATION_BACKUPS)?;
    }

    let mut applied_versions = Vec::with_capacity(pending.len());
    let applied_at = now.to_rfc3339_opts(SecondsFormat::Secs, true);

    for migration in pending {
        apply_single_migration(conn, migration, &applied_at)?;
        applied_versions.push(migration.version);
    }

    verify_database_integrity(conn)?;

    let final_version = read_current_version(conn)?;
    tracing::info!(
        target: "estudy.storage",
        current_version = final_version,
        applied_count = applied_versions.len(),
        "migrações aplicadas e integridade verificada"
    );
    Ok(MigrationReport {
        applied_versions,
        current_version: final_version,
    })
}

fn read_current_version(conn: &Connection) -> Result<u32, StorageError> {
    let has_table: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
        [],
        |row| row.get(0),
    )?;

    if has_table == 0 {
        return Ok(0);
    }

    let version: Option<u32> =
        conn.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })?;
    Ok(version.unwrap_or(0))
}

fn apply_single_migration(
    conn: &mut Connection,
    migration: &Migration,
    applied_at: &str,
) -> Result<(), StorageError> {
    let checksum = hex::encode(Sha256::digest(migration.sql.as_bytes()));
    let tx = conn.transaction()?;
    tx.execute_batch(migration.sql)?;
    tx.execute(
        "INSERT INTO schema_migrations (version, name, checksum_sha256, applied_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![migration.version, migration.name, checksum, applied_at],
    )?;
    tx.commit()?;
    Ok(())
}

fn verify_database_integrity(conn: &Connection) -> Result<(), StorageError> {
    let quick_check: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if quick_check != "ok" {
        return Err(StorageError::IntegrityCheckFailed(format!(
            "PRAGMA quick_check retornou: {quick_check}"
        )));
    }

    let mut fk_stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let fk_violations = fk_stmt.query_map([], |_row| Ok(()))?.count();
    if fk_violations > 0 {
        return Err(StorageError::IntegrityCheckFailed(format!(
            "PRAGMA foreign_key_check encontrou {fk_violations} violações"
        )));
    }

    Ok(())
}
