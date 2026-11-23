//! Testes de integração da camada de persistência `SQLite` v1 do eStudy.

#![allow(clippy::expect_used)]

use chrono::{DateTime, Duration, Utc};
use estudy_domain::{
    DomainError, DomainEvent, EntityId, InMemoryEventBus, LedgerEntryType, WalletEntryDraft,
};
use estudy_storage::{
    AppPaths, LATEST_SCHEMA_VERSION, MAX_PRE_MIGRATION_BACKUPS, SqliteWalletRepository,
    StorageError, apply_migrations, create_pre_migration_backup, export_clean_snapshot,
    open_connection, open_in_memory,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn fixed_now(offset_secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(1_760_000_000 + offset_secs, 0).expect("timestamp válido")
}

#[test]
fn should_resolve_override_paths_and_create_directories() {
    let temp = TempDir::new().expect("diretório temporário");
    let custom_data = temp.path().join("custom-data");

    let resolved = AppPaths::resolve(Some(&custom_data)).expect("resolução XDG com override");
    assert_eq!(resolved.data_dir, custom_data);
    assert_eq!(resolved.db_path(), custom_data.join("estudy.db"));
    assert_eq!(resolved.backups_dir(), custom_data.join("backups"));

    resolved.ensure_dirs().expect("criação de diretórios");
    assert!(resolved.data_dir.is_dir());
    assert!(resolved.backups_dir().is_dir());
    assert!(resolved.covers_cache_dir().is_dir());
    assert!(resolved.logs_dir().is_dir());
}

#[test]
fn should_apply_wal_pragmas_and_migrate_all_v1_tables_idempotently() {
    let temp = TempDir::new().expect("diretório temporário");
    let paths = AppPaths::from_sandbox_root(temp.path());
    paths.ensure_dirs().expect("diretórios criados");

    let mut conn = open_connection(&paths.db_path()).expect("conexão SQLite WAL");

    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .expect("pragma journal_mode");
    assert_eq!(journal_mode.to_lowercase(), "wal");

    let foreign_keys: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("pragma foreign_keys");
    assert_eq!(foreign_keys, 1);

    let cache_size: i64 = conn
        .query_row("PRAGMA cache_size", [], |row| row.get(0))
        .expect("pragma cache_size");
    assert_eq!(cache_size, -4000);

    let first_run =
        apply_migrations(&mut conn, Some(&paths.backups_dir()), fixed_now(0)).expect("migração v1");
    assert_eq!(first_run.applied_versions, vec![LATEST_SCHEMA_VERSION]);
    assert_eq!(first_run.current_version, LATEST_SCHEMA_VERSION);

    let second_run = apply_migrations(&mut conn, Some(&paths.backups_dir()), fixed_now(5))
        .expect("segunda execução idempotente");
    assert!(second_run.applied_versions.is_empty());
    assert_eq!(second_run.current_version, LATEST_SCHEMA_VERSION);

    let expected_tables = [
        "schema_migrations",
        "asset_cache",
        "local_profiles",
        "mood_logs",
        "projects",
        "tasks",
        "project_tasks",
        "task_checklist_items",
        "tags",
        "entity_tags",
        "calendar_events",
        "task_templates",
        "study_subjects",
        "study_topics",
        "study_sessions",
        "spaced_reviews",
        "study_planner_blocks",
        "flashcards",
        "error_notebook",
        "books",
        "bookshelves",
        "bookshelf_items",
        "book_reading_logs",
        "book_quotes",
        "reading_goals",
        "notes",
        "note_revisions",
        "notes_fts",
        "daily_checkins",
        "wallet_ledger",
        "inventory_items",
        "cosmetic_loadout",
        "garden_plantings",
        "pet_state",
        "achievement_progress",
        "user_preferences",
        "mcp_audit_log",
    ];

    for table in expected_tables {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .expect("consulta sqlite_master");
        assert_eq!(count, 1, "tabela esperada ausente no schema v1: {table}");
    }
}

#[test]
fn should_keep_notes_fts_synchronized_on_insert_update_and_delete() {
    let temp = TempDir::new().expect("diretório temporário");
    let paths = AppPaths::from_sandbox_root(temp.path());
    paths.ensure_dirs().expect("diretórios");

    let mut conn = open_connection(&paths.db_path()).expect("conexão WAL");
    apply_migrations(&mut conn, Some(&paths.backups_dir()), fixed_now(0)).expect("migração");

    let note_id = EntityId::new_v7().to_string();
    conn.execute(
        "INSERT INTO notes (id, slug, title, kind, summary, body_md, created_at, updated_at)
         VALUES (?1, 'revisao-calculo', 'Revisão de Cálculo', 'subject', 'Integrais definidas', 'Teorema fundamental da álgebra linear', '2026-11-22T10:00:00Z', '2026-11-22T10:00:00Z')",
        [&note_id],
    )
    .expect("inserir nota");

    let hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'calculo'",
            [],
            |row| row.get(0),
        )
        .expect("busca FTS sem acento");
    assert_eq!(hits, 1);

    conn.execute(
        "UPDATE notes SET body_md = 'Conteúdo atualizado de geometria analítica' WHERE id = ?1",
        [&note_id],
    )
    .expect("atualizar nota");

    let updated_hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'geometria'",
            [],
            |row| row.get(0),
        )
        .expect("busca FTS após update");
    assert_eq!(updated_hits, 1);

    conn.execute("DELETE FROM notes WHERE id = ?1", [&note_id])
        .expect("remover nota");
    let deleted_hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes_fts WHERE notes_fts MATCH 'geometria'",
            [],
            |row| row.get(0),
        )
        .expect("busca FTS após delete");
    assert_eq!(deleted_hits, 0);
}

