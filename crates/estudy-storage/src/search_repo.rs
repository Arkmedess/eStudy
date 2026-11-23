//! Busca global unificada (`FTS5` + entidades ativas) para a Barra de Comando `Ctrl+K`.

use rusqlite::{Connection, params};

use crate::error::StorageError;

/// Categoria de item retornado pela pesquisa global.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchEntityKind {
    /// Projeto.
    Project,
    /// Tarefa.
    Task,
    /// Evento de calendário.
    Event,
    /// Nota indexada via `FTS5`.
    Note,
    /// Disciplina de estudo.
    Subject,
    /// Livro da estante.
    Book,
}

/// Resultado retornado pela pesquisa global no `SQLite`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSearchHit {
    /// Tipo da entidade encontrada.
    pub kind: SearchEntityKind,
    /// Identificador `UUIDv7` da entidade.
    pub id: String,
    /// Título principal exibido no resultado.
    pub title: String,
    /// Subtítulo contextual (status, resumo ou autor).
    pub subtitle: String,
}

/// Executa pesquisa global unificada combinando `FTS5` e entidades ativas do banco.
pub struct SqliteSearchRepository;

impl SqliteSearchRepository {
    /// Busca por termo em notas (`FTS5`), projetos, tarefas, eventos, disciplinas e livros.
    pub fn search_all(
        conn: &Connection,
        query: &str,
        limit: usize,
    ) -> Result<Vec<GlobalSearchHit>, StorageError> {
        let started = std::time::Instant::now();
        let normalized_query = fold_accents_lowercase(query);
        if normalized_query.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let mut hits = Vec::new();
        if let Some(fts_expr) = build_fts5_prefix_query(query) {
            search_notes_fts(conn, &fts_expr, limit, &mut hits)?;
        }

        let table_queries: &[(&str, SearchEntityKind, &str)] = &[
            (
                "SELECT id, title, status FROM projects WHERE deleted_at IS NULL",
                SearchEntityKind::Project,
                "Projeto",
            ),
            (
                "SELECT id, title, status FROM tasks WHERE deleted_at IS NULL",
                SearchEntityKind::Task,
                "Tarefa",
            ),
            (
                "SELECT id, title, starts_at FROM calendar_events WHERE deleted_at IS NULL",
                SearchEntityKind::Event,
                "Evento",
            ),
            (
                "SELECT id, name, 'Estudos' FROM study_subjects WHERE deleted_at IS NULL",
                SearchEntityKind::Subject,
                "Disciplina",
            ),
            (
                "SELECT id, title, author FROM books WHERE deleted_at IS NULL",
                SearchEntityKind::Book,
                "Livro",
            ),
        ];

        for &(sql, kind, fallback_sub) in table_queries {
            if hits.len() >= limit {
                break;
            }
            search_active_table(
                conn,
                sql,
                kind,
                fallback_sub,
                &normalized_query,
                limit,
                &mut hits,
            )?;
        }

        hits.truncate(limit);
        let elapsed_us = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
        tracing::debug!(
            target: "estudy.storage",
            query_len = normalized_query.chars().count(),
            hit_count = hits.len(),
            elapsed_us,
            "busca global unificada executada"
        );
        Ok(hits)
    }
}

fn search_notes_fts(
    conn: &Connection,
    fts_expr: &str,
    limit: usize,
    out: &mut Vec<GlobalSearchHit>,
) -> Result<(), StorageError> {
    let limit_i64 = i64::try_from(limit).unwrap_or(50);
    let mut stmt = conn.prepare(
        "SELECT n.id, n.title, n.summary
         FROM notes_fts
         INNER JOIN notes n ON n.rowid = notes_fts.rowid
         WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
         ORDER BY bm25(notes_fts) ASC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![fts_expr, limit_i64], |row| {
        Ok(GlobalSearchHit {
            kind: SearchEntityKind::Note,
            id: row.get(0)?,
            title: row.get(1)?,
            subtitle: row.get(2)?,
        })
    })?;

    for row in rows {
        out.push(row?);
    }
    Ok(())
}

fn search_active_table(
    conn: &Connection,
    sql: &str,
    kind: SearchEntityKind,
    fallback_sub: &str,
    needle: &str,
    limit: usize,
    out: &mut Vec<GlobalSearchHit>,
) -> Result<(), StorageError> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let extra: String = row.get(2)?;
        Ok((id, title, extra))
    })?;

    for row in rows {
        if out.len() >= limit {
            break;
        }
        let (id, title, extra) = row?;
        let hay_title = fold_accents_lowercase(&title);
        let hay_extra = fold_accents_lowercase(&extra);
        if hay_title.contains(needle) || hay_extra.contains(needle) {
            let subtitle = if extra.trim().is_empty() {
                fallback_sub.to_string()
            } else {
                format!("{fallback_sub} · {extra}")
            };
            out.push(GlobalSearchHit {
                kind,
                id,
                title,
                subtitle,
            });
        }
    }
    Ok(())
}

fn build_fts5_prefix_query(raw: &str) -> Option<String> {
    let tokens: Vec<String> = raw
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| format!("\"{part}\"*"))
        .collect();

    if tokens.is_empty() {
        None
    } else {
        Some(tokens.join(" AND "))
    }
}

fn fold_accents_lowercase(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| match ch {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}
