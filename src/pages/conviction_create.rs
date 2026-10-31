//! Conviction creation: lock governance tokens at a chosen level (1–10).

use dioxus::prelude::*;

use crate::chain::{format_amount, parse_amount};
use crate::contracts;
use crate::wallet;
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;

#[component]
pub fn ConvictionsCreate() -> Element {
    let app: App = use_context();
    let net = app.network;
    let mut tx = use_signal(|| TxState::Idle);
    let mut level = use_signal(|| String::from("5"));
    let mut amount = use_signal(String::new);

    let available = use_resource(move || {
        let net = *net.read();
        let owner = app.normalized_address().unwrap_or_default();
        async move { contracts::gov_wallet_scalar(net, "governance_available", &owner).await }
    });
    let base_lock = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u64(net, "get_conviction_base_lock").await }
    });

    let submit = move |_| {
        let Some(wei) = parse_amount(&*amount.read()) else {
            tx.set(TxState::Failed("Enter an amount in governance tokens.".to_string()));
            return;
        };
        let Ok(lvl) = level.read().parse::<u8>() else {
            tx.set(TxState::Failed("Level must be 1–10.".to_string()));
            return;
        };
        if !(1..=10).contains(&lvl) {
            tx.set(TxState::Failed("Level must be 1–10.".to_string()));
            return;
        }
        let Ok(a) = contracts::addr(*net.read(), contracts::Module::Governance) else { return };
        let mut parts = vec![starknet_rust::core::types::Felt::from(lvl as u128)];
        parts.extend(contracts::enc_u256(Amount::from_low(wei as u128)));
        let cd = contracts::calldata_json(&parts);
        let mut amount = amount.clone();
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, "create_conviction", &cd).await {
                Ok(_h) => {
                    amount.set(String::new());
                    tx.set(TxState::Sent(String::new(), "Conviction created — tokens are locked and power accrues over time.".to_string()))
                }
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    rsx! {
        div { class: "animate-fade-in-up mx-auto max-w-2xl space-y-6",
            Link {
                to: crate::Route::Convictions {},
                class: "inline-flex items-center gap-1.5 text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white",
                Icon { name: IconName::ArrowLeft, class: "h-4 w-4" }
                "Back to convictions"
            }
            Card { class: "space-y-4",
                h1 { class: "flex items-center gap-2 text-xl font-black text-zinc-900 dark:text-white",
                    Icon { name: IconName::Vote, class: "h-5 w-5 text-orange-500" }
                    "Create conviction"
                }
                p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                    "Lock available governance tokens at a level from 1 to 10. Higher levels mean more power per vote but a longer lock (base {lock_str(&base_lock)}, scaled by level)."
                }
                if app.connected() {
                    div { class: "rounded-xl bg-zinc-50 p-3 text-sm dark:bg-zinc-800",
                        span { class: "text-zinc-500 dark:text-zinc-400", "Available governance tokens: " }
                        strong { class: "text-zinc-900 dark:text-white", "{amt(&available)}" }
                    }
                    Label { hint: "1 = minimal lock, 10 = maximal power", "Level" }
                    Input {
                        oninput: on_input(move |e: Event<FormData>| level.set(e.value())),
                        value: "{level.read()}",
                        placeholder: "5".to_string(),
                        r#type: "text".to_string(),
                    }
                    Label { hint: String::new(), "Amount of governance tokens" }
                    Input {
                        oninput: on_input(move |e: Event<FormData>| amount.set(e.value())),
                        value: "{amount.read()}",
                        placeholder: "0.0".to_string(),
                        r#type: "text".to_string(),
                    }
                    Button {
                        onclick: click(Box::new(submit)),
                        variant: BtnVariant::Primary,
                        size: "md".to_string(),
                        disabled: Some(tx.read().is_pending() || parse_amount(&*amount.read()).unwrap_or(0) == 0),
                        class: String::new(),
                        "Create conviction"
                    }
                } else {
                    ConnectRequired { "Connect a wallet that holds governance tokens to create a conviction." }
                }
                div { TxStatus { state: tx.read().clone() } }
            }
        }
    }
}

fn amt(res: &Resource<Result<Amount, String>>) -> String {
    match &*res.read() {
        Some(Ok(a)) => format_amount(*a, 10),
        _ => "…".to_string(),
    }
}

fn lock_str(res: &Resource<Result<u64, String>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => crate::chain::format_duration(*v),
        _ => "…".to_string(),
    }
}
