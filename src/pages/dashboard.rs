//! Protocol + wallet snapshot across the four modules.

use dioxus::prelude::*;

use crate::chain::{format_amount, short_address};
use crate::contracts::{self, Module};
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;
use crate::wallet;

#[component]
pub fn Dashboard() -> Element {
    let app: App = use_context();

    rsx! {
        div { class: "animate-fade-in-up space-y-10",
            div {
                h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white",
                    "Protocol dashboard"
                }
                p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                    "A live snapshot of your wallet, the token economy, and jury activity on {app.network.read().label()}."
                }
            }

            if app.chain_mismatch() {
                ChainMismatch {}
            }

            if let Some(()) = app.connected().then_some(()) {
                WalletGrid { key: "{app.network.read().key()}" }
            } else {
                Card { class: "",
                    h2 { class: "mb-4 text-lg font-bold text-zinc-900 dark:text-white", "Wallet" }
                    p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                        "Not connected. Use the wallet button in the navbar to see balances and to stake, vote, or submit."
                    }
                }
            }

            BudgetBar { key: "{app.network.read().key()}" }
            JuryPulse { key: "{app.network.read().key()}" }
            ModuleAddresses { key: "{app.network.read().key()}" }
        }
    }
}

#[component]
pub fn ChainMismatch() -> Element {
    let app: App = use_context();
    let wallet_chain = app
        .account
        .read()
        .as_ref()
        .map(|c| c.chain_id.clone())
        .unwrap_or_default();
    rsx! {
        Notice { tone: NoticeTone::Warning, title: "Wallet is on a different network",
            p {
                "The wallet reports chain id {wallet_chain}, but this page reads {app.network.read().label()}. "
                "Switch the network selector in the navbar or reconnect."
            }
        }
    }
}

// ───────────────────────────── read helpers ─────────────────────────────────

fn res_value(res: &Resource<Result<Amount, String>>) -> String {
    match &*res.read() {
        Some(Ok(a)) => format_amount(*a, 10),
        Some(Err(e)) => format!("⚠ {e}"),
        None => "…".to_string(),
    }
}

fn res_show<T: ToString>(res: &Resource<Result<T, String>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => v.to_string(),
        Some(Err(e)) => format!("⚠ {e}"),
        None => "…".to_string(),
    }
}

/// Inline form of a module-resolution error (used on cards that show "…").
pub fn err_of(res: &Resource<Result<Amount, String>>) -> Option<String> {
    match &*res.read() {
        Some(Err(e)) => Some(e.clone()),
        _ => None,
    }
}

// ─────────────────────────── wallet balance grid ────────────────────────────

