//! Razão contábil de moedas e cadeia criptográfica HMAC-SHA256 anti-adulteração.

use chrono::{DateTime, SecondsFormat, Utc};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::error::DomainError;
use crate::ids::EntityId;

type HmacSha256 = Hmac<Sha256>;

/// Teto diário padrão de moedas obtidas em fontes recorrentes para evitar inflação.
pub const DAILY_COIN_CAP: i64 = 150;

/// Hash inicial usado como `prev_hash` na primeira entrada da cadeia de um perfil.
pub const GENESIS_PREV_HASH: &str = "GENESIS";

/// Categorias de lançamento aceitas no razão append-only da carteira.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LedgerEntryType {
    /// Check-in diário de constância.
    DailyCheckin,
    /// Bloco de foco Pomodoro medido por relógio monotônico.
    PomodoroFocus,
    /// Meta diária de rotina (revisões, leitura ou hábitos).
    RoutineGoal,
    /// Bônus único de conquista desbloqueada (isento do teto diário).
    AchievementReward,
    /// Compra de item cosmético na loja (valor negativo).
    ShopPurchase,
    /// Concessão explícita via modo Admin/E2E (isenta do teto diário).
    AdminGrant,
    /// Compra de proteção de sequência (*Streak Freeze*, valor negativo).
    StreakFreezePurchase,
}

const ENTRY_TYPE_MAP: &[(LedgerEntryType, &str)] = &[
    (LedgerEntryType::DailyCheckin, "daily_checkin"),
    (LedgerEntryType::PomodoroFocus, "pomodoro_focus"),
    (LedgerEntryType::RoutineGoal, "routine_goal"),
    (LedgerEntryType::AchievementReward, "achievement_reward"),
    (LedgerEntryType::ShopPurchase, "shop_purchase"),
    (LedgerEntryType::AdminGrant, "admin_grant"),
    (
        LedgerEntryType::StreakFreezePurchase,
        "streak_freeze_purchase",
    ),
];

impl LedgerEntryType {
    /// Representação textual estável usada na persistência `SQLite` e assinatura HMAC.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        ENTRY_TYPE_MAP
            .iter()
            .find(|(kind, _)| *kind == self)
            .map_or("unknown", |(_, name)| *name)
    }

    /// Converte texto persistido no banco para [`LedgerEntryType`].
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let normalized = raw.trim();
        ENTRY_TYPE_MAP
            .iter()
            .find(|(_, name)| *name == normalized)
            .map(|(kind, _)| *kind)
            .ok_or_else(|| DomainError::InvalidLedgerEntryType(normalized.to_string()))
    }

    /// Indica se o lançamento sofre limitação pelo teto diário anti-inflação.
    #[must_use]
    pub const fn counts_toward_daily_cap(self) -> bool {
        matches!(
            self,
            Self::DailyCheckin | Self::PomodoroFocus | Self::RoutineGoal
        )
    }

    /// Indica se o lançamento representa gasto de saldo (exige valor negativo).
    #[must_use]
    pub const fn is_debit(self) -> bool {
        matches!(self, Self::ShopPurchase | Self::StreakFreezePurchase)
    }
}

/// Dados de entrada para registrar um novo lançamento na carteira.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletEntryDraft {
    /// Identificador único do lançamento (`UUIDv7`).
    pub id: EntityId,
    /// Categoria do lançamento.
    pub entry_type: LedgerEntryType,
    /// Variação de moedas (positiva para crédito, negativa para débito).
    pub amount: i64,
    /// Referência contextual (ex.: ID da conquista, sessão ou item da loja).
    pub reference_id: String,
    /// Chave de idempotência para evitar créditos duplicados.
    pub idempotency_key: String,
    /// Instante UTC do lançamento.
    pub created_at: DateTime<Utc>,
}

