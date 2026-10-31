//! Contract bindings for the four modules.
//!
//! * read helpers build on-chain calls through `chain::call` and decode the
//!   returned felt vector into the frontend types (`types.rs`).
//! * write helpers only *encode* raw calldata — submitting goes through the
//!   wallet bridge (`wallet::invoke`), which needs hex-felt JSON arrays.
//!
//! Field orders mirror the Cairo `#[derive(Serde)]` structs in
//! `contracts/src/types/` exactly.

use starknet_rust::core::types::Felt;
use starknet_rust::core::utils::get_selector_from_name;

use crate::chain::{felt_u128, parse_felt};
use crate::config::Network;
use crate::types::{
    Amount, Conviction, Dispute, Evaluation, PendingAdminChange, PendingChange, PendingUpgrade,
    Proposal, Stake,
};

// ─────────────────────────── module addressing ──────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Module {
    Token,
    Selection,
    Governance,
    AdminTimelock,
}

impl Module {
    pub fn label(self) -> &'static str {
        match self {
            Module::Token => "ConvictionToken",
            Module::Selection => "Selection",
            Module::Governance => "Governance",
            Module::AdminTimelock => "AdminTimelock",
        }
    }

    /// Name of the config constant holding this module's address,
    /// e.g. `SEPOLIA_SELECTION`.
    pub fn config_const(self, net: Network) -> String {
        format!(
            "{}{}",
            net.config_prefix(),
            match self {
                Module::Token => "CONVICTION_TOKEN",
                Module::Selection => "SELECTION",
                Module::Governance => "GOVERNANCE",
                Module::AdminTimelock => "ADMIN_TIMELOCK",
            }
        )
    }
}

fn raw_addr(net: Network, m: Module) -> &'static str {
    use crate::config as cfg;
    match (net, m) {
        (Network::Sepolia, Module::Token) => cfg::SEPOLIA_CONVICTION_TOKEN,
        (Network::Sepolia, Module::Selection) => cfg::SEPOLIA_SELECTION,
        (Network::Sepolia, Module::Governance) => cfg::SEPOLIA_GOVERNANCE,
        (Network::Sepolia, Module::AdminTimelock) => cfg::SEPOLIA_ADMIN_TIMELOCK,
        (Network::Mainnet, Module::Token) => cfg::MAINNET_CONVICTION_TOKEN,
        (Network::Mainnet, Module::Selection) => cfg::MAINNET_SELECTION,
        (Network::Mainnet, Module::Governance) => cfg::MAINNET_GOVERNANCE,
        (Network::Mainnet, Module::AdminTimelock) => cfg::MAINNET_ADMIN_TIMELOCK,
    }
}

/// The module address for a network, or an error explaining which constant
/// to fill in `src/config.rs`.
pub fn addr(net: Network, m: Module) -> Result<String, String> {
    let raw = raw_addr(net, m);
    if crate::chain::is_valid_address(raw) {
        Ok(crate::chain::normalized_hex(raw))
    } else {
        Err(format!(
            "{} is not configured for {} — paste the deployed address into {} in src/config.rs",
            m.label(),
            net.label(),
            m.config_const(net),
        ))
    }
}

/// The configured address for a module, or `"—"`.
pub fn address_display(net: Network, m: Module) -> String {
    addr(net, m).unwrap_or_else(|_| "—".into())
}

pub fn is_configured(net: Network, m: Module) -> bool {
    addr(net, m).is_ok()
}

// ─────────────────────────── calldata encoding ──────────────────────────────

/// Calldata as a JSON array of `0x…` strings (the wallet-shim contract).
pub fn calldata_json(parts: &[Felt]) -> String {
    let joined: Vec<String> = parts
        .iter()
        .map(|f| format!("\"{}\"", f.to_hex_string()))
        .collect();
    format!("[{}]", joined.join(","))
}

fn scalar(v: u128) -> Felt {
    Felt::from(v)
}

/// Any small scalar (u8/u16/u32/u64): one felt.
pub fn enc_scalar(v: u128) -> Vec<Felt> {
    vec![scalar(v)]
}

