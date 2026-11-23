//! Resolução multiplataforma de diretórios seguindo o padrão XDG/SO.

use directories::ProjectDirs;
use std::path::{Path, PathBuf};

use crate::error::StorageError;

/// Nome da variável de ambiente que sobrescreve o diretório de dados.
pub const DATA_DIR_ENV_VAR: &str = "ESTUDY_DATA_DIR";

/// Conjunto de diretórios locais gerenciados pelo eStudy.
///
/// Separa dados do usuário (sincronizáveis via snapshot) de cache descartável.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// Pasta principal de dados persistentes (`estudy.db` e `backups/`).
    pub data_dir: PathBuf,
    /// Pasta de configurações (`config.toml` e sobrescritas de tema).
    pub config_dir: PathBuf,
    /// Pasta de cache descartável (`covers/` e `logs/`).
    pub cache_dir: PathBuf,
}

impl AppPaths {
    /// Resolve os diretórios padrão do SO aceitando override via `--data-dir` ou `ESTUDY_DATA_DIR`.
    pub fn resolve(cli_data_dir: Option<&Path>) -> Result<Self, StorageError> {
        let env_data_dir = std::env::var_os(DATA_DIR_ENV_VAR)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        let override_dir = cli_data_dir.map(Path::to_path_buf).or(env_data_dir);

        if let Some(data_dir) = override_dir {
            return Ok(Self::with_data_override(data_dir));
        }

        let project_dirs = ProjectDirs::from("io", "estudy", "eStudy")
            .ok_or(StorageError::UnavailableProjectDirs)?;

        Ok(Self {
            data_dir: project_dirs.data_local_dir().to_path_buf(),
            config_dir: project_dirs.config_dir().to_path_buf(),
            cache_dir: project_dirs.cache_dir().to_path_buf(),
        })
    }

    fn with_data_override(data_dir: PathBuf) -> Self {
        let fallback = ProjectDirs::from("io", "estudy", "eStudy");
        let config_dir = fallback.as_ref().map_or_else(
            || data_dir.join("config"),
            |dirs| dirs.config_dir().to_path_buf(),
        );
        let cache_dir = fallback.as_ref().map_or_else(
            || data_dir.join("cache"),
            |dirs| dirs.cache_dir().to_path_buf(),
        );

        Self {
            data_dir,
            config_dir,
            cache_dir,
        }
    }

    /// Cria um layout totalmente isolado sob uma raiz única (ideal para testes e `--e2e`).
    #[must_use]
    pub fn from_sandbox_root(root: &Path) -> Self {
        Self {
            data_dir: root.join("data"),
            config_dir: root.join("config"),
            cache_dir: root.join("cache"),
        }
    }

    /// Garante que todas as pastas necessárias existam no disco.
    pub fn ensure_dirs(&self) -> Result<(), StorageError> {
        for dir in [
            &self.data_dir,
            &self.backups_dir(),
            &self.config_dir,
            &self.cache_dir,
            &self.covers_cache_dir(),
            &self.logs_dir(),
        ] {
            std::fs::create_dir_all(dir).map_err(|source| StorageError::Io {
                path: dir.clone(),
                source,
            })?;
        }
        Ok(())
    }

    /// Caminho do banco `SQLite` principal (`estudy.db`).
    #[must_use]
    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("estudy.db")
    }

    /// Diretório onde ficam armazenados os backups automáticos pré-migração.
    #[must_use]
    pub fn backups_dir(&self) -> PathBuf {
        self.data_dir.join("backups")
    }

    /// Caminho do arquivo de configuração local (`config.toml`).
    #[must_use]
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Diretório de cache de capas em WebP.
    #[must_use]
    pub fn covers_cache_dir(&self) -> PathBuf {
        self.cache_dir.join("covers")
    }

    /// Diretório de logs locais descartáveis.
    #[must_use]
    pub fn logs_dir(&self) -> PathBuf {
        self.cache_dir.join("logs")
    }
}