/// Entrada assinada e encadeada no razão `wallet_ledger`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedWalletEntry {
    /// Identificador único do lançamento.
    pub id: EntityId,
    /// Posição sequencial monotônica iniciada em `1`.
    pub seq: i64,
    /// Assinatura hexadecimal da entrada anterior (ou [`GENESIS_PREV_HASH`]).
    pub prev_hash: String,
    /// Categoria do lançamento.
    pub entry_type: LedgerEntryType,
    /// Variação de moedas aplicada.
    pub amount: i64,
    /// Referência contextual associada.
    pub reference_id: String,
    /// Chave única de idempotência.
    pub idempotency_key: String,
    /// Instante UTC do lançamento.
    pub created_at: DateTime<Utc>,
    /// Assinatura HMAC-SHA256 em hexadecimal minúsculo.
    pub hmac_sig: String,
}

/// Motor puro para derivação de chave, assinatura encadeada e verificação da carteira.
pub struct WalletChain;

impl WalletChain {
    /// Deriva a chave HMAC de 256 bits vinculada ao segredo da aplicação e ao perfil local.
    pub fn derive_profile_key(
        app_secret: &[u8],
        profile_id: &EntityId,
    ) -> Result<[u8; 32], DomainError> {
        let mut mac = HmacSha256::new_from_slice(app_secret)
            .map_err(|err| DomainError::CryptoFailure(err.to_string()))?;
        mac.update(b"estudy-wallet-v1:");
        mac.update(profile_id.to_string().as_bytes());
        let bytes = mac.finalize().into_bytes();
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    }

    /// Aplica o teto diário anti-inflação apenas sobre fontes recorrentes.
    #[must_use]
    pub fn apply_daily_cap(
        already_earned_today: i64,
        requested_amount: i64,
        entry_type: LedgerEntryType,
    ) -> i64 {
        if requested_amount <= 0 {
            return 0;
        }
        if !entry_type.counts_toward_daily_cap() {
            return requested_amount;
        }
        let remaining = (DAILY_COIN_CAP - already_earned_today.max(0)).max(0);
        requested_amount.min(remaining)
    }

    /// Valida regras de negócio e gera o próximo lançamento assinado da cadeia.
    pub fn sign_next_entry(
        key: &[u8; 32],
        previous: Option<&SignedWalletEntry>,
        current_balance: i64,
        draft: WalletEntryDraft,
    ) -> Result<SignedWalletEntry, DomainError> {
        validate_draft_amount(draft.entry_type, draft.amount, current_balance)?;

        let (seq, prev_hash) = match previous {
            Some(prev) => (prev.seq + 1, prev.hmac_sig.clone()),
            None => (1, GENESIS_PREV_HASH.to_string()),
        };

        let hmac_sig = compute_entry_hmac(
            key,
            seq,
            &prev_hash,
            draft.entry_type,
            draft.amount,
            &draft.reference_id,
            draft.created_at,
        )?;

        Ok(SignedWalletEntry {
            id: draft.id,
            seq,
            prev_hash,
            entry_type: draft.entry_type,
            amount: draft.amount,
            reference_id: draft.reference_id,
            idempotency_key: draft.idempotency_key,
            created_at: draft.created_at,
            hmac_sig,
        })
    }

    /// Percorre todas as entradas em ordem verificando sequência, encadeamento e assinatura HMAC.
    ///
    /// Retorna o saldo total consolidado quando toda a cadeia é autêntica.
    pub fn verify_chain(key: &[u8; 32], entries: &[SignedWalletEntry]) -> Result<i64, DomainError> {
        let mut expected_prev_hash = GENESIS_PREV_HASH.to_string();
        let mut balance = 0_i64;

        for (expected_seq, entry) in (1_i64..).zip(entries.iter()) {
            verify_single_entry(key, entry, expected_seq, &expected_prev_hash)?;
            balance = balance.checked_add(entry.amount).ok_or_else(|| {
                DomainError::WalletChainTampered {
                    seq: entry.seq,
                    reason: "estouro numérico ao somar saldo".to_string(),
                }
            })?;
            if balance < 0 {
                return Err(DomainError::WalletChainTampered {
                    seq: entry.seq,
                    reason: "saldo negativo detectado no histórico".to_string(),
                });
            }

            expected_prev_hash.clone_from(&entry.hmac_sig);
        }

        Ok(balance)
    }
}

