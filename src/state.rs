//! Shared UI state: the selected network, the connected wallet, and a
//! 1-second wall-clock tick signal for countdowns.

use dioxus::prelude::*;

use crate::config::Network;
use crate::wallet;

/// Everything pages reach for through `use_context`.
#[derive(Clone, Copy)]
pub struct App {
    pub network: Signal<Network>,
    pub account: Signal<Option<wallet::Connected>>,
    pub now: Signal<u64>,
}

impl App {
    pub fn connected(&self) -> bool {
        self.account.read().is_some()
    }

    pub fn address(&self) -> Option<String> {
        self.account.read().as_ref().map(|c| c.address.clone())
    }

    pub fn normalized_address(&self) -> Option<String> {
        self.account
            .read()
            .as_ref()
            .map(|c| crate::chain::normalized_hex(&c.address))
    }

    /// True when the wallet is on a different network than the UI selector.
    pub fn chain_mismatch(&self) -> bool {
        self.account
            .read()
            .as_ref()
            .map(|c| !c.chain_id.eq_ignore_ascii_case(self.network.read().chain_id_hex()))
            .unwrap_or(false)
    }
}

/// Copy text to the browser clipboard (fire-and-forget).
pub fn copy_to_clipboard(text: &str) {
    let text = text.replace('\\', "\\\\").replace('\'', "\\'");
    spawn(async move {
        let _ = dioxus::document::eval(&format!("navigator.clipboard.writeText('{text}')"));
    });
}

/// A short-lived "copied" flag that resets after 1.5 seconds.
pub async fn timer_reset(mut copied: Signal<bool>) {
    gloo_timers::future::TimeoutFuture::new(1_500).await;
    copied.set(false);
}
