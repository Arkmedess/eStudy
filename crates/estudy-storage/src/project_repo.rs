//! Repositório transacional para Projetos, Tarefas, Checklists (`0/20`) e Tags.

use chrono::{DateTime, SecondsFormat, Utc};
use estudy_domain::{
    DomainError, DomainEvent, EisenhowerQuadrant, EntityId, EventBusPort, MAX_TASK_CHECKLIST_ITEMS,
    Project, ProjectFramework, ProjectStatus, Task, TaskChecklistItem, TaskPriority,
    TaskProjectLink, TaskStatus, validate_focus_pin_budget,
};
use rusqlite::{Connection, Transaction, params};
use std::collections::HashMap;

use crate::error::StorageError;

/// Gerencia persistência de projetos, tarefas multi-projeto, checklists e tags no `SQLite`.
pub struct SqliteProjectRepository;

impl SqliteProjectRepository {
    /// Cria ou atualiza um projeto dentro de uma transação única.
    pub fn save_project(
        conn: &mut Connection,
        project: &Project,
        event_bus: Option<&dyn EventBusPort>,
    ) -> Result<(), StorageError> {
        let tx = conn.transaction()?;
        let created_at = project
            .created_at
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        let updated_at = project
            .updated_at
            .to_rfc3339_opts(SecondsFormat::Secs, true);

        tx.execute(
            "INSERT INTO projects (id, title, description, color_token, status, framework, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                color_token = excluded.color_token,
                status = excluded.status,
                framework = excluded.framework,
                sort_order = excluded.sort_order,
                updated_at = excluded.updated_at,
                deleted_at = NULL",
            params![
                project.id.to_string(),
                project.title,
                project.description,
                project.color_token,
                project.status.as_str(),
                project.framework.as_str(),
                project.sort_order,
                created_at,
                updated_at,
            ],
        )?;
        tx.commit()?;

        if let Some(bus) = event_bus {
            bus.publish(DomainEvent::ProjectSaved {
                project_id: project.id.clone(),
            });
        }
        Ok(())
    }

    /// Lista todos os projetos ativos ordenados por `sort_order`.
    pub fn list_projects(conn: &Connection) -> Result<Vec<Project>, StorageError> {
        let mut stmt = conn.prepare(
            "SELECT id, title, description, color_token, status, framework, sort_order, created_at, updated_at
             FROM projects
             WHERE deleted_at IS NULL
             ORDER BY sort_order ASC, created_at ASC",
        )?;

        let rows = stmt.query_map([], map_project_row)?;
        let mut projects = Vec::new();
        for item in rows {
            projects.push(item??);
        }
        Ok(projects)
    }

    /// Atualiza o framework selecionado para um projeto específico.
    pub fn update_project_framework(
        conn: &mut Connection,
        project_id: &EntityId,
        framework: ProjectFramework,
        updated_at: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let ts = updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        conn.execute(
            "UPDATE projects SET framework = ?1, updated_at = ?2 WHERE id = ?3 AND deleted_at IS NULL",
            params![framework.as_str(), ts, project_id.to_string()],
        )?;
        Ok(())
    }

    /// Salva uma tarefa completa (projetos vinculados, tags e checklist `0/20`) em transação única.
    pub fn save_task(
        conn: &mut Connection,
        task: &Task,
        event_bus: Option<&dyn EventBusPort>,
    ) -> Result<(), StorageError> {
        if task.checklist.len() > MAX_TASK_CHECKLIST_ITEMS {
            return Err(StorageError::Domain(DomainError::ChecklistLimitReached {
                max: MAX_TASK_CHECKLIST_ITEMS,
            }));
        }

        let tx = conn.transaction()?;
        enforce_focus_pin_limit_in_tx(&tx, task)?;
        upsert_task_row(&tx, task)?;
        sync_project_links(&tx, task)?;
        sync_task_tags(&tx, task)?;
        sync_task_checklist(&tx, task)?;
        tx.commit()?;

        if let Some(bus) = event_bus {
            bus.publish(DomainEvent::TaskSaved {
                task_id: task.id.clone(),
            });
        }
        Ok(())
    }

    /// Carrega todas as tarefas ativas com vínculos, tags e checklist sem consultas N+1.
    pub fn list_tasks(conn: &Connection) -> Result<Vec<Task>, StorageError> {
        let mut links_map = load_all_project_links(conn)?;
        let mut tags_map = load_all_task_tags(conn)?;
        let mut checklist_map = load_all_checklists(conn)?;

        let mut stmt = conn.prepare(
            "SELECT id, title, description_md, status, priority, eisenhower_quadrant,
                    pinned_focus, start_at, due_at, recurrence_json, estimated_pomos,
                    completed_pomos, created_at, updated_at
             FROM tasks
             WHERE deleted_at IS NULL
             ORDER BY sort_order ASC, created_at ASC",
        )?;

        let rows = stmt.query_map([], map_task_base_row)?;
        let mut tasks = Vec::new();
        for item in rows {
            let mut task = item??;
            let key = task.id.to_string();
            task.project_links = links_map.remove(&key).unwrap_or_default();
            task.tags = tags_map.remove(&key).unwrap_or_default();
            task.checklist = checklist_map.remove(&key).unwrap_or_default();
            tasks.push(task);
        }
        Ok(tasks)
    }

