//! Repositório transacional para eventos de calendário e tags vinculadas.

use chrono::{DateTime, SecondsFormat, Utc};
use estudy_domain::{CalendarEvent, DomainEvent, EntityId, EventBusPort, RecurrenceRule};
use rusqlite::{Connection, Transaction, params};
use std::collections::HashMap;

use crate::error::StorageError;

/// Persiste e consulta eventos recorrentes e multi-dia no `SQLite`.
pub struct SqliteCalendarRepository;

impl SqliteCalendarRepository {
    /// Valida e salva um evento de calendário com suas tags em transação única.
    pub fn save_event(
        conn: &mut Connection,
        event: &CalendarEvent,
        event_bus: Option<&dyn EventBusPort>,
    ) -> Result<(), StorageError> {
        event.validate()?;

        let tx = conn.transaction()?;
        upsert_event_row(&tx, event)?;
        sync_event_tags(&tx, event)?;
        tx.commit()?;

        if let Some(bus) = event_bus {
            bus.publish(DomainEvent::CalendarEventSaved {
                event_id: event.id.clone(),
            });
        }
        Ok(())
    }

    /// Carrega todos os eventos ativos ordenados por início sem consultas N+1.
    pub fn list_events(conn: &Connection) -> Result<Vec<CalendarEvent>, StorageError> {
        let mut tags_map = load_all_event_tags(conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, title, description, project_id, starts_at, ends_at,
                    all_day, recurrence_json, color_token, created_at, updated_at
             FROM calendar_events
             WHERE deleted_at IS NULL
             ORDER BY starts_at ASC",
        )?;

        let rows = stmt.query_map([], map_event_row)?;
        let mut events = Vec::new();
        for item in rows {
            let mut event = item??;
            let key = event.id.to_string();
            event.tags = tags_map.remove(&key).unwrap_or_default();
            events.push(event);
        }
        Ok(events)
    }

    /// Remove um evento logicamente via `deleted_at`.
    pub fn soft_delete_event(
        conn: &mut Connection,
        event_id: &EntityId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let ts = deleted_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        conn.execute(
            "UPDATE calendar_events SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![ts, event_id.to_string()],
        )?;
        Ok(())
    }
}

fn upsert_event_row(tx: &Transaction<'_>, event: &CalendarEvent) -> Result<(), StorageError> {
    let starts_at = event.starts_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let ends_at = event.ends_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let created_at = event.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let updated_at = event.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let project_id = event.project_id.as_ref().map(ToString::to_string);
    let recurrence = event.recurrence.map(RecurrenceRule::as_str);

    tx.execute(
        "INSERT INTO calendar_events (id, title, description, project_id, starts_at, ends_at,
                                      all_day, recurrence_json, color_token, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            description = excluded.description,
            project_id = excluded.project_id,
            starts_at = excluded.starts_at,
            ends_at = excluded.ends_at,
            all_day = excluded.all_day,
            recurrence_json = excluded.recurrence_json,
            color_token = excluded.color_token,
            updated_at = excluded.updated_at,
            deleted_at = NULL",
        params![
            event.id.to_string(),
            event.title,
            event.description,
            project_id,
            starts_at,
            ends_at,
            i64::from(event.all_day),
            recurrence,
            event.color_token,
            created_at,
            updated_at,
        ],
    )?;
    Ok(())
}

fn sync_event_tags(tx: &Transaction<'_>, event: &CalendarEvent) -> Result<(), StorageError> {
    let event_id = event.id.to_string();
    let now = event.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    tx.execute(
        "DELETE FROM entity_tags WHERE entity_type = 'event' AND entity_id = ?1",
        [&event_id],
    )?;

    for raw_tag in &event.tags {
        let name = raw_tag.trim();
        if name.is_empty() {
            continue;
        }
        let existing_id: Option<String> = tx
            .query_row(
                "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE",
                [name],
                |row| row.get(0),
            )
            .ok();

        let tag_id = if let Some(id) = existing_id {
            id
        } else {
            let new_id = EntityId::new_v7().to_string();
            tx.execute(
                "INSERT INTO tags (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![&new_id, name, &now],
            )?;
            new_id
        };

        tx.execute(
            "INSERT OR IGNORE INTO entity_tags (tag_id, entity_type, entity_id) VALUES (?1, 'event', ?2)",
            params![tag_id, &event_id],
        )?;
    }
    Ok(())
}

fn load_all_event_tags(conn: &Connection) -> Result<HashMap<String, Vec<String>>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT et.entity_id, t.name
         FROM entity_tags et
         INNER JOIN tags t ON t.id = et.tag_id
         WHERE et.entity_type = 'event'
         ORDER BY t.name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let event_id: String = row.get(0)?;
        let name: String = row.get(1)?;
        Ok((event_id, name))
    })?;

    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for item in rows {
        let (event_id, name) = item?;
        grouped.entry(event_id).or_default().push(name);
    }
    Ok(grouped)
}

fn map_event_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<CalendarEvent, StorageError>> {
    let id_raw: String = row.get(0)?;
    let title: String = row.get(1)?;
    let description: String = row.get(2)?;
    let project_id_raw: Option<String> = row.get(3)?;
    let starts_raw: String = row.get(4)?;
    let ends_raw: String = row.get(5)?;
    let all_day: i64 = row.get(6)?;
    let recurrence_raw: Option<String> = row.get(7)?;
    let color_token: String = row.get(8)?;
    let created_raw: String = row.get(9)?;
    let updated_raw: String = row.get(10)?;

    Ok((|| {
        let project_id = project_id_raw.as_deref().map(EntityId::parse).transpose()?;
        let recurrence = recurrence_raw
            .as_deref()
            .map(RecurrenceRule::parse)
            .transpose()?;

        Ok(CalendarEvent {
            id: EntityId::parse(&id_raw)?,
            title,
            description,
            project_id,
            starts_at: parse_ts(&starts_raw)?,
            ends_at: parse_ts(&ends_raw)?,
            all_day: all_day != 0,
            recurrence,
            color_token,
            tags: Vec::new(),
            created_at: parse_ts(&created_raw)?,
            updated_at: parse_ts(&updated_raw)?,
        })
    })())
}

fn parse_ts(raw: &str) -> Result<DateTime<Utc>, StorageError> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| StorageError::IntegrityCheckFailed(format!("timestamp inválido: {err}")))
}
