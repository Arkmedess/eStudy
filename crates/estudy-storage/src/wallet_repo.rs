//! Repositório transacional do razão `wallet_ledger` com verificação HMAC-SHA256.

use chrono::{DateTime, SecondsFormat, Utc};
use estudy_domain::{
    DomainError, DomainEvent, EntityId, EventBusPort, LedgerEntryType, SignedWalletEntry,
    WalletChain, WalletEntryDraft,
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::error::StorageError;

/// Persiste e audita lançamentos da carteira no `SQLite` garantindo integridade criptográfica.
pub struct SqliteWalletRepository;

impl SqliteWalletRepository {
    /// Registra um novo lançamento na carteira dentro de uma transação única (idempotente).
    pub fn record_entry(
        conn: &mut Connection,
        app_secret: &[u8],
        profile_id: &EntityId,
        draft: WalletEntryDraft,
        event_bus: Option<&dyn EventBusPort>,
    ) -> Result<SignedWalletEntry, StorageError> {
        let tx = conn.transaction()?;
        let (signed, inserted) = Self::record_entry_in_tx(&tx, app_secret, profile_id, draft)?;
        tx.commit()?;

        if inserted && let Some(bus) = event_bus {
            bus.publish(DomainEvent::WalletEntryRecorded {
                profile_id: profile_id.clone(),
                seq: signed.seq,
                delta: signed.amount,
            });
        }

        Ok(signed)
    }

    pub(crate) fn record_entry_in_tx(
        tx: &Transaction<'_>,
        app_secret: &[u8],
        profile_id: &EntityId,
        draft: WalletEntryDraft,
    ) -> Result<(SignedWalletEntry, bool), StorageError> {
        let key = WalletChain::derive_profile_key(app_secret, profile_id)?;
        if let Some(existing) = find_by_idempotency_key(tx, &draft.idempotency_key)? {
            return Ok((existing, false));
        }

        let entries = load_entries_for_profile(tx, profile_id)?;
        let current_balance = WalletChain::verify_chain(&key, &entries)?;
        let signed = WalletChain::sign_next_entry(&key, entries.last(), current_balance, draft)?;

        let created_at = signed
            .created_at
            .to_rfc3339_opts(SecondsFormat::Micros, true);
        tx.execute(
            "INSERT INTO wallet_ledger (id, profile_id, seq, prev_hash, entry_type, amount, reference_id, idempotency_key, hmac_sig, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                signed.id.to_string(),
                profile_id.to_string(),
                signed.seq,
                signed.prev_hash,
                signed.entry_type.as_str(),
                signed.amount,
                signed.reference_id,
                signed.idempotency_key,
                signed.hmac_sig,
                created_at,
            ],
        )?;
        Ok((signed, true))
    }

    /// Carrega todas as entradas do perfil em ordem e verifica a cadeia HMAC-SHA256.
    pub fn verify_wallet_balance(
        conn: &Connection,
        app_secret: &[u8],
        profile_id: &EntityId,
    ) -> Result<i64, StorageError> {
        let key = WalletChain::derive_profile_key(app_secret, profile_id)?;
        let entries = load_entries_for_profile(conn, profile_id)?;
        let balance = WalletChain::verify_chain(&key, &entries)?;
        Ok(balance)
    }
}

fn find_by_idempotency_key(
    tx: &Transaction<'_>,
    idempotency_key: &str,
) -> Result<Option<SignedWalletEntry>, StorageError> {
    let row = tx
        .query_row(
            "SELECT id, seq, prev_hash, entry_type, amount, reference_id, idempotency_key, created_at, hmac_sig
             FROM wallet_ledger
             WHERE idempotency_key = ?1",
            [idempotency_key],
            map_row,
        )
        .optional()?;

    row.transpose()
}

fn load_entries_for_profile(
    conn: &Connection,
    profile_id: &EntityId,
) -> Result<Vec<SignedWalletEntry>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, seq, prev_hash, entry_type, amount, reference_id, idempotency_key, created_at, hmac_sig
         FROM wallet_ledger
         WHERE profile_id = ?1
         ORDER BY seq ASC",
    )?;

    let rows = stmt.query_map([profile_id.to_string()], map_row)?;
    let mut entries = Vec::new();
    for item in rows {
        entries.push(item??);
    }
    Ok(entries)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Result<SignedWalletEntry, StorageError>> {
    let raw_id: String = row.get(0)?;
    let seq: i64 = row.get(1)?;
    let prev_hash: String = row.get(2)?;
    let raw_type: String = row.get(3)?;
    let amount: i64 = row.get(4)?;
    let reference_id: String = row.get(5)?;
    let idempotency_key: String = row.get(6)?;
    let raw_created_at: String = row.get(7)?;
    let hmac_sig: String = row.get(8)?;

    Ok(parse_signed_entry(
        &raw_id,
        seq,
        prev_hash,
        &raw_type,
        amount,
        reference_id,
        idempotency_key,
        &raw_created_at,
        hmac_sig,
    ))
}

#[allow(clippy::too_many_arguments)]
fn parse_signed_entry(
    raw_id: &str,
    seq: i64,
    prev_hash: String,
    raw_type: &str,
    amount: i64,
    reference_id: String,
    idempotency_key: String,
    raw_created_at: &str,
    hmac_sig: String,
) -> Result<SignedWalletEntry, StorageError> {
    let id = EntityId::parse(raw_id)?;
    let entry_type = LedgerEntryType::parse(raw_type)?;
    let created_at = DateTime::parse_from_rfc3339(raw_created_at)
        .map_err(|err| {
            StorageError::Domain(DomainError::WalletChainTampered {
                seq,
                reason: format!("timestamp inválido ({err})"),
            })
        })?
        .with_timezone(&Utc);

    Ok(SignedWalletEntry {
        id,
        seq,
        prev_hash,
        entry_type,
        amount,
        reference_id,
        idempotency_key,
        created_at,
        hmac_sig,
    })
}
