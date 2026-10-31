//! Chain logic built on starknet-rust: read-only JSON-RPC calls, event
//! discovery, and the encoding/formatting helpers used by every page.
//!
//! Signing is NOT done here — the browser wallet signs and submits
//! transactions (see `wallet.rs` / `assets/wallet.js`).

use starknet_rust::core::types::{
    AddressFilter, BlockId, BlockTag, EmittedEvent, EventFilter, Felt, FunctionCall,
};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_rust::providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider};
use url::Url;

use crate::config::Network;
use crate::types::Amount;

pub const TOKEN_DECIMALS: u32 = 18;

fn provider_for(rpc_url: &str) -> Result<JsonRpcClient<HttpTransport>, String> {
    let url = Url::parse(rpc_url).map_err(|e| format!("bad rpc url: {e}"))?;
    Ok(JsonRpcClient::new(HttpTransport::new(url)))
}

// ────────────────────────────── values / felts ──────────────────────────────

/// Parse a hex or decimal felt value ("0x..." / "123").
pub fn parse_felt(s: &str) -> Result<Felt, String> {
    Felt::from_hex(s)
        .or_else(|_| s.parse::<u128>().map(Felt::from).map_err(|_| ()))
        .map_err(|_| format!("invalid value: {s}"))
}

/// True when `value` is a plausible non-zero contract address.
pub fn is_valid_address(value: &str) -> bool {
    Felt::from_hex(value.trim()).map(|f| f != Felt::ZERO).unwrap_or(false)
}

/// Extract the bottom u128 limb of a felt.
pub fn felt_u128(f: &Felt) -> u128 {
    let bytes = f.to_bytes_be();
    u128::from_be_bytes(bytes[16..32].try_into().unwrap())
}

/// `0x` hex without leading zeros.
pub fn normalized_hex(address: &str) -> String {
    match Felt::from_hex(address.trim()) {
        Ok(f) => f.to_hex_string(),
        Err(_) => address.trim().to_string(),
    }
}

/// `0x1234…abcd` short display.
pub fn short_address(address: &str, size: usize) -> String {
    let norm = normalized_hex(address);
    if norm.len() <= size * 2 + 2 {
        return norm;
    }
    format!("{}…{}", &norm[..size + 2], &norm[norm.len() - size..])
}

// ─────────────────────────── formatting Amounts ─────────────────────────────

/// Long-division of the u256 (low, high) limbs by base 10^9, producing the
/// plain decimal digits of the raw token amount.
pub fn u256_decimal_string(value: Amount) -> String {
    let mut limbs: [u64; 4] = [
        (value.low & u128::from(u64::MAX)) as u64,
        (value.low >> 64) as u64,
        (value.high & u128::from(u64::MAX)) as u64,
        (value.high >> 64) as u64,
    ];
    if limbs.iter().all(|&l| l == 0) {
        return "0".to_string();
    }
    let base: u128 = 1_000_000_000;
    let mut chunks: Vec<u32> = Vec::new();
    while limbs.iter().any(|&l| l != 0) {
        let mut rem = 0u128;
        for limb in limbs.iter_mut().rev() {
            // rem < 10^9 and limb < 2^64 so rem<<64 | limb < 2^94 — fits u128.
            let cur = (rem << 64) | (*limb as u128);
            *limb = (cur / base) as u64;
            rem = cur % base;
        }
        chunks.push(rem as u32);
    }
    if chunks.is_empty() {
        return "0".into();
    }
    // chunks are ordered least-significant first; most significant chunk is
    // printed without padding, the rest use fixed-width 9 digits.
    let mut s = String::new();
    for (i, chunk) in chunks.iter().enumerate().rev() {
        let text = chunk.to_string();
        if i != chunks.len() - 1 {
            for _ in 0..(9 - text.len()) {
                s.push('0');
            }
        }
        s.push_str(&text);
    }
    s
}

/// Raw amount (18 decimals) → human string like `12,345.67` (grouped).
pub fn format_amount(value: Amount, display_dp: usize) -> String {
    let raw = u256_decimal_string(value);
    if raw == "0" {
        return "0".into();
    }
    let dp = TOKEN_DECIMALS as usize;
    let digits = raw.len();
    let (whole, frac_raw) = if digits <= dp {
        ("0".to_string(), format!("{:0>width$}", raw, width = dp))
    } else {
        (raw[..digits - dp].to_string(), raw[digits - dp..].to_string())
    };
    let mut frac: String = frac_raw.chars().take(display_dp).collect();
    while frac.ends_with('0') {
        frac.pop();
    }
    let whole = group_thousands(&whole);
    if frac.is_empty() {
        whole
    } else {
        format!("{whole}.{frac}")
    }
}