#[component]
fn WalletGrid() -> Element {
    let app: App = use_context();
    let net = app.network;
    let addr = app.normalized_address().unwrap_or_default();

    let cl_addr = addr.clone();
    let stk = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::token_balance_of(net, &addr).await }
    });
    let cl_addr = addr.clone();
    let gas = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { crate::chain::strk_balance(net, &addr, crate::config::STRK).await }
    });
    let cl_addr = addr.clone();
    let gov_bal = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::gov_wallet_scalar(net, "governance_balance", &addr).await }
    });
    let cl_addr = addr.clone();
    let gov_avail = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::gov_wallet_scalar(net, "governance_available", &addr).await }
    });
    let cl_addr = addr.clone();
    let gov_locked = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::gov_wallet_scalar(net, "governance_locked", &addr).await }
    });
    let cl_addr = addr.clone();
    let juror = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::juror_amount(net, "get_juror_stake", &addr).await }
    });
    let cl_addr = addr.clone();
    let juror_locked = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::juror_amount(net, "get_juror_locked_stake", &addr).await }
    });
    let cl_addr = addr.clone();
    let juror_open = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::juror_open_evaluations(net, &addr).await }
    });

    let wallet_name = app
        .account
        .read()
        .as_ref()
        .map(|c| c.wallet.clone())
        .unwrap_or_default();
    let wallet_addr = app
        .account
        .read()
        .as_ref()
        .map(|c| c.address.clone())
        .unwrap_or_default();

    rsx! {
        Card { class: "lg:col-span-2",
            h2 { class: "mb-4 text-lg font-bold text-zinc-900 dark:text-white", "Balances" }
            div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
                BalanceTile { icon: IconName::Coins, label: "STK (transferable)", res: stk }
                BalanceTile { icon: IconName::Wallet, label: "STRK (gas token)", res: gas }
                BalanceTile { icon: IconName::Gavel, label: "Juror stake", res: juror }
                BalanceTile { icon: IconName::Lock, label: "Juror locked", res: juror_locked }
            }
            div { class: "mt-4 grid gap-4 border-t border-zinc-100 pt-4 sm:grid-cols-3 dark:border-zinc-800",
                BalanceTile { icon: IconName::Vote, label: "Governance balance", res: gov_bal }
                BalanceTile { icon: IconName::Vote, label: "Available", res: gov_avail }
                BalanceTile { icon: IconName::Lock, label: "Locked in convictions", res: gov_locked }
            }
            p { class: "mt-4 text-xs text-zinc-400",
                "Evaluations keeping your stake locked: {res_show(&juror_open)}"
            }
        }
        Card { class: "space-y-3 lg:col-span-1",
            h2 { class: "text-lg font-bold text-zinc-900 dark:text-white", "Account" }
            div { class: "rounded-xl bg-gradient-to-br from-orange-500 to-amber-500 p-4 text-white dark:from-orange-600 dark:to-amber-600",
                div { class: "mb-1 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wide text-white/80",
                    Icon { name: IconName::Wallet, class: "h-3.5 w-3.5" }
                    "STRK balance"
                }
                if let Some(Ok(a)) = gas.read().as_ref() {
                    div { class: "text-2xl font-black tracking-tight",
                        "{format_amount(*a, 10)} {crate::config::GAS_SYMBOL}"
                    }
                } else if let Some(Err(e)) = gas.read().as_ref() {
                    div { class: "text-sm font-semibold text-red-100", "⚠ {e}" }
                } else {
                    div { class: "text-2xl font-black tracking-tight", "…" }
                }
                p { class: "mt-1 text-xs text-white/70",
                    "native gas token on {net.read().label()} · not a governance metric"
                }
            }
            AddressLine { label: "Your address", addr: wallet_addr }
            SendStrkBox { key: "{app.network.read().key()}" }
            p { class: "text-xs text-zinc-400", "Wallet: {wallet_name}" }
            Link { to: crate::Route::Stake {},
                class: "inline-flex items-center gap-1 text-sm font-semibold text-orange-600 dark:text-orange-400",
                "Stake tokens " Icon { name: IconName::ArrowRight, class: "h-4 w-4" }
            }
            Link { to: crate::Route::ProposalsCreate {},
                class: "inline-flex items-center gap-1 text-sm font-semibold text-orange-600 dark:text-orange-400",
                "Submit a funding proposal " Icon { name: IconName::ArrowRight, class: "h-4 w-4" }
            }
        }
    }
}

#[component]
fn BalanceTile(icon: IconName, label: &'static str, res: Resource<Result<Amount, String>>) -> Element {
    rsx! {
        div { class: "rounded-xl bg-zinc-50 p-4 dark:bg-zinc-800",
            div { class: "mb-1 flex items-center gap-1.5 text-xs text-zinc-400",
                Icon { name: icon, class: "h-3.5 w-3.5" }
                "{label}"
            }
            div { class: "text-xl font-black text-zinc-900 dark:text-white", "{res_value(&res)}" }
        }
    }
}

#[component]
pub fn AddressLine(label: &'static str, addr: String) -> Element {
    let net = use_context::<Signal<crate::config::Network>>();
    let mut copied = use_signal(|| false);
    let addr_copy = addr.clone();
    rsx! {
        div { class: "rounded-xl bg-zinc-50 p-3 dark:bg-zinc-800",
            div { class: "text-xs text-zinc-400", "{label}" }
            div { class: "mt-0.5 flex items-center gap-2",
                code { class: "text-sm font-mono font-semibold text-zinc-900 break-all dark:text-white",
                    "{short_address(&addr, 6)}"
                }
                button {
                    class: "rounded-full p-1 text-zinc-400 hover:bg-zinc-100 hover:text-zinc-900 dark:hover:bg-zinc-800 dark:hover:text-white",
                    onclick: move |_| {
                        crate::state::copy_to_clipboard(&addr_copy);
                        copied.set(true);
                        spawn(async move {
                            crate::state::timer_reset(copied).await;
                        });
                    },
                    if *copied.read() { Icon { name: IconName::Check, class: "h-4 w-4 text-emerald-500" } }
                    else { Icon { name: IconName::Copy, class: "h-4 w-4" } }
                }
                a {
                    href: crate::config::explorer_address_url(*net.read(), &addr),
                    target: "_blank",
                    class: "text-xs font-semibold text-orange-600 dark:text-orange-400",
                    "explorer ↗"
                }
            }
        }
    }
}

