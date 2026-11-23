//! Camada de persistência local-first do **eStudy** baseada em `SQLite` WAL.
//!
//! Fornece resolução XDG de diretórios, abertura configurada do `SQLite`,
//! migrações versionadas com backup prévio, snapshot limpo para nuvem e
//! repositório auditável da carteira HMAC.

pub mod connection;
pub mod error;
pub mod migrations;
pub mod paths;
pub mod project_repo;
pub mod snapshot;
pub mod wallet_repo;

pub use connection::{open_connection, open_in_memory};
pub use error::StorageError;
pub use migrations::{LATEST_SCHEMA_VERSION, MigrationReport, apply_migrations};
pub use paths::AppPaths;
pub use project_repo::SqliteProjectRepository;
pub use snapshot::{MAX_PRE_MIGRATION_BACKUPS, create_pre_migration_backup, export_clean_snapshot};
pub use wallet_repo::SqliteWalletRepository;
