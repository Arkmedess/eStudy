//! Regras puras do Segundo Cérebro (notas, ideias, referências e links).

use chrono::{DateTime, Utc};

use crate::ids::EntityId;

/// Tipo de artefato armazenado no Segundo Cérebro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteKind {
    /// Nota estruturada ou resumo de estudo.
    Note,
    /// Ideia rápida ou rascunho criativo.
    Idea,
    /// Referência bibliográfica, trecho ou modelo.
    Reference,
    /// Link externo comentado.
    Link,
}

impl NoteKind {
    /// Todos os tipos em ordem de exibição.
    pub const ALL: [Self; 4] = [Self::Note, Self::Idea, Self::Reference, Self::Link];

    /// Rótulo legível em português.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Note => "Nota",
            Self::Idea => "Ideia",
            Self::Reference => "Referência",
            Self::Link => "Link",
        }
    }

    /// Rótulo plural para abas de filtro.
    #[must_use]
    pub const fn plural_label(self) -> &'static str {
        match self {
            Self::Note => "Notas",
            Self::Idea => "Ideias",
            Self::Reference => "Referências",
            Self::Link => "Links",
        }
    }
}

/// Cartão de conhecimento do Segundo Cérebro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeNote {
    /// Identificador cronológico (`UUIDv7`).
    pub id: EntityId,
    /// Título conciso da nota.
    pub title: String,
    /// Tipo do registro.
    pub kind: NoteKind,
    /// Conteúdo em Markdown.
    pub body_md: String,
    /// URL opcional (especialmente útil para `NoteKind::Link` e `Reference`).
    pub source_url: Option<String>,
    /// Tags de organização (ex.: `rust`, `estudos`, `design`).
    pub tags: Vec<String>,
    /// Fixada no topo da grade.
    pub pinned: bool,
    /// Data de criação UTC.
    pub created_at: DateTime<Utc>,
    /// Última atualização UTC.
    pub updated_at: DateTime<Utc>,
}

impl KnowledgeNote {
    /// Retorna um trecho curto limpo para visualização em cartões.
    #[must_use]
    pub fn preview_excerpt(&self, max_chars: usize) -> String {
        let single_line = self
            .body_md
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if single_line.chars().count() <= max_chars {
            return single_line;
        }
        let mut cut: String = single_line
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect();
        cut.push('…');
        cut
    }

    /// Verifica se a nota corresponde aos filtros ativos de tipo, tag e busca textual.
    #[must_use]
    pub fn matches_filter(
        &self,
        kind_filter: Option<NoteKind>,
        tag_filter: Option<&str>,
        query: &str,
    ) -> bool {
        if let Some(expected) = kind_filter
            && self.kind != expected
        {
            return false;
        }
        if let Some(tag) = tag_filter
            && !self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
        {
            return false;
        }
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.title.to_lowercase().contains(&q)
            || self.body_md.to_lowercase().contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
    }
}