#[component]
fn SendStrkBox() -> Element {
    let app: App = use_context();
    let net = app.network;
    let mut tx = use_signal(|| TxState::Idle);
    let mut recipient = use_signal(String::new);
    let mut amount = use_signal(String::new);

    let valid = crate::chain::parse_amount(&*amount.read()).unwrap_or(0) > 0
        && crate::chain::parse_felt(&*recipient.read()).is_ok();

    let do_send = move |_| {
        let Some(wei) = crate::chain::parse_amount(&*amount.read()) else { return };
        let Ok(rcp) = crate::chain::parse_felt(&*recipient.read()) else { return };
        let mut parts = vec![rcp];
        parts.extend(contracts::enc_u256(Amount::from_low(wei)));
        let cd = contracts::calldata_json(&parts);
        let contract = crate::config::STRK.to_string();
        let mut recipient = recipient.clone();
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&contract, "transfer", &cd).await {
                Ok(_hash) => {
                    recipient.set(String::new());
                    tx.set(TxState::Sent(String::new(), "STRK transfer signed — check your wallet/explorer.".to_string()))
                }
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    rsx! {
        div { class: "rounded-xl bg-zinc-50 p-3 dark:bg-zinc-800",
            div { class: "mb-2 flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wide text-zinc-400",
                Icon { name: IconName::Send, class: "h-3.5 w-3.5" }
                "Send STRK"
            }
            if app.connected() {
                div { class: "space-y-2",
                    Input {
                        oninput: on_input(move |e: Event<FormData>| recipient.set(e.value())),
                        value: "{recipient.read()}",
                        placeholder: "recipient 0x…".to_string(),
                        r#type: "text".to_string(),
                    }
                    Input {
                        oninput: on_input(move |e: Event<FormData>| amount.set(e.value())),
                        value: "{amount.read()}",
                        placeholder: "amount (STRK)".to_string(),
                        r#type: "text".to_string(),
                    }
                    Button {
                        onclick: click(Box::new(do_send)),
                        variant: BtnVariant::Secondary,
                        size: "sm".to_string(),
                        disabled: Some(tx.read().is_pending() || !valid),
                        class: "w-full".to_string(),
                        Icon { name: IconName::Send, class: "h-4 w-4" }
                        "Send"
                    }
                    TxStatus { state: tx.read().clone() }
                }
            } else {
                p { class: "text-xs text-zinc-400", "Connect a wallet to send STRK." }
            }
        }
    }
}

// ───────────────────────────── monthly budget ───────────────────────────────

#[component]
fn BudgetBar() -> Element {
    let app: App = use_context();
    let net = app.network;
    let month = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_scalar_u64(net, "get_current_month").await }
    });
    let budget = use_resource(move || {
        let net = *net.read();
        let month = month.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or(0);
        async move { contracts::token_month_amount(net, "get_month_budget", month).await }
    });
    let remaining = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_u256(net, "get_remaining_month_budget").await }
    });
    let inflation = use_resource(move || {
        let net = *net.read();
        async move { contracts::token_scalar_u16(net, "get_inflation_rate_bps").await }
    });

    let pct: (f64, bool) = match (budget.read().clone(), remaining.read().clone()) {
        (Some(Ok(b)), Some(Ok(r))) => (ratio_pct(r, b), true),
        _ => (0.0, false),
    };

    rsx! {
        Card { class: "",
            SectionTitle {
                eyebrow: "Token economy".to_string(),
                title: "Monthly emission budget".to_string(),
                description: "A fixed budget releases each month (month #{res_show(&month)}) and shrinks with accepted decay calls; unspent budget never rolls over.".to_string(),
                class: "mb-4".to_string(),
            }
            if !pct.1 && matches!(month.read().as_ref().map(|r| r.is_err()), Some(true)) {
                Notice { tone: NoticeTone::Warning, title: "ConvictionToken not configured",
                    {month.read().as_ref().and_then(|r| r.as_ref().err()).map(|e| rsx! { "{e}" })}
                }
            }
            div { class: "mb-2 flex items-end justify-between",
                span { class: "text-2xl font-black text-zinc-900 dark:text-white",
                    "{res_value(&remaining)} {crate::config::TOKEN_SYMBOL}"
                }
                span { class: "text-sm text-zinc-500 dark:text-zinc-400",
                    "of {res_value(&budget)} remaining"
                }
            }
            div { class: "h-3 w-full overflow-hidden rounded-full bg-zinc-100 dark:bg-zinc-800",
                div { class: "h-full rounded-full bg-gradient-to-r from-orange-500 to-amber-500 transition-all",
                    style: "width: {pct.0}%",
                }
            }
            div { class: "mt-2 text-xs text-zinc-400",
                "{pct.0:.1}% of this month's budget is still available · {res_show(&inflation)} bps monthly inflation"
            }
        }
    }
}

