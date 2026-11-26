//! Testes de integração da migração `SQLite` v2 (`weekly_turns`, Hábitos e Segundo Cérebro).

use chrono::{NaiveDate, TimeZone, Utc};
use estudy_domain::{
    EntityId, Habit, KnowledgeNote, NoteKind, Project, ProjectFramework, ProjectStatus,
};
use estudy_storage::{
    LATEST_SCHEMA_VERSION, SearchEntityKind, SqliteHabitsRepository, SqliteProjectRepository,
    SqliteSearchRepository, SqliteSecondBrainRepository, StorageError, apply_migrations,
    open_in_memory,
};
use std::collections::BTreeSet;

fn fixed_now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 25, 14, 0, 0)
        .single()
        .unwrap_or_default()
}

#[test]
fn should_apply_v2_migration_and_persist_weekly_turns_projects() -> Result<(), StorageError> {
    let mut conn = open_in_memory()?;
    let report = apply_migrations(&mut conn, None, fixed_now())?;

    assert_eq!(report.current_version, LATEST_SCHEMA_VERSION);
    assert_eq!(LATEST_SCHEMA_VERSION, 2);
    assert_eq!(report.applied_versions, vec![1, 2]);

    let project = Project {
        id: EntityId::new_v7(),
        title: "Rotina Semanal".to_string(),
        description: "Tarefas divididas por turno".to_string(),
        color_token: "sage".to_string(),
        status: ProjectStatus::InProgress,
        framework: ProjectFramework::WeeklyTurns,
        sort_order: 0,
        created_at: fixed_now(),
        updated_at: fixed_now(),
    };
    SqliteProjectRepository::save_project(&mut conn, &project, None)?;

    let loaded = SqliteProjectRepository::list_projects(&conn)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].framework, ProjectFramework::WeeklyTurns);
    Ok(())
}

#[test]
fn should_persist_habits_and_logs_and_soft_delete() -> Result<(), StorageError> {
    let mut conn = open_in_memory()?;
    apply_migrations(&mut conn, None, fixed_now())?;

    let d1 = NaiveDate::from_ymd_opt(2026, 11, 24).unwrap_or_default();
    let d2 = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap_or_default();
    let mut dates = BTreeSet::new();
    dates.insert(d1);
    dates.insert(d2);

    let habit = Habit {
        id: EntityId::new_v7(),
        title: "Leitura técnica".to_string(),
        subtitle: "20 páginas por dia".to_string(),
        color_index: 2,
        weekly_goal: 5,
        completed_dates: dates,
    };

    SqliteHabitsRepository::save_habit(&mut conn, &habit, fixed_now())?;

    let loaded = SqliteHabitsRepository::list_habits(&conn)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].title, "Leitura técnica");
    assert_eq!(loaded[0].completed_dates.len(), 2);
    assert!(loaded[0].is_done_on(d2));

    SqliteHabitsRepository::delete_habit(&conn, &habit.id, fixed_now())?;
    let after_delete = SqliteHabitsRepository::list_habits(&conn)?;
    assert!(after_delete.is_empty());
    Ok(())
}

#[test]
fn should_persist_second_brain_notes_and_index_in_fts5() -> Result<(), StorageError> {
    let mut conn = open_in_memory()?;
    apply_migrations(&mut conn, None, fixed_now())?;

    let note = KnowledgeNote {
        id: EntityId::new_v7(),
        title: "Arquitetura Local-First em Rust".to_string(),
        kind: NoteKind::Reference,
        body_md: "SQLite em modo WAL com FTS5 garante busca instantânea offline.".to_string(),
        source_url: Some("https://sqlite.org/fts5.html".to_string()),
        tags: vec!["rust".to_string(), "sqlite".to_string()],
        pinned: true,
        created_at: fixed_now(),
        updated_at: fixed_now(),
    };

    SqliteSecondBrainRepository::save_note(&conn, &note)?;

    let loaded = SqliteSecondBrainRepository::list_notes(&conn)?;
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].kind, NoteKind::Reference);
    assert_eq!(loaded[0].tags, vec!["rust", "sqlite"]);
    assert!(loaded[0].pinned);

    let hits = SqliteSearchRepository::search_all(&conn, "instantânea", 10)?;
    assert!(
        hits.iter()
            .any(|h| h.kind == SearchEntityKind::Note && h.id == note.id.to_string())
    );
    Ok(())
}