#[test]
fn should_rotate_pre_migration_backups_and_export_clean_cloud_snapshot() {
    let temp = TempDir::new().expect("diretório temporário");
    let paths = AppPaths::from_sandbox_root(temp.path());
    paths.ensure_dirs().expect("diretórios");

    let mut conn = open_connection(&paths.db_path()).expect("conexão WAL");
    apply_migrations(&mut conn, Some(&paths.backups_dir()), fixed_now(0)).expect("migração");

    let profile_id = EntityId::new_v7().to_string();
    conn.execute(
        "INSERT INTO local_profiles (id, display_name, created_at, updated_at) VALUES (?1, 'Arthur', '2026-11-22T10:00:00Z', '2026-11-22T10:00:00Z')",
        [&profile_id],
    )
    .expect("inserir perfil");

    for step in 0..7_i64 {
        create_pre_migration_backup(
            &conn,
            &paths.backups_dir(),
            1,
            fixed_now(0) + Duration::seconds(step + 1),
            MAX_PRE_MIGRATION_BACKUPS,
        )
        .expect("backup pré-migração");
    }

    let backup_count = std::fs::read_dir(paths.backups_dir())
        .expect("ler diretório de backups")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "db"))
        .count();
    assert_eq!(backup_count, MAX_PRE_MIGRATION_BACKUPS);

    let cloud_snapshot = temp.path().join("cloud/estudy-snapshot.db");
    let exported = export_clean_snapshot(&conn, &cloud_snapshot).expect("exportar snapshot limpo");
    assert!(exported.is_file());
    assert!(!temp.path().join("cloud/estudy-snapshot.db-wal").exists());

    let snapshot_conn = Connection::open(&exported).expect("abrir snapshot isolado");
    let name: String = snapshot_conn
        .query_row(
            "SELECT display_name FROM local_profiles WHERE id = ?1",
            [&profile_id],
            |row| row.get(0),
        )
        .expect("ler perfil no snapshot");
    assert_eq!(name, "Arthur");
}