/// Cairo u256: (low, high) felt pair.
pub fn enc_u256(a: Amount) -> Vec<Felt> {
    vec![Felt::from(a.low), Felt::from(a.high)]
}

/// ContractAddress felt.
pub fn enc_felt(value: &str) -> Result<Vec<Felt>, String> {
    Ok(vec![parse_felt(value)?])
}

/// Cairo ByteArray serde: `[n_words, w0..wn, pending_len, pending_word]`.
pub fn enc_byte_array(text: &str) -> Result<Vec<Felt>, String> {
    let bytes = text.as_bytes();
    let n_words = bytes.len() / 31;
    let pending_len = bytes.len() % 31;
    let mut out = Vec::with_capacity(n_words + 3);
    out.push(scalar(n_words as u128));
    for chunk in bytes.chunks(31) {
        out.push(felt_of_bytes(chunk));
    }
    let pending = &bytes[bytes.len() - pending_len..];
    out.push(scalar(pending_len as u128));
    out.push(felt_of_bytes(pending));
    Ok(out)
}

fn felt_of_bytes(bytes: &[u8]) -> Felt {
    if bytes.is_empty() {
        return Felt::ZERO;
    }
    let mut padded = vec![0u8; 32 - bytes.len()];
    padded.extend_from_slice(bytes);
    Felt::from_bytes_be(padded.as_slice().try_into().unwrap())
}

/// felt252 encoding of an i8 (two's complement over the field prime).
pub fn enc_score(s: i8) -> Felt {
    if s < 0 {
        -Felt::from((s as i128).unsigned_abs())
    } else {
        Felt::from(s as u128)
    }
}

/// Decode score felt → i64 (negative form is `PRIME - |n|`).
pub fn dec_score(f: &Felt) -> i64 {
    let top = f.to_bytes_be();
    // Small magnitudes (±INT8 range) always have a zero first byte when
    // positive; negative encodings wrap near the prime and set high bytes.
    if top[0] != 0 {
        let neg = -f.clone();
        let bytes = neg.to_bytes_be();
        // i128 from the last 16 bytes
        let mut v: i128 = 0;
        for b in &bytes[bytes.len().saturating_sub(16)..] {
            v = (v << 8) | (*b as i128);
        }
        v as i64
    } else {
        felt_u128(f) as i64
    }
}

/// True when `felt` byte-decodes as a nonzero boolean.
pub fn dec_bool(f: &Felt) -> bool {
    felt_u128(f) != 0
}

pub fn dec_u64(f: &Felt) -> u64 {
    felt_u128(f) as u64
}

pub fn dec_u32(f: &Felt) -> u32 {
    felt_u128(f) as u32
}

pub fn dec_u16(f: &Felt) -> u16 {
    felt_u128(f) as u16
}

pub fn dec_u8(f: &Felt) -> u8 {
    felt_u128(f) as u8
}

pub fn dec_addr(out: &[Felt], i: usize) -> String {
    out.get(i).map(|f| f.to_hex_string()).unwrap_or_default()
}

pub fn dec_amount(out: &[Felt], i: usize) -> Amount {
    Amount::new(
        out.get(i).map(felt_u128).unwrap_or(0),
        out.get(i + 1).map(felt_u128).unwrap_or(0),
    )
}