    /// Marca uma tarefa como removida via soft delete (`deleted_at`).
    pub fn soft_delete_task(
        conn: &mut Connection,
        task_id: &EntityId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), StorageError> {
        let ts = deleted_at.to_rfc3339_opts(SecondsFormat::Secs, true);
        conn.execute(
            "UPDATE tasks SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![ts, task_id.to_string()],
        )?;
        Ok(())
    }
}

fn enforce_focus_pin_limit_in_tx(tx: &Transaction<'_>, task: &Task) -> Result<(), StorageError> {
    if !task.pinned_focus {
        return Ok(());
    }
    let task_id = task.id.to_string();
    let already_pinned: i64 = tx.query_row(
        "SELECT COUNT(*) FROM tasks WHERE id = ?1 AND pinned_focus = 1 AND deleted_at IS NULL",
        [&task_id],
        |row| row.get(0),
    )?;
    let pinned_count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM tasks WHERE pinned_focus = 1 AND deleted_at IS NULL",
        [],
        |row| row.get(0),
    )?;
    let count_usize = usize::try_from(pinned_count).unwrap_or(usize::MAX);
    validate_focus_pin_budget(count_usize, task.pinned_focus, already_pinned > 0)?;
    Ok(())
}

fn upsert_task_row(tx: &Transaction<'_>, task: &Task) -> Result<(), StorageError> {
    let created_at = task.created_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let updated_at = task.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let completed_at = (task.status == TaskStatus::Done).then_some(updated_at.as_str());
    let quadrant = task.eisenhower_quadrant.map(EisenhowerQuadrant::as_str);

    tx.execute(
        "INSERT INTO tasks (id, title, description_md, status, priority, eisenhower_quadrant,
                            pinned_focus, start_at, due_at, recurrence_json, estimated_pomos,
                            completed_pomos, completed_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            description_md = excluded.description_md,
            status = excluded.status,
            priority = excluded.priority,
            eisenhower_quadrant = excluded.eisenhower_quadrant,
            pinned_focus = excluded.pinned_focus,
            start_at = excluded.start_at,
            due_at = excluded.due_at,
            recurrence_json = excluded.recurrence_json,
            estimated_pomos = excluded.estimated_pomos,
            completed_pomos = excluded.completed_pomos,
            completed_at = excluded.completed_at,
            updated_at = excluded.updated_at,
            deleted_at = NULL",
        params![
            task.id.to_string(),
            task.title,
            task.description_md,
            task.status.as_str(),
            task.priority.as_str(),
            quadrant,
            i64::from(task.pinned_focus),
            task.start_at,
            task.due_at,
            task.recurrence,
            task.estimated_pomos,
            task.completed_pomos,
            completed_at,
            created_at,
            updated_at,
        ],
    )?;
    Ok(())
}

fn sync_project_links(tx: &Transaction<'_>, task: &Task) -> Result<(), StorageError> {
    let task_id = task.id.to_string();
    tx.execute("DELETE FROM project_tasks WHERE task_id = ?1", [&task_id])?;
    for (idx, link) in task.project_links.iter().enumerate() {
        let sort_order = i32::try_from(idx).unwrap_or(i32::MAX);
        tx.execute(
            "INSERT INTO project_tasks (project_id, task_id, milestone_name, sort_order) VALUES (?1, ?2, ?3, ?4)",
            params![link.project_id.to_string(), &task_id, link.milestone_name, sort_order],
        )?;
    }
    Ok(())
}

fn sync_task_tags(tx: &Transaction<'_>, task: &Task) -> Result<(), StorageError> {
    let task_id = task.id.to_string();
    let now = task.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    tx.execute(
        "DELETE FROM entity_tags WHERE entity_type = 'task' AND entity_id = ?1",
        [&task_id],
    )?;

    for raw_tag in &task.tags {
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
            "INSERT OR IGNORE INTO entity_tags (tag_id, entity_type, entity_id) VALUES (?1, 'task', ?2)",
            params![tag_id, &task_id],
        )?;
    }
    Ok(())
}

fn sync_task_checklist(tx: &Transaction<'_>, task: &Task) -> Result<(), StorageError> {
    let task_id = task.id.to_string();
    let now = task.updated_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    tx.execute(
        "DELETE FROM task_checklist_items WHERE task_id = ?1",
        [&task_id],
    )?;

    for item in &task.checklist {
        tx.execute(
            "INSERT INTO task_checklist_items (id, task_id, title, done, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                item.id.to_string(),
                &task_id,
                item.title,
                i64::from(item.done),
                item.sort_order,
                &now,
            ],
        )?;
    }
    Ok(())
}

