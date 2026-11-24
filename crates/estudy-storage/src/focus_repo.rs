//! Repositório transacional para sessões de foco (`study_sessions`), captura rápida e moedas.

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use estudy_domain::{
    DAILY_COIN_CAP, DomainEvent, EntityId, EventBusPort, FocusModality, FocusSessionDraft,
    LedgerEntryType, WalletEntryDraft,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::error::StorageError;
use crate::wallet_repo::SqliteWalletRepository;

/// Resumo persistido de uma sessão de foco concluída.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredFocusSession {
    /// Identificador único da sessão.
    pub id: EntityId,
    /// Modalidade da sessão (`focus`, `theory`, `questions`, `review`, `reading`).
    pub modality: FocusModality,
    /// Duração efetivamente focada em minutos.
    pub duration_minutes: u32,
    /// Total de questões respondidas.
    pub questions_total: u32,
    /// Total de acertos.
    pub questions_right: u32,
    /// Data calendário (`YYYY-MM-DD`).
    pub session_date: NaiveDate,
    /// Horário de término em UTC.
    pub ended_at: DateTime<Utc>,
}

/// Resultado da gravação atômica de uma sessão de foco concluída.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedFocusOutcome {
    /// Identificador da sessão gravada.
    pub session_id: EntityId,
    /// Identificador da nota criada caso o usuário tenha capturado um resumo rápido.
    pub note_id: Option<EntityId>,
    /// Moedas creditadas após verificação de duração mínima (`15 min`) e teto diário.
    pub awarded_coins: i64,
}

/// Persistência de sessões Pomodoro/Foco em transação única `SQLite`.
pub struct SqliteFocusRepository;

impl SqliteFocusRepository {
    /// Grava sessão, nota rápida (`FTS5`) e crédito HMAC na carteira em uma única transação.
    pub fn record_session(
        conn: &mut Connection,
        app_secret: &[u8],
        profile_id: &EntityId,
        draft: FocusSessionDraft,
        event_bus: Option<&dyn EventBusPort>,
    ) -> Result<RecordedFocusOutcome, StorageError> {
        let validated = draft.validate()?;
        let session = validated.draft;
        let session_id = session.id.clone();
        let session_id_str = session_id.to_string();
        let session_date_str = session.ended_at.date_naive().format("%Y-%m-%d").to_string();
        let started_str = session
            .started_at
            .to_rfc3339_opts(SecondsFormat::Micros, true);
        let ended_str = session
            .ended_at
            .to_rfc3339_opts(SecondsFormat::Micros, true);

        let tx = conn.transaction()?;

        if let Some(existing) = find_existing_outcome(&tx, &session_id, &session_id_str)? {
            tx.commit()?;
            return Ok(existing);
        }

        tx.execute(
            "INSERT INTO study_sessions (
                id, task_id, modality, duration_minutes, questions_total, questions_right,
                session_date, started_at, ended_at, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                session_id_str,
                session.task_id.as_ref().map(ToString::to_string),
                session.modality.as_str(),
                i64::from(session.duration_minutes),
                i64::from(session.questions_total),
                i64::from(session.questions_right),
                session_date_str,
                started_str,
                ended_str,
                ended_str,
            ],
        )?;

        let note_id = insert_quick_capture_note(
            &tx,
            session.note_capture.as_deref(),
            &session_id_str,
            &session_date_str,
            &ended_str,
        )?;

        let (awarded_coins, wallet_seq) = credit_focus_coins_in_tx(
            &tx,
            app_secret,
            profile_id,
            &session_id_str,
            &session_date_str,
            validated.awarded_coins,
            session.ended_at,
        )?;

        tx.commit()?;

        tracing::info!(
            target: "estudy.storage",
            duration_minutes = session.duration_minutes,
            awarded_coins,
            has_note = note_id.is_some(),
            "sessão de foco registrada no SQLite"
        );

        publish_focus_events(
            event_bus,
            profile_id,
            &session_id,
            session.duration_minutes,
            awarded_coins,
            wallet_seq,
        );

        Ok(RecordedFocusOutcome {
            session_id,
            note_id,
            awarded_coins,
        })
    }

    /// Soma os minutos focados registrados na data informada.
    pub fn today_focus_minutes(conn: &Connection, date: NaiveDate) -> Result<u32, StorageError> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let sum: i64 = conn.query_row(
            "SELECT COALESCE(SUM(duration_minutes), 0)
             FROM study_sessions
             WHERE session_date = ?1 AND deleted_at IS NULL",
            [date_str],
            |row| row.get(0),
        )?;
        Ok(u32::try_from(sum.max(0)).unwrap_or(u32::MAX))
    }

    /// Lista as sessões de foco mais recentes ordenadas por término decrescente.
    pub fn list_recent(
        conn: &Connection,
        limit: usize,
    ) -> Result<Vec<StoredFocusSession>, StorageError> {
        let clamped_limit = i64::try_from(limit.clamp(1, 100)).unwrap_or(20);
        let mut stmt = conn.prepare(
            "SELECT id, modality, duration_minutes, questions_total, questions_right, session_date, ended_at
             FROM study_sessions
             WHERE deleted_at IS NULL
             ORDER BY ended_at DESC, id DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map([clamped_limit], |row| {
            let raw_id: String = row.get(0)?;
            let raw_mod: String = row.get(1)?;
            let duration: i64 = row.get(2)?;
            let q_total: i64 = row.get(3)?;
            let q_right: i64 = row.get(4)?;
            let raw_date: String = row.get(5)?;
            let raw_ended: String = row.get(6)?;
            Ok((
                raw_id, raw_mod, duration, q_total, q_right, raw_date, raw_ended,
            ))
        })?;

        let mut sessions = Vec::new();
        for item in rows {
            let (raw_id, raw_mod, duration, q_total, q_right, raw_date, raw_ended) = item?;
            let id = EntityId::parse(&raw_id)?;
            let modality = parse_modality(&raw_mod);
            let session_date = NaiveDate::parse_from_str(&raw_date, "%Y-%m-%d").map_err(|err| {
                StorageError::IntegrityCheckFailed(format!(
                    "data de sessão inválida '{raw_date}': {err}"
                ))
            })?;
            let ended_at = DateTime::parse_from_rfc3339(&raw_ended)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|err| {
                    StorageError::IntegrityCheckFailed(format!(
                        "timestamp de sessão inválido: {err}"
                    ))
                })?;

            sessions.push(StoredFocusSession {
                id,
                modality,
                duration_minutes: u32::try_from(duration.max(0)).unwrap_or(0),
                questions_total: u32::try_from(q_total.max(0)).unwrap_or(0),
                questions_right: u32::try_from(q_right.max(0)).unwrap_or(0),
                session_date,
                ended_at,
            });
        }
        Ok(sessions)
    }
}

