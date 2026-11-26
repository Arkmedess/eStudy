//! Repositório `SQLite` para o Segundo Cérebro reaproveitando `notes` + `notes_fts` (`FTS5`).

use chrono::{DateTime, SecondsFormat, Utc};
use estudy_domain::{EntityId, KnowledgeNote, NoteKind};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::error::StorageError;

#[derive(Debug, Default, Serialize, Deserialize)]
struct BrainExtra {
    #[serde(default)]
    brain_kind: Option<String>,
    #[serde(default)]
    source_url: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// Operações de persistência de notas do Segundo Cérebro sincronizadas com `FTS5`.
pub struct SqliteSecondBrainRepository;

impl SqliteSecondBrainRepository {
    /// Carrega notas ativas do Segundo Cérebro ordenadas por fixadas e mais recentes.
    pub fn list_notes(conn: &Connection) -> Result<Vec<KnowledgeNote>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, title, body_md, pinned, extra_json, created_at, updated_at
             FROM notes
             WHERE deleted_at IS NULL AND kind = 'free'
             ORDER BY pinned DESC, updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut notes = Vec::new();
        for row in rows {
            let (raw_id, title, body_md, pinned, extra_raw, created_raw, updated_raw) = row?;
            let id = EntityId::parse(&raw_id)?;
            let extra: BrainExtra = serde_json::from_str(&extra_raw).unwrap_or_default();
            let kind = parse_note_kind(extra.brain_kind.as_deref());
            let created_at = DateTime::parse_from_rfc3339(&created_raw)
                .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc));
            let updated_at = DateTime::parse_from_rfc3339(&updated_raw)
                .map_or_else(|_| created_at, |dt| dt.with_timezone(&Utc));

            notes.push(KnowledgeNote {
                id,
                title,
                kind,
                body_md,
                source_url: extra.source_url,
                tags: extra.tags,
                pinned: pinned != 0,
                created_at,
                updated_at,
            });
        }

        Ok(notes)
    }

    /// Salva (upsert) uma nota na tabela `notes` mantendo `notes_fts` atualizado via triggers.
    pub fn save_note(conn: &Connection, note: &KnowledgeNote) -> Result<(), StorageError> {
        let id_str = note.id.to_string();
        let slug = format!("brain-{id_str}");
        let summary = note.preview_excerpt(160);
        let extra = BrainExtra {
            brain_kind: Some(note_kind_as_str(note.kind).to_string()),
            source_url: note.source_url.clone(),
            tags: note.tags.clone(),
        };
        let extra_json = serde_json::to_string(&extra).unwrap_or_else(|_| "{}".to_string());
        let created_iso = note.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let updated_iso = note.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        let pinned_i64 = i64::from(note.pinned);

        conn.execute(
            "INSERT INTO notes (id, slug, title, kind, summary, body_md, pinned, extra_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'free', ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                summary = excluded.summary,
                body_md = excluded.body_md,
                pinned = excluded.pinned,
                extra_json = excluded.extra_json,
                updated_at = excluded.updated_at,
                deleted_at = NULL",
            params![
                id_str,
                slug,
                note.title,
                summary,
                note.body_md,
                pinned_i64,
                extra_json,
                created_iso,
                updated_iso
            ],
        )?;
        Ok(())
    }

    /// Remove (soft delete) uma nota do Segundo Cérebro.
    pub fn delete_note(
        conn: &Connection,
        note_id: &EntityId,
        now: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let now_iso = now.to_rfc3339_opts(SecondsFormat::Secs, true);
        conn.execute(
            "UPDATE notes SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now_iso, note_id.to_string()],
        )?;
        Ok(())
    }
}

const fn note_kind_as_str(kind: NoteKind) -> &'static str {
    match kind {
        NoteKind::Note => "note",
        NoteKind::Idea => "idea",
        NoteKind::Reference => "reference",
        NoteKind::Link => "link",
    }
}

fn parse_note_kind(raw: Option<&str>) -> NoteKind {
    match raw {
        Some("idea") => NoteKind::Idea,
        Some("reference") => NoteKind::Reference,
        Some("link") => NoteKind::Link,
        _ => NoteKind::Note,
    }
}