#[test]
fn should_record_wallet_entries_idempotently_and_detect_sql_tampering() {
    let temp = TempDir::new().expect("diretório temporário");
    let paths = AppPaths::from_sandbox_root(temp.path());
    paths.ensure_dirs().expect("diretórios");

    let mut conn = open_connection(&paths.db_path()).expect("conexão WAL");
    apply_migrations(&mut conn, Some(&paths.backups_dir()), fixed_now(0)).expect("migração");

    let profile_id = EntityId::new_v7();
    conn.execute(
        "INSERT INTO local_profiles (id, display_name, created_at, updated_at) VALUES (?1, 'Arthur', '2026-11-22T10:00:00Z', '2026-11-22T10:00:00Z')",
        [profile_id.to_string()],
    )
    .expect("inserir perfil");

    let bus = InMemoryEventBus::new();
    let app_secret = b"estudy-test-secret";

    let draft = WalletEntryDraft {
        id: EntityId::new_v7(),
        entry_type: LedgerEntryType::DailyCheckin,
        amount: 25,
        reference_id: "checkin:2026-11-22".to_string(),
        idempotency_key: "checkin-2026-11-22".to_string(),
        created_at: fixed_now(10),
    };

    let first = SqliteWalletRepository::record_entry(
        &mut conn,
        app_secret,
        &profile_id,
        draft.clone(),
        Some(&bus),
    )
    .expect("gravar entrada na carteira");

    let duplicate =
        SqliteWalletRepository::record_entry(&mut conn, app_secret, &profile_id, draft, Some(&bus))
            .expect("retry idempotente deve retornar lançamento existente");
    assert_eq!(first, duplicate);
    assert_eq!(bus.snapshot().len(), 1);
    assert!(matches!(
        bus.snapshot()[0],
        DomainEvent::WalletEntryRecorded {
            seq: 1,
            delta: 25,
            ..
        }
    ));

    let verified_balance =
        SqliteWalletRepository::verify_wallet_balance(&conn, app_secret, &profile_id)
            .expect("saldo verificado");
    assert_eq!(verified_balance, 25);

    conn.execute(
        "UPDATE wallet_ledger SET amount = 9999 WHERE id = ?1",
        [first.id.to_string()],
    )
    .expect("simular adulteração direta no SQLite");

    let tamper_err = SqliteWalletRepository::verify_wallet_balance(&conn, app_secret, &profile_id)
        .expect_err("adulteração direta deve invalidar a cadeia HMAC");
    assert!(matches!(
        tamper_err,
        StorageError::Domain(DomainError::WalletChainTampered { seq: 1, .. })
    ));
}

#[test]
fn should_support_in_memory_db_for_chained_credits_and_debits() {
    let mut conn = open_in_memory().expect("banco em memória");
    apply_migrations(&mut conn, None, fixed_now(0)).expect("migração em memória");

    let profile_id = EntityId::new_v7();
    conn.execute(
        "INSERT INTO local_profiles (id, display_name, created_at, updated_at) VALUES (?1, 'Tester', '2026-11-22T10:00:00Z', '2026-11-22T10:00:00Z')",
        [profile_id.to_string()],
    )
    .expect("inserir perfil");

    let secret = b"memory-secret";
    SqliteWalletRepository::record_entry(
        &mut conn,
        secret,
        &profile_id,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::PomodoroFocus,
            amount: 60,
            reference_id: "pomo:1".to_string(),
            idempotency_key: "pomo-1".to_string(),
            created_at: fixed_now(1),
        },
        None,
    )
    .expect("crédito pomodoro");

    SqliteWalletRepository::record_entry(
        &mut conn,
        secret,
        &profile_id,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::ShopPurchase,
            amount: -25,
            reference_id: "frame:japandi".to_string(),
            idempotency_key: "buy-frame-japandi".to_string(),
            created_at: fixed_now(2),
        },
        None,
    )
    .expect("débito loja");

    let balance = SqliteWalletRepository::verify_wallet_balance(&conn, secret, &profile_id)
        .expect("saldo verificado");
    assert_eq!(balance, 35);
}