fn load_all_project_links(
    conn: &Connection,
) -> Result<HashMap<String, Vec<TaskProjectLink>>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT task_id, project_id, milestone_name FROM project_tasks ORDER BY sort_order ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let task_id: String = row.get(0)?;
        let project_id_raw: String = row.get(1)?;
        let milestone_name: Option<String> = row.get(2)?;
        Ok((task_id, project_id_raw, milestone_name))
    })?;

    let mut grouped: HashMap<String, Vec<TaskProjectLink>> = HashMap::new();
    for item in rows {
        let (task_id, project_id_raw, milestone_name) = item?;
        let project_id = EntityId::parse(&project_id_raw)?;
        grouped.entry(task_id).or_default().push(TaskProjectLink {
            project_id,
            milestone_name,
        });
    }
    Ok(grouped)
}

fn load_all_task_tags(conn: &Connection) -> Result<HashMap<String, Vec<String>>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT et.entity_id, t.name
         FROM entity_tags et
         INNER JOIN tags t ON t.id = et.tag_id
         WHERE et.entity_type = 'task'
         ORDER BY t.name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let task_id: String = row.get(0)?;
        let name: String = row.get(1)?;
        Ok((task_id, name))
    })?;

    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();
    for item in rows {
        let (task_id, name) = item?;
        grouped.entry(task_id).or_default().push(name);
    }
    Ok(grouped)
}

fn load_all_checklists(
    conn: &Connection,
) -> Result<HashMap<String, Vec<TaskChecklistItem>>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, title, done, sort_order
         FROM task_checklist_items
         ORDER BY sort_order ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let id_raw: String = row.get(0)?;
        let task_id: String = row.get(1)?;
        let title: String = row.get(2)?;
        let done: i64 = row.get(3)?;
        let sort_order: i32 = row.get(4)?;
        Ok((id_raw, task_id, title, done != 0, sort_order))
    })?;

    let mut grouped: HashMap<String, Vec<TaskChecklistItem>> = HashMap::new();
    for item in rows {
        let (id_raw, task_id, title, done, sort_order) = item?;
        let id = EntityId::parse(&id_raw)?;
        grouped.entry(task_id).or_default().push(TaskChecklistItem {
            id,
            title,
            done,
            sort_order,
        });
    }
    Ok(grouped)
}

fn map_project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<Project, StorageError>> {
    let id_raw: String = row.get(0)?;
    let title: String = row.get(1)?;
    let description: String = row.get(2)?;
    let color_token: String = row.get(3)?;
    let status_raw: String = row.get(4)?;
    let framework_raw: String = row.get(5)?;
    let sort_order: i32 = row.get(6)?;
    let created_raw: String = row.get(7)?;
    let updated_raw: String = row.get(8)?;

    Ok((|| {
        Ok(Project {
            id: EntityId::parse(&id_raw)?,
            title,
            description,
            color_token,
            status: ProjectStatus::parse(&status_raw)?,
            framework: ProjectFramework::parse(&framework_raw)?,
            sort_order,
            created_at: parse_ts(&created_raw)?,
            updated_at: parse_ts(&updated_raw)?,
        })
    })())
}

fn map_task_base_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<Task, StorageError>> {
    let id_raw: String = row.get(0)?;
    let title: String = row.get(1)?;
    let description_md: String = row.get(2)?;
    let status_raw: String = row.get(3)?;
    let priority_raw: String = row.get(4)?;
    let quadrant_raw: Option<String> = row.get(5)?;
    let pinned_focus: i64 = row.get(6)?;
    let start_at: Option<String> = row.get(7)?;
    let due_at: Option<String> = row.get(8)?;
    let recurrence: Option<String> = row.get(9)?;
    let estimated_pomos: i32 = row.get(10)?;
    let completed_pomos: i32 = row.get(11)?;
    let created_raw: String = row.get(12)?;
    let updated_raw: String = row.get(13)?;

    Ok((|| {
        let quadrant = quadrant_raw
            .as_deref()
            .map(EisenhowerQuadrant::parse)
            .transpose()?;
        Ok(Task {
            id: EntityId::parse(&id_raw)?,
            title,
            description_md,
            status: TaskStatus::parse(&status_raw)?,
            priority: TaskPriority::parse(&priority_raw)?,
            eisenhower_quadrant: quadrant,
            pinned_focus: pinned_focus != 0,
            start_at,
            due_at,
            recurrence,
            estimated_pomos,
            completed_pomos,
            project_links: Vec::new(),
            tags: Vec::new(),
            checklist: Vec::new(),
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
