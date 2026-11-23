//! Identificadores ordenáveis baseados em `UUIDv7`.

use std::fmt;
use uuid::Uuid;

use crate::error::DomainError;

/// Identificador único cronologicamente ordenável (`UUIDv7`).
///
/// Garante paginação por cursor eficiente no `SQLite` e evita colisões em sincronizações futuras.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Gera um novo identificador `UUIDv7` usando o relógio do sistema.
    #[must_use]
    pub fn new_v7() -> Self {
        Self(Uuid::now_v7())
    }

    /// Converte uma string UUID válida em [`EntityId`].
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let parsed = Uuid::parse_str(raw.trim())
            .map_err(|err| DomainError::InvalidEntityId(err.to_string()))?;
        Ok(Self(parsed))
    }

    /// Retorna o UUID interno subjacente.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