/// Compact stat formatting (`12.3K`, `4.56M`, `7.89B`).
pub fn compact_amount(value: Amount) -> String {
    let tokens = amount_whole_tokens(value);
    if tokens < 1_000.0 {
        format_amount(value, 2)
    } else if tokens < 1_000_000.0 {
        format!("{:.1}K", tokens / 1_000.0)
    } else if tokens < 1_000_000_000.0 {
        format!("{:.2}M", tokens / 1_000_000.0)
    } else {
        format!("{:.2}B", tokens / 1_000_000_000.0)
    }
}

fn amount_whole_tokens(value: Amount) -> f64 {
    let units = u256_to_f64(value, TOKEN_DECIMALS);
    units
}

fn u256_to_f64(value: Amount, dp: u32) -> f64 {
    // f64 has 53-bit mantissa; lossy conversion is fine for stat cards.
    let base = 10f64.powi(dp as i32);
    let low = value.low as f64 / base;
    let high_shift = 2f64.powi(128);
    let high = value.high as f64 * high_shift / base;
    low + high
}

fn group_thousands(digits: &str) -> String {
    let mut out = String::new();
    let len = digits.len();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

// ───────────────────────────── RPC reads ────────────────────────────────────

/// Read-only contract call against the network's public RPC.
pub async fn call(
    net: Network,
    addr: &str,
    entrypoint: &str,
    calldata: Vec<Felt>,
) -> Result<Vec<Felt>, String> {
    let address = Felt::from_hex(addr).map_err(|e| format!("bad address {addr}: {e}"))?;
    let selector = get_selector_from_name(entrypoint).map_err(|e| format!("selector: {e}"))?;
    provider_for(net.rpc_url())?
        .call(
            FunctionCall {
                contract_address: address,
                entry_point_selector: selector,
                calldata,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .map_err(|e| format!("rpc error: {e}"))
}

/// STRK gas-token `balanceOf(owner)`.
pub async fn strk_balance(net: Network, owner: &str, token: &str) -> Result<Amount, String> {
    let owner = Felt::from_hex(owner).map_err(|e| format!("bad address: {e}"))?;
    let token_felt = Felt::from_hex(token).map_err(|e| format!("bad token: {e}"))?;
    let selector = get_selector_from_name("balanceOf").map_err(|e| format!("selector: {e}"))?;
    let out = provider_for(net.rpc_url())?
        .call(
            FunctionCall {
                contract_address: token_felt,
                entry_point_selector: selector,
                calldata: vec![owner],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .map_err(|e| format!("rpc error: {e}"))?;
    if out.len() < 2 {
        return Err("unexpected balanceOf result".into());
    }
    Ok(Amount::new(felt_u128(&out[0]), felt_u128(&out[1])))
}

/// Latest block number, used to bound event history scans.
pub async fn current_block_number(net: Network) -> Result<u64, String> {
    provider_for(net.rpc_url())?
        .block_number()
        .await
        .map_err(|e| format!("block_number: {e}"))
}

// ─────────────────────────── event queries ──────────────────────────────────

/// All events of `event_name` emitted by `contract`, walking continuation
/// pages from `from_block` (if any).
pub async fn events_for_event_name(
    net: Network,
    contract: &str,
    event_name: &str,
    from_block: Option<u64>,
) -> Result<Vec<EmittedEvent>, String> {
    let contract_felt =
        Felt::from_hex(contract).map_err(|e| format!("bad address {contract}: {e}"))?;
    let key = get_selector_from_name(event_name).map_err(|e| format!("selector: {e}"))?;
    let filter = EventFilter {
        from_block: from_block.map(BlockId::Number),
        to_block: None,
        address: Some(AddressFilter::Single(contract_felt)),
        keys: Some(vec![vec![key]]),
    };
    let provider = provider_for(net.rpc_url())?;
    let mut all = Vec::new();
    let mut continuation: Option<String> = None;
    for _ in 0..40 {
        let page = provider
            .get_events(filter.clone(), continuation, 150)
            .await
            .map_err(|e| format!("get_events: {e}"))?;
        all.extend(page.events);
        match page.continuation_token {
            Some(tok) => continuation = Some(tok),
            None => break,
        }
    }
    Ok(all)
}

// ─────────────────────── wall clock / dates / durations ─────────────────────

/// Seconds since the Unix epoch (browser wall clock).
pub fn now_seconds() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

/// Locale date-time via the JS Date object.
pub fn date_string(seconds: u64) -> String {
    if seconds == 0 {
        return "—".into();
    }
    let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(seconds as f64 * 1000.0));
    d.to_locale_string("en-US", &wasm_bindgen::JsValue::UNDEFINED)
        .as_string()
        .unwrap_or_default()
}

/// Compact human duration: `3d 4h`, `12h 5m`, `9m 30s`.
pub fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        return "0s".into();
    }
    let d = seconds / 86_400;
    let h = (seconds % 86_400) / 3_600;
    let m = (seconds % 3_600) / 60;
    let s = seconds % 60;
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}

/// Countdown relative to `now`; `done` when the deadline is in the past.
pub fn countdown(target: u64, now: u64) -> String {
    if target <= now {
        "done".into()
    } else {
        format_duration(target - now)
    }
}

// ─────────────────────── input parsing (calldata) ───────────────────────────

/// Decimal token input ("1.25") → wei u128 with 18 decimals. None when invalid.
pub fn parse_amount(input: &str) -> Option<u128> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (whole, frac) = trimmed.split_once('.').unwrap_or((trimmed, ""));
    if whole.is_empty() && frac.is_empty() {
        return None;
    }
    if !whole.chars().all(|c| c.is_ascii_digit()) || !frac.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let dp = TOKEN_DECIMALS as usize;
    if frac.len() > dp {
        return None;
    }
    let whole_u: u128 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let mut frac_padded = frac.to_string();
    while frac_padded.len() < dp {
        frac_padded.push('0');
    }
    let frac_u: u128 = if frac_padded.is_empty() {
        0
    } else {
        frac_padded.parse().ok()?
    };
    let base = 10u128.pow(dp as u32);
    whole_u.checked_mul(base)?.checked_add(frac_u)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(low: u128, high: u128) -> Amount {
        Amount::new(low, high)
    }

    #[test]
    fn u256_decimal_string_small() {
        assert_eq!(u256_decimal_string(amount(0, 0)), "0");
        assert_eq!(u256_decimal_string(amount(1, 0)), "1");
        assert_eq!(u256_decimal_string(amount(999_999_999, 0)), "999999999");
        assert_eq!(u256_decimal_string(amount(1_000_000_000, 0)), "1000000000");
        let x = u256_decimal_string(amount(0, 1));
        assert_eq!(x.len(), 39); // 2^128 = 340282366920938463463374607431768211456
        assert_eq!(x, "340282366920938463463374607431768211456");
    }

    #[test]
    fn u256_decimal_string_multi_chunk() {
        // 10^27 = 1000000000000000000000000000 spans three 9-digit chunks
        let ten27 = amount(10u128.pow(27), 0);
        assert_eq!(u256_decimal_string(ten27), "1000000000000000000000000000");
    }

    #[test]
    fn format_amount_roundtrip() {
        assert_eq!(format_amount(amount(0, 0), 2), "0");
        assert_eq!(format_amount(amount(1_500_000_000_000_000_000, 0), 2), "1.5");
        assert_eq!(format_amount(amount(1_234_500_000_000_000_000, 0), 2), "1.2345");
        let one_token = parse_amount("1").unwrap();
        assert_eq!(format_amount(amount(one_token, 0), 2), "1");
    }

    #[test]
    fn parse_amount_validates() {
        assert_eq!(parse_amount("1.5"), Some(1_500_000_000_000_000_000));
        assert_eq!(parse_amount("abc"), None);
        assert_eq!(parse_amount(""), None);
        assert_eq!(parse_amount("1.9999999999999999999"), None);
    }

    #[test]
    fn short_address_shapes() {
        assert_eq!(
            short_address("0x00000000000000000000000000000000000000000000000000000000000abc12", 4),
            "0xabc12"
        );
    }
}
