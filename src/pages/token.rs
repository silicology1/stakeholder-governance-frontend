//! ConvictionToken page: transferable STK economics, decay, and burn.

use dioxus::prelude::*;

use crate::chain::{compact_amount, format_amount};
use crate::contracts;
use crate::wallet;
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;

type Res<T> = Resource<Result<T, String>>;

#[component]
pub fn TokenPage() -> Element {
    let app: App = use_context();
    let net = app.network;

    let month = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_scalar_u64(net, "get_current_month").await }
    });
    let total = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_scalar_u256(net, "total_supply").await }
    });
    let max_supply = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_u256(net, "token_max_supply").await }
    });
    let remaining = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_u256(net, "token_remaining_mintable").await }
    });
    let month_budget = use_resource(move || {
        let net = *net.read();
        let m = month_val(&month);
        async move { contracts::token_month_amount(net, "get_month_budget", m).await }
    });
    let month_minted = use_resource(move || {
        let net = *net.read();
        let m = month_val(&month);
        async move { contracts::token_month_amount(net, "get_month_budget_minted", m).await }
    });
    let month_burned = use_resource(move || {
        let net = *net.read();
        let m = month_val(&month);
        async move { contracts::token_month_amount(net, "get_month_burned", m).await }
    });
    let remaining_budget = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_u256(net, "get_remaining_month_budget").await }
    });
    let decay = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_scalar_u16(net, "get_decay_rate_bps").await }
    });
    let inflation = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_scalar_u16(net, "get_inflation_rate_bps").await }
    });
    let min_decay = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_u256(net, "get_min_decay_for_reward").await }
    });
    let caller_reward = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_u256(net, "get_caller_reward_amount").await }
    });

    rsx! {
        div { class: "animate-fade-in-up space-y-8",
            div {
                h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white", "ConvictionToken" }
                p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                    "The transferable STK token: hourly balance decay, a monthly mint budget, and the escrow hub custodying every stake, bond, and proposal deposit."
                }
            }

            div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
                Stat { label: "Total supply", value: "{amt(&total)}", sub: "circulating STK", icon: IconName::Coins, accent: Accent::Orange }
                Stat { label: "Max supply", value: "{amt(&max_supply)}", sub: "hard cap", icon: IconName::Landmark, accent: Accent::Blue }
                Stat { label: "Remaining mintable", value: "{amt(&remaining)}", sub: "before the cap", icon: IconName::PieChart, accent: Accent::Emerald }
                Stat { label: "Current month", value: "#{show(&month)}", sub: "{show(&inflation)} bps monthly inflation", icon: IconName::Calendar, accent: Accent::Violet }
            }

            Card { class: "",
                SectionTitle {
                    eyebrow: "Treasury".to_string(),
                    title: "Monthly emission budget".to_string(),
                    description: "Rewards mint from a per-month budget; burned tokens are tracked for recycling but unspent budget does not roll over to the next month.".to_string(),
                    class: "!mb-4".to_string(),
                }
                div { class: "grid gap-4 sm:grid-cols-3",
                    Metric { label: "Budget", value: "{amt(&month_budget)}" }
                    Metric { label: "Minted", value: "{amt(&month_minted)}" }
                    Metric { label: "Burned", value: "{amt(&month_burned)}" }
                }
                div { class: "mt-4 text-xs text-zinc-400",
                    "{amt(&remaining_budget)} still spendable this month · decay {show(&decay)} bps/hour"
                }
            }

            DecayCard { key: "{net.read().key()}" }

            TransferCard { key: "{net.read().key()}" }

            div { class: "grid gap-6 lg:grid-cols-2",
                Card { class: "space-y-3",
                    h3 { class: "flex items-center gap-2 font-bold text-zinc-900 dark:text-white",
                        Icon { name: IconName::Flame, class: "h-4 w-4 text-orange-500" }
                        "Hourly decay"
                    }
                    p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "Balances shrink {show(&decay)} bps every hour unless the holder is protected. Anyone can call apply_decay: the accrued amount is burned and qualifying calls pay a caller reward of {amt(&caller_reward)}."
                    }
                    p { class: "text-xs text-zinc-400",
                        "Caller rewards require burning at least {amt(&min_decay)} in one call (24h cooldown)."
                    }
                }
                Card { class: "space-y-3",
                    h3 { class: "flex items-center gap-2 font-bold text-zinc-900 dark:text-white",
                        Icon { name: IconName::Percent, class: "h-4 w-4 text-orange-500" }
                        "Two balance systems"
                    }
                    p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "The transferable STK here is used for staking, rewards, and deposits — it decays and can move between wallets. Governance tokens minted by Selection for coherent jurors are non-transferable and power conviction voting."
                    }
                }
            }
        }
    }
}