fn ratio_pct(a: Amount, b: Amount) -> f64 {
    let av = amount_f64(a);
    let bv = amount_f64(b);
    if bv <= 0.0 {
        0.0
    } else {
        (av / bv * 100.0).clamp(0.0, 100.0)
    }
}

fn amount_f64(a: Amount) -> f64 {
    let base = 10f64.powi(crate::chain::TOKEN_DECIMALS as i32);
    (a.low as f64) / base + (a.high as f64) * 2f64.powi(128) / base
}

// ───────────────────────────── jury activity ────────────────────────────────

#[component]
fn JuryPulse() -> Element {
    let app: App = use_context();
    let net = app.network;
    let weight = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u256(net, "total_stake_weight").await }
    });
    let min_ids = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u32(net, "get_min_stake_ids").await }
    });
    let min_jurors = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u32(net, "get_min_jurors").await }
    });
    let bond = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u256(net, "get_evaluation_bond_amount").await }
    });

    rsx! {
        div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
            Stat { label: "Stake securing sortition", value: "{res_value(&weight)}", sub: "escrowed STK in the Selection module", icon: IconName::Gavel, accent: Accent::Orange }
            Stat { label: "Min stake IDs / jurors", value: "{res_show(&min_ids)} / {res_show(&min_jurors)}", sub: "required per evaluation", icon: IconName::Dices, accent: Accent::Blue }
            Stat { label: "Evaluation bond", value: "{res_value(&bond)} STK", sub: "escrowed by each evaluation requester", icon: IconName::Shield, accent: Accent::Violet }
            Stat { label: "List_history window", value: "{crate::config::EVENT_LOOKBACK} blocks", sub: "history scanned for event-id lists", icon: IconName::Clock, accent: Accent::Emerald }
        }
    }
}

// ───────────────────────────── module addresses ─────────────────────────────

#[component]
fn ModuleAddresses() -> Element {
    let app: App = use_context();
    rsx! {
        Card { class: "",
            h2 { class: "mb-4 text-lg font-bold text-zinc-900 dark:text-white",
                "Modules on {app.network.read().label()}"
            }
            div { class: "space-y-2",
                AddressRow { module: Module::Token }
                AddressRow { module: Module::Selection }
                AddressRow { module: Module::Governance }
                AddressRow { module: Module::AdminTimelock }
            }
        }
    }
}

#[component]
pub fn AddressRow(module: Module) -> Element {
    let app: App = use_context();
    let net = app.network.read();
    let configured = contracts::is_configured(*net, module);
    rsx! {
        div { class: "flex items-center justify-between gap-3 rounded-xl bg-zinc-50 px-3 py-2.5 dark:bg-zinc-800",
            span { class: "text-sm font-semibold text-zinc-900 dark:text-white", "{module.label()}" }
            if configured {
                code { class: "text-xs font-mono text-zinc-500 dark:text-zinc-300",
                    "{contracts::address_display(*net, module)}"
                }
            } else {
                Badge { tone: BadgeTone::Amber, class: "".to_string(), "not deployed" }
            }
        }
    }
}