fn insert_quick_capture_note(
    tx: &Transaction<'_>,
    note_capture: Option<&str>,
    session_id: &str,
    session_date: &str,
    now_iso: &str,
) -> Result<Option<EntityId>, StorageError> {
    let Some(content) = note_capture.map(str::trim).filter(|text| !text.is_empty()) else {
        return Ok(None);
    };

    let note_id = EntityId::new_v7();
    let slug = format!("focus-{session_id}");
    let title: String = content
        .lines()
        .next()
        .unwrap_or("Captura de Foco")
        .chars()
        .take(60)
        .collect();

    tx.execute(
        "INSERT INTO notes (id, slug, title, kind, summary, body_md, note_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'free', ?4, ?5, ?6, ?7, ?8)",
        params![
            note_id.to_string(),
            slug,
            title,
            content.chars().take(120).collect::<String>(),
            content,
            session_date,
            now_iso,
            now_iso,
        ],
    )?;

    Ok(Some(note_id))
}

fn remaining_daily_coin_cap(
    tx: &Transaction<'_>,
    profile_id: &EntityId,
    session_date: &str,
) -> Result<i64, StorageError> {
    let earned_today: i64 = tx.query_row(
        "SELECT COALESCE(SUM(amount), 0)
         FROM wallet_ledger
         WHERE profile_id = ?1
           AND entry_type IN ('daily_checkin', 'pomodoro_focus', 'routine_goal')
           AND amount > 0
           AND substr(created_at, 1, 10) = ?2",
        params![profile_id.to_string(), session_date],
        |row| row.get(0),
    )?;
    Ok((DAILY_COIN_CAP - earned_today).max(0))
}

fn find_existing_outcome(
    tx: &Transaction<'_>,
    session_id: &EntityId,
    session_id_str: &str,
) -> Result<Option<RecordedFocusOutcome>, StorageError> {
    let already_exists: Option<String> = tx
        .query_row(
            "SELECT id FROM study_sessions WHERE id = ?1",
            [session_id_str],
            |row| row.get(0),
        )
        .optional()?;

    if already_exists.is_none() {
        return Ok(None);
    }

    let existing_coins: Option<i64> = tx
        .query_row(
            "SELECT amount FROM wallet_ledger WHERE idempotency_key = ?1",
            [format!("focus_session:{session_id_str}")],
            |row| row.get(0),
        )
        .optional()?;

    Ok(Some(RecordedFocusOutcome {
        session_id: session_id.clone(),
        note_id: None,
        awarded_coins: existing_coins.unwrap_or(0),
    }))
}

fn credit_focus_coins_in_tx(
    tx: &Transaction<'_>,
    app_secret: &[u8],
    profile_id: &EntityId,
    session_id_str: &str,
    session_date_str: &str,
    base_award: i64,
    ended_at: DateTime<Utc>,
) -> Result<(i64, Option<(i64, i64)>), StorageError> {
    let remaining_cap = remaining_daily_coin_cap(tx, profile_id, session_date_str)?;
    let awarded_coins = base_award.min(remaining_cap).max(0);

    if awarded_coins <= 0 {
        return Ok((0, None));
    }

    let entry_draft = WalletEntryDraft {
        id: EntityId::new_v7(),
        entry_type: LedgerEntryType::PomodoroFocus,
        amount: awarded_coins,
        reference_id: session_id_str.to_string(),
        idempotency_key: format!("focus_session:{session_id_str}"),
        created_at: ended_at,
    };
    let (signed, inserted) =
        SqliteWalletRepository::record_entry_in_tx(tx, app_secret, profile_id, entry_draft)?;
    let wallet_seq = inserted.then_some((signed.seq, signed.amount));
    Ok((awarded_coins, wallet_seq))
}

fn publish_focus_events(
    event_bus: Option<&dyn EventBusPort>,
    profile_id: &EntityId,
    session_id: &EntityId,
    duration_minutes: u32,
    awarded_coins: i64,
    wallet_seq: Option<(i64, i64)>,
) {
    let Some(bus) = event_bus else {
        return;
    };
    if let Some((seq, delta)) = wallet_seq {
        bus.publish(DomainEvent::WalletEntryRecorded {
            profile_id: profile_id.clone(),
            seq,
            delta,
        });
    }
    bus.publish(DomainEvent::FocusSessionLogged {
        session_id: session_id.clone(),
        duration_minutes,
        awarded_coins,
    });
}

fn parse_modality(raw: &str) -> FocusModality {
    match raw.trim() {
        "theory" => FocusModality::Theory,
        "questions" => FocusModality::Questions,
        "review" => FocusModality::Review,
        "reading" => FocusModality::Reading,
        _ => FocusModality::Focus,
    }
}