/// Decode a ByteArray return: `[n_full_words, …words, pending_len, pending]`.
pub fn dec_byte_array(out: &[Felt]) -> String {
    if out.is_empty() {
        return String::new();
    }
    let n_words = felt_u128(&out[0]) as usize;
    if 1 + n_words + 2 > out.len() {
        // malformed; degrade gracefully
        return String::new();
    }
    let mut bytes = Vec::new();
    for w in &out[1..1 + n_words] {
        bytes.extend_from_slice(&w.to_bytes_be()[1..]); // full word = 31 bytes
    }
    let pending_len = felt_u128(&out[1 + n_words]) as usize;
    if pending_len > 0 {
        let w = out[2 + n_words].to_bytes_be();
        bytes.extend_from_slice(&w[32 - pending_len..]);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

// ─────────────────────────── poseidon commit ────────────────────────────────

/// The on-chain `_compute_commit_hash`: poseidon over
/// `[evaluation_id, stake_id, juror, score, salt]` (span hash). `salt` may be
/// hex or decimal.
pub fn poseidon_commit_hash(
    evaluation_id: u64,
    stake_id: u64,
    juror: &str,
    score: i8,
    salt: &str,
) -> Result<String, String> {
    use starknet_types_core::hash::Poseidon;
    use starknet_types_core::hash::StarkHash;

    let juror_felt = parse_felt(juror)?;
    let salt_felt = parse_felt(salt)?;
    let score_felt = enc_score(score);
    let ev_felt = Felt::from(evaluation_id);
    let stake_felt = Felt::from(stake_id);
    Ok(Poseidon::hash_array(&[ev_felt, stake_felt, juror_felt, score_felt, salt_felt])
        .to_hex_string())
}

// ─────────────────────────── unit conversions ───────────────────────────────

/// `get_selector_from_name` wrapper.
pub fn selector_of(name: &str) -> Felt {
    get_selector_from_name(name).unwrap_or_default()
}

/// Extract the id at a `#[key]` u256 (low, high) pair from event keys.
fn felt_pair_id(keys: &[Felt], low_index: usize) -> Option<u64> {
    let low = keys.get(low_index).map(felt_u128).unwrap_or(0);
    Some(low as u64)
}

// ───────────────────────────── Selection reads ──────────────────────────────

async fn sel_call(net: Network, ep: &str, cd: Vec<Felt>) -> Result<Vec<Felt>, String> {
    let a = addr(net, Module::Selection)?;
    chain_call(net, a, ep, cd).await
}

async fn chain_call(net: Network, a: String, ep: &str, cd: Vec<Felt>) -> Result<Vec<Felt>, String> {
    crate::chain::call(net, &a, ep, cd).await
}

pub async fn get_stake(net: Network, id: u64) -> Result<Stake, String> {
    let out = sel_call(net, "get_stake", enc_scalar(id as u128)).await?;
    if out.len() < 5 {
        return Err("unexpected get_stake result".into());
    }
    Ok(Stake {
        id: id.to_string(),
        owner: dec_addr(&out, 0),
        amount: dec_amount(&out, 1),
        created_at: dec_u64(&out[3]),
        retired: dec_bool(&out[4]),
    })
}

pub async fn get_evaluation(net: Network, id: u64) -> Result<Evaluation, String> {
    let out = sel_call(net, "get_evaluation", enc_scalar(id as u128)).await?;
    if out.len() < 20 {
        return Err("unexpected get_evaluation result".into());
    }
    Ok(Evaluation {
        id: id.to_string(),
        candidate: dec_addr(&out, 0),
        requester: dec_addr(&out, 1),
        bond_amount: dec_amount(&out, 2),
        phase: dec_u8(&out[4]),
        apply_deadline: dec_u64(&out[5]),
        draw_deadline: dec_u64(&out[6]),
        commit_deadline: dec_u64(&out[7]),
        reveal_deadline: dec_u64(&out[8]),
        slash_deadline: dec_u64(&out[9]),
        applied_count: dec_u32(&out[10]),
        drawn_count: dec_u32(&out[11]),
        min_jurors: dec_u32(&out[12]),
        final_score: dec_score(&out[13]),
        final_score_set: dec_bool(&out[14]),
        slash_pool: dec_amount(&out, 15),
        governance_tokens_minted: dec_amount(&out, 17),
        bond_settled: dec_bool(&out[19]),
    })
}

pub async fn get_dispute(net: Network, id: u64) -> Result<Dispute, String> {
    let out = sel_call(net, "get_dispute", enc_scalar(id as u128)).await?;
    if out.len() < 17 {
        return Err("unexpected get_dispute result".into());
    }
    Ok(Dispute {
        id: id.to_string(),
        candidate: dec_addr(&out, 0),
        unique_juror_count: dec_u32(&out[1]),
        commit_deadline: dec_u64(&out[2]),
        reveal_deadline: dec_u64(&out[3]),
        phase: dec_u8(&out[4]),
        final_score: dec_score(&out[5]),
        final_score_set: dec_bool(&out[6]),
        total_weight_revealed: dec_u32(&out[7]),
        total_coherent_weight: dec_u32(&out[8]),
        slash_pool: dec_amount(&out, 9),
        governance_tokens_minted: dec_amount(&out, 11),
        requester: dec_addr(&out, 13),
        bond_amount: dec_amount(&out, 14),
        bond_settled: dec_bool(&out[16]),
    })
}

pub async fn evidence_of_evaluation(net: Network, id: u64) -> Result<String, String> {
    let out = sel_call(net, "get_evaluation_evidence", enc_scalar(id as u128)).await?;
    Ok(dec_byte_array(&out))
}

pub async fn juror_stake_ids(net: Network, juror: &str) -> Result<Vec<u64>, String> {
    let cd = enc_felt(juror)?;
    let out = sel_call(net, "get_juror_stake_ids", cd).await?;
    // [len, id0_low, id0_high, id1_low, ...]
    if out.is_empty() {
        return Ok(vec![]);
    }
    let count = felt_u128(&out[0]) as usize;
    let mut ids = Vec::new();
    for k in 0..count {
        let i = 1 + k * 2;
        ids.push(felt_u128(out.get(i).unwrap_or(&Felt::ZERO)) as u64);
    }
    Ok(ids)
}

pub async fn juror_amount(net: Network, ep: &str, juror: &str) -> Result<Amount, String> {
    let out = sel_call(net, ep, enc_felt(juror)?).await?;
    Ok(dec_amount(&out, 0))
}

pub async fn juror_open_evaluations(net: Network, juror: &str) -> Result<u32, String> {
    let out = sel_call(net, "get_juror_open_evaluation_count", enc_felt(juror)?).await?;
    Ok(dec_u32(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn is_stake_drawn(net: Network, ev: u64, stake: u64) -> Result<bool, String> {
    let mut cd = vec![Felt::from(ev as u128), Felt::ZERO, Felt::from(stake as u128), Felt::ZERO];
    let out = sel_call(net, "get_is_stake_drawn", cd).await?;
    Ok(dec_bool(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn sel_scalar_u256(net: Network, ep: &str) -> Result<Amount, String> {
    let out = sel_call(net, ep, vec![]).await?;
    Ok(dec_amount(&out, 0))
}

pub async fn sel_scalar_u64(net: Network, ep: &str) -> Result<u64, String> {
    let out = sel_call(net, ep, vec![]).await?;
    Ok(dec_u64(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn sel_scalar_u32(net: Network, ep: &str) -> Result<u32, String> {
    let out = sel_call(net, ep, vec![]).await?;
    Ok(dec_u32(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn sel_scalar_u16(net: Network, ep: &str) -> Result<u16, String> {
    let out = sel_call(net, ep, vec![]).await?;
    Ok(dec_u16(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn sel_scalar_u8(net: Network, ep: &str) -> Result<u8, String> {
    let out = sel_call(net, ep, vec![]).await?;
    Ok(dec_u8(&out.get(0).unwrap_or(&Felt::ZERO)))
}

// ────────────────────────────── Governance reads ────────────────────────────

async fn gov_call(net: Network, ep: &str, cd: Vec<Felt>) -> Result<Vec<Felt>, String> {
    let a = addr(net, Module::Governance)?;
    crate::chain::call(net, &a, ep, cd).await
}

pub async fn gov_wallet_scalar(net: Network, ep: &str, holder: &str) -> Result<Amount, String> {
    let out = gov_call(net, ep, enc_felt(holder)?).await?;
    Ok(dec_amount(&out, 0))
}

pub async fn gov_conviction_power(net: Network, holder: &str, cid: u64) -> Result<Amount, String> {
    let mut cd = enc_felt(holder)?;
    cd.extend(enc_scalar(cid as u128));
    let out = gov_call(net, "get_conviction_power", cd).await?;
    Ok(dec_amount(&out, 0))
}

pub async fn get_proposal(net: Network, id: u64) -> Result<Proposal, String> {
    let out = gov_call(net, "get_proposal", enc_scalar(id as u128)).await?;
    Ok(decode_proposal(&out))
}

pub fn decode_proposal(out: &[Felt]) -> Proposal {
    if out.len() < 3 {
        return Proposal::default();
    }
    let author = dec_addr(out, 0);
    let funding_wallet = dec_addr(out, 1);
    // evidence is a ByteArray: header at [2], variable length
    let n = felt_u128(&out[2]) as usize;
    let ba_end = 2 + 1 + n + 2;
    let mut i = ba_end;
    let evidence = dec_byte_array(&out[2..ba_end]);
    let mut created = 0u64;
    let mut executed = false;
    let mut power = Amount::default();
    let mut deposit = Amount::default();
    let mut score = 0i64;
    let mut score_set = false;
    if out.len() >= i + 10 {
        created = dec_u64(&out[i]);
        i += 1;
        executed = dec_bool(&out[i]);
        i += 1;
        power = dec_amount(out, i);
        i += 2;
        deposit = dec_amount(out, i);
        i += 2;
        score = dec_score(&out[i]);
        i += 1;
        score_set = dec_bool(&out[i]);
    }
    Proposal {
        author,
        funding_wallet,
        evidence,
        created_at: created,
        executed,
        power_total: power,
        deposit_amount: deposit,
        final_score: score,
        final_score_set: score_set,
    }
}

pub async fn get_conviction(net: Network, owner: &str, cid: u64) -> Result<Conviction, String> {
    let mut cd = enc_felt(owner)?;
    cd.extend(enc_scalar(cid as u128));
    let out = gov_call(net, "get_conviction", cd).await?;
    if out.len() < 9 {
        return Err("unexpected get_conviction result".into());
    }
    Ok(Conviction {
        owner: dec_addr(&out, 0),
        id: dec_u32(&out[1]) as u64,
        level: dec_u8(&out[2]),
        amount: dec_amount(&out, 3),
        created_at: dec_u64(&out[5]),
        lock_duration: dec_u64(&out[6]),
        votes_cast: dec_u64(&out[7]),
        released: dec_bool(&out[8]),
    })
}

pub async fn gov_scalar_u256(net: Network, ep: &str) -> Result<Amount, String> {
    let out = gov_call(net, ep, vec![]).await?;
    Ok(dec_amount(&out, 0))
}

pub async fn gov_scalar_u64(net: Network, ep: &str) -> Result<u64, String> {
    let out = gov_call(net, ep, vec![]).await?;
    Ok(dec_u64(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn gov_address_of(net: Network, ep: &str) -> Result<String, String> {
    let out = gov_call(net, ep, vec![]).await?;
    Ok(dec_addr(&out, 0))
}

// ─────────────────────────── ConvictionToken reads ──────────────────────────

async fn token_call(net: Network, ep: &str, cd: Vec<Felt>) -> Result<Vec<Felt>, String> {
    let a = addr(net, Module::Token)?;
    crate::chain::call(net, &a, ep, cd).await
}

pub async fn token_balance_of(net: Network, holder: &str) -> Result<Amount, String> {
    let out = token_call(net, "balance_of", enc_felt(holder)?).await?;
    Ok(dec_amount(&out, 0))
}

/// No-arg u256 getter on the token module (alias of `token_u256`).
pub async fn token_scalar_u256(net: Network, ep: &str) -> Result<Amount, String> {
    token_u256(net, ep).await
}

pub async fn token_u256(net: Network, ep: &str) -> Result<Amount, String> {
    let out = token_call(net, ep, vec![]).await?;
    Ok(dec_amount(&out, 0))
}

pub async fn token_month_amount(net: Network, ep: &str, month: u64) -> Result<Amount, String> {
    let out = token_call(net, ep, enc_scalar(month as u128)).await?;
    Ok(dec_amount(&out, 0))
}

pub async fn token_scalar_u64(net: Network, ep: &str) -> Result<u64, String> {
    let out = token_call(net, ep, vec![]).await?;
    Ok(dec_u64(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn token_scalar_u16(net: Network, ep: &str) -> Result<u16, String> {
    let out = token_call(net, ep, vec![]).await?;
    Ok(dec_u16(&out.get(0).unwrap_or(&Felt::ZERO)))
}

pub async fn token_address(net: Network, ep: &str) -> Result<String, String> {
    let out = token_call(net, ep, vec![]).await?;
    Ok(dec_addr(&out, 0))
}

pub async fn token_modules(net: Network) -> Result<(String, String), String> {
    let out = token_call(net, "get_modules", vec![]).await?;
    Ok((dec_addr(&out, 0), dec_addr(&out, 1)))
}

// ─────────────────────────── AdminTimelock reads ────────────────────────────

async fn timelock_call(net: Network, ep: &str, cd: Vec<Felt>) -> Result<Vec<Felt>, String> {
    let a = addr(net, Module::AdminTimelock)?;
    crate::chain::call(net, &a, ep, cd).await
}

/// Generic module view dispatcher used by the data-driven admin tables.
pub async fn module_call(
    net: Network,
    module: Module,
    entrypoint: &str,
    calldata: Vec<Felt>,
) -> Result<Vec<Felt>, String> {
    let a = addr(net, module)?;
    crate::chain::call(net, &a, entrypoint, calldata).await
}


pub async fn admin_count(net: Network) -> Result<u32, String> {
    let out = timelock_call(net, "admin_count", vec![]).await?;
    Ok(dec_u32(&out.get(0).unwrap_or(&Felt::ZERO)))
}

/// felt encoding of a ≤31-char ascii short string.
fn felt_of_short_string(s: &str) -> Result<Felt, String> {
    if s.len() > 31 {
        return Err("short string too long".into());
    }
    Ok(felt_of_bytes(s.as_bytes()))
}

/// Pending timelocked numeric change for (module target, short-string key).
/// Pending changes live inside the AdminTimelock storage, keyed per target
/// contract, so both the timelock address and the parameter key are needed.
pub async fn pending_change(net: Network, m: Module, param_key: &str) -> Result<PendingChange, String> {
    let target = addr(net, m)?;
    let mut cd = enc_felt(&target)?;
    cd.push(felt_of_short_string(param_key)?);
    let out = timelock_call(net, "get_pending_change", cd).await?;
    Ok(PendingChange {
        new_value: dec_amount(&out, 0),
        effective_at: dec_u64(&out[2]),
        exists: dec_bool(&out[3]),
    })
}

/// Effective time of a queued protected-address change (0 when none).
pub async fn pending_protected_at(net: Network, target: &str) -> Result<u64, String> {
    let out = timelock_call(net, "get_pending_protected", enc_felt(target)?).await?;
    if out.len() < 2 {
        return Ok(0);
    }
    Ok(dec_u64(&out[1]))
}

pub async fn pending_admin_change(net: Network, target: &str) -> Result<PendingAdminChange, String> {
    let out = timelock_call(net, "get_pending_admin_change", enc_felt(target)?).await?;
    if out.len() < 3 {
        return Ok(PendingAdminChange::default());
    }
    Ok(PendingAdminChange {
        is_add: dec_bool(&out[0]),
        effective_at: dec_u64(&out[1]),
        exists: dec_bool(&out[2]),
    })
}

pub async fn pending_upgrade(net: Network, target: &str) -> Result<PendingUpgrade, String> {
    let out = timelock_call(net, "get_pending_upgrade", enc_felt(target)?).await?;
    if out.len() < 3 {
        return Ok(PendingUpgrade::default());
    }
    Ok(PendingUpgrade {
        class_hash: dec_addr(&out, 0),
        effective_at: dec_u64(&out[1]),
        exists: dec_bool(&out[2]),
    })
}

pub async fn upgrades_disabled(net: Network, target: &str) -> Result<bool, String> {
    let out = timelock_call(net, "is_upgrades_disabled", enc_felt(target)?).await?;
    Ok(dec_bool(&out.get(0).unwrap_or(&Felt::ZERO)))
}

// ─────────────────────────── event-driven lists ─────────────────────────────

/// Evaluation ids from `EvaluationRequested` keys `[sel, ev_low, ev_high, candidate]`.
pub async fn list_evaluation_ids(net: Network) -> Result<Vec<u64>, String> {
    let addr_v = addr(net, Module::Selection)?;
    let lookback = crate::chain::current_block_number(net)
        .await
        .ok()
        .map(|bn| bn.saturating_sub(crate::config::EVENT_LOOKBACK));
    let events = crate::chain::events_for_event_name(net, &addr_v, "EvaluationRequested", lookback).await?;
    let mut ids = Vec::new();
    for e in &events {
        if let Some(id) = felt_pair_id(&e.keys, 1) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids.sort_unstable();
    ids.reverse();
    Ok(ids)
}

/// Proposal ids from `ProposalCreated` keys `[sel, id_low, id_high, author]`.
pub async fn list_proposal_ids(net: Network) -> Result<Vec<u64>, String> {
    let a = addr(net, Module::Governance)?;
    let lookback = crate::chain::current_block_number(net)
        .await
        .ok()
        .map(|bn| bn.saturating_sub(crate::config::EVENT_LOOKBACK));
    let events = crate::chain::events_for_event_name(net, &a, "ProposalCreated", lookback).await?;
    let mut ids = Vec::new();
    for e in &events {
        if let Some(id) = felt_pair_id(&e.keys, 1) {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    ids.sort_unstable();
    ids.reverse();
    Ok(ids)
}

/// Conviction ids owned by `owner`, from `ConvictionCreated` keys `[sel, owner, id]`.
pub async fn list_conviction_ids(net: Network, owner: &str) -> Result<Vec<u64>, String> {
    let a = addr(net, Module::Governance)?;
    let lookback = crate::chain::current_block_number(net)
        .await
        .ok()
        .map(|bn| bn.saturating_sub(crate::config::EVENT_LOOKBACK));
    let events = crate::chain::events_for_event_name(net, &a, "ConvictionCreated", lookback).await?;
    let owner_felt = parse_felt(owner).ok();
    let mut ids = Vec::new();
    for e in &events {
        if e.keys.len() < 3 {
            continue;
        }
        let ev_owner = e.keys[1].to_hex_string();
        let same = match (&owner_felt, Felt::from_hex(&ev_owner)) {
            (Some(a), Ok(b)) => a == &b,
            _ => false,
        };
        if !same {
            continue;
        }
        let id = felt_u128(&e.keys[2]) as u64;
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids.sort_unstable();
    Ok(ids)
}

/// (stake_id, juror) draw records for an evaluation from `JurorDrawn` events.
pub async fn drawn_jurors(net: Network, ev: u64) -> Result<Vec<(u64, String)>, String> {
    let a = addr(net, Module::Selection)?;
    let lookback = crate::chain::current_block_number(net)
        .await
        .ok()
        .map(|bn| bn.saturating_sub(crate::config::EVENT_LOOKBACK));
    let events = crate::chain::events_for_event_name(net, &a, "JurorDrawn", lookback).await?;
    let mut out = Vec::new();
    for e in &events {
        if let Some(id) = felt_pair_id(&e.keys, 1) {
            if id != ev || e.keys.len() < 6 {
                continue;
            }
            let stake = felt_u128(&e.keys[3]) as u64;
            let juror = e.keys[5].to_hex_string();
            if !out.iter().any(|(s, _)| *s == stake) {
                out.push((stake, juror));
            }
        }
    }
    Ok(out)
}