fn month_val(res: &Res<u64>) -> u64 {
    res.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or(0)
}

fn show<T: ToString>(res: &Res<T>) -> String {
    match &*res.read() {
        Some(Ok(v)) => v.to_string(),
        Some(Err(e)) => format!("⚠ {e}"),
        None => "…".to_string(),
    }
}

fn amt(res: &Res<Amount>) -> String {
    match &*res.read() {
        Some(Ok(a)) => format_amount(*a, 10),
        Some(Err(_)) => "—".to_string(),
        None => "…".to_string(),
    }
}

fn compact(res: &Res<Amount>) -> String {
    match &*res.read() {
        Some(Ok(a)) => compact_amount(*a),
        _ => "—".to_string(),
    }
}

// ─────────────────────────── decay + burn actions ───────────────────────────

#[component]
fn DecayCard() -> Element {
    let app: App = use_context();
    let net = app.network;
    let addr = app.normalized_address().unwrap_or_default();
    let mut tx = use_signal(|| TxState::Idle);
    let mut amount = use_signal(String::new);

    let cl_addr = addr.clone();
    let balance = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::token_balance_of(net, &addr).await }
    });

    let do_decay = move |_| {
        if app.address().is_none() {
            return;
        }
        let a = contracts::addr(*net.read(), contracts::Module::Token).unwrap_or_default();
        let Ok(f) = contracts::enc_felt(&addr) else { return };
        let cd = contracts::calldata_json(&f);
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, "apply_decay", &cd).await {
                Ok(_hash) => tx.set(TxState::Sent(String::new(), "Decay applied — the burned amount is recorded for the month budget.".to_string())),
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    let do_burn = move |_| {
        let Some(wei) = crate::chain::parse_amount(&*amount.read()) else { return };
        let a = contracts::addr(*net.read(), contracts::Module::Token).unwrap_or_default();
        let cd = contracts::calldata_json(&contracts::enc_u256(Amount::from_low(wei as u128)));
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, "burn", &cd).await {
                Ok(_hash) => {
                    amount.set(String::new());
                    tx.set(TxState::Sent(String::new(), "Tokens burned — the burned amounts feed the monthly budget accounting.".to_string()))
                },
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    rsx! {
        Card { class: "",
            SectionTitle {
                eyebrow: "Your wallet".to_string(),
                title: "Decay & burn".to_string(),
                description: "Applying decay to your own unprotected address advances the token inflation accounting. Burning is open directly to holders.".to_string(),
                class: "!mb-4".to_string(),
            }
            if app.connected() {
                div { class: "grid gap-6 lg:grid-cols-2",
                    div { class: "space-y-3",
                        div { class: "rounded-xl bg-zinc-50 p-3 dark:bg-zinc-800",
                            div { class: "text-xs text-zinc-400", "STK balance" }                            div { class: "text-lg font-black text-zinc-900 dark:text-white",
                                "{bal_str(&balance)}"
                            }
                        }
                        Button { onclick: click(Box::new(do_decay)), variant: BtnVariant::Secondary, size: "md".to_string(), disabled: Some(tx.read().is_pending()), class: "".to_string(),
                            Icon { name: IconName::Flame, class: "h-4 w-4" }
                            "Apply decay to my address"
                        }
                    }
                    div { class: "space-y-3",
                        Label { hint: "18 decimals applied automatically", "Amount to burn" }
                        Input { oninput: on_input(move |e: Event<FormData>| amount.set(e.value())), value: "{amount.read()}", placeholder: "0.0".to_string(), r#type: "text".to_string() }
                        Button { onclick: click(Box::new(do_burn)), variant: BtnVariant::Outline, size: "md".to_string(), disabled: Some(tx.read().is_pending() || crate::chain::parse_amount(&*amount.read()).unwrap_or(0) == 0), class: "".to_string(),
                            "Burn"
                        }
                    }
                }
            } else {
                ConnectRequired { "Connect a wallet to apply decay or burn." }
            }
            div { class: "mt-3", TxStatus { state: tx.read().clone() } }
        }
    }
}

// ─────────────────────────── transfer card ──────────────────────────────────

/// Wallet-signed ERC20 `transfer` of the app token (STK).
#[component]
fn TransferCard() -> Element {
    let app: App = use_context();
    let net = app.network;
    let mut tx = use_signal(|| TxState::Idle);
    let mut recipient = use_signal(String::new);
    let mut amount = use_signal(String::new);

    let configured = contracts::is_configured(*net.read(), contracts::Module::Token);

    let valid = configured
        && crate::chain::parse_amount(&*amount.read()).unwrap_or(0) > 0
        && crate::chain::parse_felt(&*recipient.read()).is_ok();

    let do_transfer = move |_| {
        let Some(wei) = crate::chain::parse_amount(&*amount.read()) else { return };
        let Ok(rcp) = crate::chain::parse_felt(&*recipient.read()) else { return };
        let Ok(a) = contracts::addr(*net.read(), contracts::Module::Token) else { return };
        let mut parts = vec![rcp];
        parts.extend(contracts::enc_u256(Amount::from_low(wei)));
        let cd = contracts::calldata_json(&parts);
        let contract = a;
        let mut recipient = recipient.clone();
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&contract, "transfer", &cd).await {
                Ok(_hash) => {
                    recipient.set(String::new());
                    tx.set(TxState::Sent(String::new(), "Transfer signed by your wallet — STK is on its way.".to_string()))
                }
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    rsx! {
        Card { class: "",
            SectionTitle {
                eyebrow: "Your wallet".to_string(),
                title: "Transfer STK".to_string(),
                description: "Standard ERC20 transfer, signed by your connected Starknet wallet. Decimals are applied automatically.".to_string(),
                class: "!mb-4".to_string(),
            }
            if app.connected() {
                div { class: "grid gap-4 sm:grid-cols-2",
                    div { class: "space-y-2",
                        Label { hint: "0x… recipient address", "Send to" }
                        Input {
                            oninput: on_input(move |e: Event<FormData>| recipient.set(e.value())),
                            value: "{recipient.read()}",
                            placeholder: "0x…".to_string(),
                            r#type: "text".to_string(),
                        }
                    }
                    div { class: "space-y-2",
                        Label { hint: "18 decimals applied automatically", "Amount (STK)" }
                        Input {
                            oninput: on_input(move |e: Event<FormData>| amount.set(e.value())),
                            value: "{amount.read()}",
                            placeholder: "0.0".to_string(),
                            r#type: "text".to_string(),
                        }
                    }
                }
                Button {
                    onclick: click(Box::new(do_transfer)),
                    variant: BtnVariant::Primary,
                    size: "md".to_string(),
                    disabled: Some(tx.read().is_pending() || !valid),
                    class: "mt-4".to_string(),
                    Icon { name: IconName::Wallet, class: "h-4 w-4" }
                    "Send STK"
                }
                if !configured {
                    p { class: "mt-2 text-sm text-amber-600 dark:text-amber-400",
                        "ConvictionToken is not deployed on {net.read().label()} yet — transfers are disabled."
                    }
                }
            } else {
                ConnectRequired { "Connect a wallet to transfer STK." }
            }
            div { class: "mt-3", TxStatus { state: tx.read().clone() } }
        }
    }
}

fn bal_str(res: &Res<Amount>) -> String {
    match &*res.read() {
        Some(Ok(a)) => format_amount(*a, 10),
        _ => "—".to_string(),
    }
}

