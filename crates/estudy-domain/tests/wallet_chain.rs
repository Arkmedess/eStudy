//! Testes de comportamento da cadeia HMAC e regras econômicas da carteira.

#![allow(clippy::expect_used)]

use chrono::{DateTime, Utc};
use estudy_domain::{
    DAILY_COIN_CAP, DomainError, EntityId, GENESIS_PREV_HASH, LedgerEntryType, WalletChain,
    WalletEntryDraft,
};

fn sample_time(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(1_760_000_000 + secs, 0).unwrap_or(DateTime::UNIX_EPOCH)
}

#[test]
fn should_sign_and_verify_chained_wallet_entries_and_compute_balance() {
    let profile_id = EntityId::new_v7();
    let key = WalletChain::derive_profile_key(b"estudy-local-app-secret", &profile_id)
        .expect("derivação de chave deve funcionar");

    let first = WalletChain::sign_next_entry(
        &key,
        None,
        0,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::DailyCheckin,
            amount: 30,
            reference_id: "checkin:2026-11-22".to_string(),
            idempotency_key: "checkin-2026-11-22".to_string(),
            created_at: sample_time(10),
        },
    )
    .expect("primeiro lançamento válido deve ser assinado");

    assert_eq!(first.seq, 1);
    assert_eq!(first.prev_hash, GENESIS_PREV_HASH);
    assert!(!first.hmac_sig.is_empty());

    let second = WalletChain::sign_next_entry(
        &key,
        Some(&first),
        30,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::PomodoroFocus,
            amount: 50,
            reference_id: "pomo:01".to_string(),
            idempotency_key: "pomo-01".to_string(),
            created_at: sample_time(20),
        },
    )
    .expect("segundo lançamento deve encadear com o primeiro");

    assert_eq!(second.seq, 2);
    assert_eq!(second.prev_hash, first.hmac_sig);

    let third = WalletChain::sign_next_entry(
        &key,
        Some(&second),
        80,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::ShopPurchase,
            amount: -45,
            reference_id: "pet:coelho".to_string(),
            idempotency_key: "buy-pet-coelho".to_string(),
            created_at: sample_time(30),
        },
    )
    .expect("compra com saldo suficiente deve ser assinada");

    let verified_balance =
        WalletChain::verify_chain(&key, &[first, second, third]).expect("cadeia íntegra");
    assert_eq!(verified_balance, 35);
}

#[test]
fn should_detect_tampered_amount_or_broken_sequence() {
    let profile_id = EntityId::new_v7();
    let key = WalletChain::derive_profile_key(b"estudy-local-app-secret", &profile_id)
        .expect("derivação deve funcionar");

    let first = WalletChain::sign_next_entry(
        &key,
        None,
        0,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::DailyCheckin,
            amount: 20,
            reference_id: "checkin:1".to_string(),
            idempotency_key: "k1".to_string(),
            created_at: sample_time(1),
        },
    )
    .expect("lançamento inicial válido");

    let mut tampered = first.clone();
    tampered.amount = 99_999;

    let err =
        WalletChain::verify_chain(&key, &[tampered]).expect_err("adulteração de valor deve falhar");
    assert!(matches!(
        err,
        DomainError::WalletChainTampered { seq: 1, .. }
    ));

    let mut wrong_seq = first;
    wrong_seq.seq = 2;
    let seq_err =
        WalletChain::verify_chain(&key, &[wrong_seq]).expect_err("lacuna na sequência deve falhar");
    assert!(matches!(
        seq_err,
        DomainError::WalletChainTampered { seq: 2, .. }
    ));
}

#[test]
fn should_reject_insufficient_balance_and_invalid_signs() {
    let profile_id = EntityId::new_v7();
    let key = WalletChain::derive_profile_key(b"secret", &profile_id).expect("chave válida");

    let err = WalletChain::sign_next_entry(
        &key,
        None,
        10,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::ShopPurchase,
            amount: -25,
            reference_id: "item:1".to_string(),
            idempotency_key: "buy-1".to_string(),
            created_at: sample_time(5),
        },
    )
    .expect_err("compra acima do saldo deve falhar");
    assert_eq!(
        err,
        DomainError::InsufficientBalance {
            available: 10,
            required: 25,
        }
    );

    let bad_credit = WalletChain::sign_next_entry(
        &key,
        None,
        100,
        WalletEntryDraft {
            id: EntityId::new_v7(),
            entry_type: LedgerEntryType::PomodoroFocus,
            amount: -10,
            reference_id: "pomo".to_string(),
            idempotency_key: "p1".to_string(),
            created_at: sample_time(6),
        },
    )
    .expect_err("crédito negativo deve falhar");
    assert!(matches!(
        bad_credit,
        DomainError::InvalidWalletAmount { .. }
    ));
}

#[test]
fn should_enforce_daily_cap_only_for_recurring_faucets() {
    assert_eq!(
        WalletChain::apply_daily_cap(130, 40, LedgerEntryType::PomodoroFocus),
        DAILY_COIN_CAP - 130
    );
    assert_eq!(
        WalletChain::apply_daily_cap(DAILY_COIN_CAP, 30, LedgerEntryType::DailyCheckin),
        0
    );
    assert_eq!(
        WalletChain::apply_daily_cap(DAILY_COIN_CAP, 100, LedgerEntryType::AchievementReward),
        100
    );
    assert_eq!(
        WalletChain::apply_daily_cap(DAILY_COIN_CAP, 500, LedgerEntryType::AdminGrant),
        500
    );
}
