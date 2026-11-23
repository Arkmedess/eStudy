//! Backups automáticos pré-migração e exportação de snapshot limpo para nuvem.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, backup::Backup};
use std::path::{Path, PathBuf};

use crate::error::StorageError;

/// Quantidade máxima de backups pré-migração mantidos em rotação.
pub const MAX_PRE_MIGRATION_BACKUPS: usize = 5;

/// Gera um backup consistente antes de aplicar migrações usando a Online Backup API do `SQLite`.
pub fn create_pre_migration_backup(
    conn: &Connection,
    backups_dir: &Path,
    target_version: u32,
    now: DateTime<Utc>,
    max_keep: usize,
) -> Result<PathBuf, StorageError> {
    std::fs::create_dir_all(backups_dir).map_err(|source| StorageError::Io {
        path: backups_dir.to_path_buf(),
        source,
    })?;

    let stamp = now.format("%Y%m%dT%H%M%S");
    let filename = format!("estudy-pre-v{target_version}-{stamp}.db");
    let dest_path = backups_dir.join(filename);

    run_online_backup(conn, &dest_path)?;
    rotate_old_backups(backups_dir, max_keep)?;

    Ok(dest_path)
}

/// Exporta um arquivo `.db` único e consolidado (sem `-wal`/`-shm`) para pasta de nuvem.
pub fn export_clean_snapshot(
    conn: &Connection,
    destination_file: &Path,
) -> Result<PathBuf, StorageError> {
    if let Some(parent) = destination_file.parent() {
        std::fs::create_dir_all(parent).map_err(|source| StorageError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let temp_path = destination_file.with_extension("db.tmp");
    if temp_path.exists() {
        std::fs::remove_file(&temp_path).map_err(|source| StorageError::Io {
            path: temp_path.clone(),
            source,
        })?;
    }

    run_online_backup(conn, &temp_path)?;
    std::fs::rename(&temp_path, destination_file).map_err(|source| StorageError::Io {
        path: destination_file.to_path_buf(),
        source,
    })?;

    Ok(destination_file.to_path_buf())
}

fn run_online_backup(source: &Connection, dest_path: &Path) -> Result<(), StorageError> {
    let mut dest_conn = Connection::open(dest_path)?;
    dest_conn.execute_batch("PRAGMA journal_mode = DELETE;")?;
    {
        let backup = Backup::new(source, &mut dest_conn)?;
        backup.step(-1)?;
    }
    Ok(())
}

fn rotate_old_backups(backups_dir: &Path, max_keep: usize) -> Result<(), StorageError> {
    if max_keep == 0 {
        return Ok(());
    }

    let mut files: Vec<(String, PathBuf)> = std::fs::read_dir(backups_dir)
        .map_err(|source| StorageError::Io {
            path: backups_dir.to_path_buf(),
            source,
        })?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_db = path.extension().is_some_and(|ext| ext == "db");
            let name = path.file_name()?.to_str()?.to_string();
            (is_db && name.starts_with("estudy-pre-v")).then_some((name, path))
        })
        .collect();

    if files.len() <= max_keep {
        return Ok(());
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    let excess = files.len() - max_keep;
    for (_, old_path) in files.into_iter().take(excess) {
        std::fs::remove_file(&old_path).map_err(|source| StorageError::Io {
            path: old_path,
            source,
        })?;
    }
    Ok(())
}
