//! Bridge to the browser wallet through assets/wallet.js.
//!
//! The wallet signs and submits every transaction; Rust builds the raw
//! calldata (hex-felt JSON, see `contracts.rs`) and lets the extension do the
//! ABI encoding as required by `wallet_addInvokeTransaction`.

use js_sys::{JSON, Reflect};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/wallet.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = list_wallets)]
    async fn js_list_wallets() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn connect_wallet(wallet_id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn send_invoke(
        contract_address: &str,
        entry_point: &str,
        calldata_json: &str,
    ) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct WalletOption {
    pub id: String,
    pub name: String,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Connected {
    pub wallet: String,
    pub address: String,
    pub chain_id: String,
}

fn js_err(e: JsValue) -> String {
    if let Some(s) = e.as_string() {
        return s;
    }
    Reflect::get(&e, &JsValue::from_str("message"))
        .ok()
        .and_then(|m| m.as_string())
        .unwrap_or_else(|| "wallet request failed or was rejected".into())
}

fn get_str(obj: &JsValue, key: &str) -> Result<String, String> {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
        .ok_or_else(|| format!("wallet response missing `{key}`"))
}

/// Discover injected Starknet wallet extensions.
pub async fn list() -> Result<Vec<WalletOption>, String> {
    let v = js_list_wallets().await.map_err(js_err)?;
    let text = v.as_string().ok_or("unexpected wallet response")?;
    let arr = JSON::parse(&text).map_err(js_err)?;
    let mut out = Vec::new();
    for entry in js_sys::Array::from(&arr).values() {
        let entry = entry.map_err(js_err)?;
        out.push(WalletOption {
            id: get_str(&entry, "id")?,
            name: get_str(&entry, "name")?,
            icon: get_str(&entry, "icon").unwrap_or_default(),
        });
    }
    Ok(out)
}

/// Ask the wallet to connect (the user approves in the wallet popup).
pub async fn connect(wallet_id: &str) -> Result<Connected, String> {
    let v = connect_wallet(wallet_id).await.map_err(js_err)?;
    let text = v.as_string().ok_or("unexpected wallet response")?;
    let obj = JSON::parse(&text).map_err(js_err)?;
    Ok(Connected {
        wallet: get_str(&obj, "wallet")?,
        address: get_str(&obj, "address")?,
        chain_id: get_str(&obj, "chainId")?,
    })
}

/// Submit a single-call invoke through the wallet. Returns the transaction hash.
pub async fn invoke(
    contract: &str,
    entry_point: &str,
    calldata_json: &str,
) -> Result<String, String> {
    let v = send_invoke(contract, entry_point, calldata_json)
        .await
        .map_err(js_err)?;
    v.as_string()
        .ok_or_else(|| "wallet returned no transaction hash".into())
}
