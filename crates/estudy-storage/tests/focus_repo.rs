//! Testes de integração `SQLite` para `SqliteFocusRepository`.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use chrono::{NaiveDate, TimeZone, Utc};
use estudy_domain::{
    DomainEvent, EntityId, EventBusPort, FocusModality, FocusSessionDraft, InMemoryEventBus,
};
use estudy_storage::{
    SqliteFocusRepository, SqliteSearchRepository, SqliteWalletRepository, apply_migrations,
    open_in_memory,
};

#[test]
fn should_record_focus_session_quick_note_and_hmac_wallet_credit_in_single_transaction() {
    let mut conn = open_in_memory().expect("abrir sqlite em memória");
    let boot = Utc.with_ymd_and_hms(2026, 11, 23, 8, 0, 0).unwrap();
    apply_migrations(&mut conn, None, boot).expect("aplicar migrações v1");

    let profile_id = EntityId::new_v7();
    conn.execute(
        "INSERT INTO local_profiles (id, display_name, created_at, updated_at)
         VALUES (?1, 'Arthur', '2026-11-23T10:00:00Z', '2026-11-23T10:00:00Z')",
        [profile_id.to_string()],
    )
    .expect("inserir perfil local");

    let secret = b"estudy-test-secret";
    let bus = InMemoryEventBus::new();
    let start = Utc.with_ymd_and_hms(2026, 11, 23, 14, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 11, 23, 14, 25, 0).unwrap();

    let draft = FocusSessionDraft {
        id: EntityId::new_v7(),
        modality: FocusModality::Focus,
        duration_minutes: 25,
        questions_total: 10,
        questions_right: 9,
        note_capture: Some("Resumo de árvores B+ gravado após Pomodoro".to_string()),
        task_id: None,
        started_at: start,
        ended_at: end,
    };

    let outcome = SqliteFocusRepository::record_session(
        &mut conn,
        secret,
        &profile_id,
        draft.clone(),
        Some(&bus as &dyn EventBusPort),
    )
    .expect("registrar sessão de foco");

    assert_eq!(outcome.awarded_coins, 5);
    assert!(outcome.note_id.is_some());

    let balance = SqliteWalletRepository::verify_wallet_balance(&conn, secret, &profile_id)
        .expect("verificar cadeia HMAC da carteira");
    assert_eq!(balance, 5);

    let today_minutes = SqliteFocusRepository::today_focus_minutes(
        &conn,
        NaiveDate::from_ymd_opt(2026, 11, 23).unwrap(),
    )
    .expect("consultar minutos focados hoje");
    assert_eq!(today_minutes, 25);

    let recent = SqliteFocusRepository::list_recent(&conn, 10).expect("listar sessões recentes");
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].duration_minutes, 25);

    let fts_hits = SqliteSearchRepository::search_all(&conn, "B+", 10)
        .expect("buscar nota capturada via FTS5");
    assert!(!fts_hits.is_empty());

    let events = bus.snapshot();
    assert!(matches!(
        events.as_slice(),
        [
            DomainEvent::WalletEntryRecorded { delta: 5, .. },
            DomainEvent::FocusSessionLogged {
                duration_minutes: 25,
                awarded_coins: 5,
                ..
            }
        ]
    ));

    // Reenvio idempotente com mesmo ID não duplica moedas nem minutos.
    let repeated =
        SqliteFocusRepository::record_session(&mut conn, secret, &profile_id, draft, None)
            .expect("idempotência da sessão");
    assert_eq!(repeated.awarded_coins, 5);
    let total_after = SqliteFocusRepository::today_focus_minutes(
        &conn,
        NaiveDate::from_ymd_opt(2026, 11, 23).unwrap(),
    )
    .unwrap();
    assert_eq!(total_after, 25);
}

#[test]
fn should_skip_wallet_credit_when_session_is_under_15_minutes() {
    let mut conn = open_in_memory().expect("abrir sqlite em memória");
    let boot = Utc.with_ymd_and_hms(2026, 11, 23, 8, 0, 0).unwrap();
    apply_migrations(&mut conn, None, boot).expect("aplicar migrações v1");

    let profile_id = EntityId::new_v7();
    conn.execute(
        "INSERT INTO local_profiles (id, display_name, created_at, updated_at)
         VALUES (?1, 'Arthur', '2026-11-23T10:00:00Z', '2026-11-23T10:00:00Z')",
        [profile_id.to_string()],
    )
    .expect("inserir perfil local");

    let secret = b"estudy-test-secret";
    let start = Utc.with_ymd_and_hms(2026, 11, 23, 15, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2026, 11, 23, 15, 10, 0).unwrap();

    let draft = FocusSessionDraft {
        id: EntityId::new_v7(),
        modality: FocusModality::Reading,
        duration_minutes: 10,
        questions_total: 0,
        questions_right: 0,
        note_capture: None,
        task_id: None,
        started_at: start,
        ended_at: end,
    };

    let outcome =
        SqliteFocusRepository::record_session(&mut conn, secret, &profile_id, draft, None)
            .expect("registrar bloco curto");
    assert_eq!(outcome.awarded_coins, 0);
    assert!(outcome.note_id.is_none());

    let balance =
        SqliteWalletRepository::verify_wallet_balance(&conn, secret, &profile_id).unwrap();
    assert_eq!(balance, 0);
}