fn validate_draft_amount(
    entry_type: LedgerEntryType,
    amount: i64,
    current_balance: i64,
) -> Result<(), DomainError> {
    if amount == 0 {
        return Err(DomainError::InvalidWalletAmount {
            entry_type: entry_type.as_str(),
            amount,
        });
    }

    if entry_type.is_debit() {
        if amount > 0 {
            return Err(DomainError::InvalidWalletAmount {
                entry_type: entry_type.as_str(),
                amount,
            });
        }
        let required = amount.saturating_abs();
        if current_balance < required {
            return Err(DomainError::InsufficientBalance {
                available: current_balance,
                required,
            });
        }
        return Ok(());
    }

    if amount < 0 {
        return Err(DomainError::InvalidWalletAmount {
            entry_type: entry_type.as_str(),
            amount,
        });
    }

    Ok(())
}

fn verify_single_entry(
    key: &[u8; 32],
    entry: &SignedWalletEntry,
    expected_seq: i64,
    expected_prev_hash: &str,
) -> Result<(), DomainError> {
    if entry.seq != expected_seq {
        return Err(DomainError::WalletChainTampered {
            seq: entry.seq,
            reason: format!(
                "sequência esperada {expected_seq}, encontrada {}",
                entry.seq
            ),
        });
    }
    if entry.prev_hash != expected_prev_hash {
        return Err(DomainError::WalletChainTampered {
            seq: entry.seq,
            reason: "prev_hash divergente da assinatura anterior".to_string(),
        });
    }

    let provided_sig =
        hex::decode(&entry.hmac_sig).map_err(|_| DomainError::WalletChainTampered {
            seq: entry.seq,
            reason: "assinatura HMAC em formato hexadecimal inválido".to_string(),
        })?;

    let mac = build_entry_mac(
        key,
        entry.seq,
        &entry.prev_hash,
        entry.entry_type,
        entry.amount,
        &entry.reference_id,
        entry.created_at,
    )?;

    mac.verify_slice(&provided_sig)
        .map_err(|_| DomainError::WalletChainTampered {
            seq: entry.seq,
            reason: "assinatura HMAC-SHA256 inválida".to_string(),
        })
}

fn compute_entry_hmac(
    key: &[u8; 32],
    seq: i64,
    prev_hash: &str,
    entry_type: LedgerEntryType,
    amount: i64,
    reference_id: &str,
    created_at: DateTime<Utc>,
) -> Result<String, DomainError> {
    let mac = build_entry_mac(
        key,
        seq,
        prev_hash,
        entry_type,
        amount,
        reference_id,
        created_at,
    )?;
    Ok(hex::encode(mac.finalize().into_bytes()))
}

fn build_entry_mac(
    key: &[u8; 32],
    seq: i64,
    prev_hash: &str,
    entry_type: LedgerEntryType,
    amount: i64,
    reference_id: &str,
    created_at: DateTime<Utc>,
) -> Result<HmacSha256, DomainError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|err| DomainError::CryptoFailure(err.to_string()))?;

    mac.update(&seq.to_be_bytes());
    update_framed(&mut mac, prev_hash.as_bytes());
    update_framed(&mut mac, entry_type.as_str().as_bytes());
    mac.update(&amount.to_be_bytes());
    update_framed(&mut mac, reference_id.as_bytes());
    let timestamp = created_at.to_rfc3339_opts(SecondsFormat::Micros, true);
    update_framed(&mut mac, timestamp.as_bytes());

    Ok(mac)
}

fn update_framed(mac: &mut HmacSha256, bytes: &[u8]) {
    let len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    mac.update(&len.to_be_bytes());
    mac.update(bytes);
}
