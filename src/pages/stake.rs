//! Selection page 1: stake management (create / list / unstake).

use dioxus::prelude::*;

use crate::chain::{format_amount, short_address};
use crate::contracts;
use crate::wallet;
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;

type Res<T> = Resource<Result<T, String>>;

#[component]
pub fn StakePage() -> Element {
    let app: App = use_context();
    let net = app.network;
    let addr = app.normalized_address().unwrap_or_default();

    let cl_addr = addr.clone();
    let balance = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::token_balance_of(net, &addr).await }
    });
    let cl_addr = addr.clone();
    let juror_stake = use_resource(move || {
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
    let juror_unlocked = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::juror_amount(net, "get_juror_unlocked_stake", &addr).await }
    });
    let cl_addr = addr.clone();
    let open_evals = use_resource(move || {
        let net = *net.read();
        let addr = cl_addr.clone();
        async move { contracts::juror_open_evaluations(net, &addr).await }
    });
    let min_stake = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u256(net, "get_stake_amount").await }
    });
    let max_evals = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u32(net, "get_max_evaluations_per_stake").await }
    });

    rsx! {
        div { class: "animate-fade-in-up space-y-8",
            div {
                h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white", "Stake" }
                p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                    "Stakes are STK escrowed by the Selection module. Stake IDs back juror applications, get drawn with a Fenwick tree, and lock while serving an evaluation."
                }
            }

            div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-5",
                Stat { label: "STK balance", value: "{amt(&balance)}", sub: wallet_line(&app), icon: IconName::Coins, accent: Accent::Orange }
                Stat { label: "Juror stake", value: "{amt(&juror_stake)}", sub: "across all stake IDs", icon: IconName::Gavel, accent: Accent::Blue }
                Stat { label: "Locked", value: "{amt(&juror_locked)}", sub: "serving open evaluations", icon: IconName::Lock, accent: Accent::Violet }
                Stat { label: "Unlocked", value: "{amt(&juror_unlocked)}", sub: "unstakeable now", icon: IconName::Unlock, accent: Accent::Emerald }
                Stat { label: "Open evaluations", value: "{num(&open_evals)}", sub: "max {show(&max_evals)} per stake", icon: IconName::PieChart, accent: Accent::Pink }
            }

            if app.connected() {
                div { class: "grid gap-6 lg:grid-cols-2",
                    CreateStakeCard { key: "{net.read().key()}", min_stake: min_stake.clone() }
                    UnstakeCard { key: "{net.read().key()}" }
                }
            } else {
                ConnectRequired {
                    "Connect a wallet — every stake pulls STK from your address into the module's escrow."
                }
            }

            if app.connected() {
                StakeList { key: "{net.read().key()}-{addr}" }
            }
        }
    }
}

fn wallet_line(app: &App) -> String {
    match app.address() {
        Some(a) => format!("at {}", short_address(&a, 4)),
        None => "not connected".to_string(),
    }
}

fn show<T: ToString>(res: &Res<T>) -> String {
    match &*res.read() {
        Some(Ok(v)) => v.to_string(),
        _ => "—".to_string(),
    }
}

fn amt(res: &Res<Amount>) -> String {
    match &*res.read() {
        Some(Ok(a)) => format_amount(*a, 10),
        _ => "—".to_string(),
    }
}

fn num(res: &Res<u32>) -> String {
    match &*res.read() {
        Some(Ok(v)) => v.to_string(),
        _ => "—".to_string(),
    }
}

// ─────────────────────────── create / unstake forms ─────────────────────────

#[component]
fn CreateStakeCard(min_stake: Res<Amount>) -> Element {
    let app: App = use_context();
    let mut tx = use_signal(|| TxState::Idle);
    let mut amount = use_signal(String::new);
    let net = app.network;

    let submit = move |_| {
        let Some(wei) = crate::chain::parse_amount(&*amount.read()) else { return };
        let Ok(a) = contracts::addr(*net.read(), contracts::Module::Selection) else {
            tx.set(TxState::Failed(contracts::addr(*net.read(), contracts::Module::Selection).unwrap_err()));
            return;
        };
        let cd = contracts::calldata_json(&contracts::enc_u256(Amount::from_low(wei as u128)));
        let mut amount = amount.clone();
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, "create_stake", &cd).await {
                Ok(_hash) => {
                    amount.set(String::new());
                    tx.set(TxState::Sent(String::new(), "Stake created — the amount moved to the escrow hub. It gets a new stake ID.".to_string()))
                }
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    let min_txt = amt(&min_stake);

    rsx! {
        Card { class: "space-y-4",
            h2 { class: "flex items-center gap-2 text-lg font-bold text-zinc-900 dark:text-white",
                Icon { name: IconName::ArrowDownToLine, class: "h-5 w-5 text-orange-500" }
                "Create stake"
            }
            p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                "Escrow STK into the module. Staked tokens back juror sortition, lock during evaluation service, and share back rewards; the module pulls from your balance, no approve step needed."
            }
            Label { hint: "minimum {min_txt} STK", "Amount" }
            Input { oninput: on_input(move |e: Event<FormData>| amount.set(e.value())), value: "{amount.read()}", placeholder: "0.0".to_string(), r#type: "text".to_string() }
            Button {
                onclick: click(Box::new(submit)),
                variant: BtnVariant::Primary,
                size: "md".to_string(),
                disabled: Some(tx.read().is_pending() || crate::chain::parse_amount(&*amount.read()).unwrap_or(0) == 0),
                class: "".to_string(),
                "Stake"
            }
            div { class: "mt-2", TxStatus { state: tx.read().clone() } }
        }
    }
}

#[component]
fn UnstakeCard() -> Element {
    let app: App = use_context();
    let mut tx = use_signal(|| TxState::Idle);
    let mut stake_id = use_signal(String::new);
    let mut amount = use_signal(String::new);
    let net = app.network;

    let ids = use_resource(move || {
        let net = *net.read();
        let owner = app.normalized_address().unwrap_or_default();
        async move { contracts::juror_stake_ids(net, &owner).await }
    });

    let submit = move |_| {
        let Some((sid, wei)) = valid_unstake(&stake_id.read().clone(), &amount.read().clone()) else {
            tx.set(TxState::Failed("Enter a numeric stake id and an amount in STK.".to_string()));
            return;
        };
        let Ok(a) = contracts::addr(*net.read(), contracts::Module::Selection) else {
            return;
        };
        let mut cd = contracts::enc_u256(Amount::from_low(sid as u128));
        cd.extend(contracts::enc_u256(Amount::from_low(wei as u128)));
        let cd = contracts::calldata_json(&cd);
        let mut amount = amount.clone();
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, "unstake_stake", &cd).await {
                Ok(_hash) => {
                    amount.set(String::new());
                    tx.set(TxState::Sent(String::new(), "Unstake submitted — the escrow releases once the stake leaves active service.".to_string()))
                }
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    rsx! {
        Card { class: "space-y-4",
            h2 { class: "flex items-center gap-2 text-lg font-bold text-zinc-900 dark:text-white",
                Icon { name: IconName::ArrowUpToLine, class: "h-5 w-5 text-orange-500" }
                "Unstake"
            }
            p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                "Withdraw from a stake ID. Only stakes with zero active evaluations can release; serving stakes must wait for their evaluations to settle."
            }
            Label { hint: "see your stake IDs below", "Stake ID" }
            Input { oninput: on_input(move |e: Event<FormData>| stake_id.set(e.value())), value: "{stake_id.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
            Label { hint: "", "Amount" }
            Input { oninput: on_input(move |e: Event<FormData>| amount.set(e.value())), value: "{amount.read()}", placeholder: "0.0".to_string(), r#type: "text".to_string() }
            Button {
                onclick: click(Box::new(submit)),
                variant: BtnVariant::Outline,
                size: "md".to_string(),
                disabled: Some(tx.read().is_pending()),
                class: "".to_string(),
                "Unstake"
            }
            div { class: "mt-2", TxStatus { state: tx.read().clone() } }
            p { class: "text-xs text-zinc-400",
                "Stake IDs of yours: {ids_line(&ids)}"
            }
        }
    }
}

fn valid_unstake(sid: &str, amount: &str) -> Option<(u64, u128)> {
    let sid = sid.trim().parse::<u64>().ok()?;
    let wei = crate::chain::parse_amount(amount)?;
    Some((sid, wei))
}

fn ids_line(res: &Res<Vec<u64>>) -> String {
    match &*res.read() {
        Some(Ok(ids)) if !ids.is_empty() => {
            let joined: Vec<String> = ids.iter().map(|i| i.to_string()).collect();
            joined.join(", ")
        }
        Some(Ok(_)) => "none yet".to_string(),
        _ => "—".to_string(),
    }
}

// ─────────────────────────────── stake list ─────────────────────────────────

#[component]
fn StakeList() -> Element {
    let app: App = use_context();
    let net = app.network;
    let ids = use_resource(move || {
        let net = *net.read();
        let owner = app.normalized_address().unwrap_or_default();
        async move { contracts::juror_stake_ids(net, &owner).await }
    });

    let loaded = match &*ids.read() {
        Some(Ok(list)) => Some((net.read().key().to_string(), list.clone())),
        _ => None,
    };

    rsx! {
        section { class: "space-y-4",
            SectionTitle {
                eyebrow: "Escrowed stake".to_string(),
                title: "Stake IDs".to_string(),
                description: "Each creation mints a stake ID that can join up to max_evaluations_per_stake evaluations simultaneously.".to_string(),
                class: "".to_string(),
            }
            match loaded {
                Some((nkey, list)) if !list.is_empty() => rsx! {
                    div { class: "grid gap-4 md:grid-cols-2",
                        for sid in list {
                            StakeCard { key: "{nkey}-{sid}", stake_id: sid }
                        }
                    }
                },
                Some((_, _)) => rsx! {
                    EmptyState {
                        title: "No stakes yet".to_string(),
                        description: "Create a stake above to start backing evaluations as a juror.".to_string(),
                        action_icon: Some(IconName::Gavel),
}
                },
                None => rsx! { Skeleton { class: "h-24 w-full rounded-2xl" } },
            }
        }
    }
}

#[component]
fn StakeCard(stake_id: u64) -> Element {
    let app: App = use_context();
    let net = app.network;
    let now = app.now;

    let info = use_resource(move || {
        let net = *net.read();
        async move { contracts::get_stake(net, stake_id).await }
    });
    let eval_count = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u32(net, "get_max_evaluations_per_stake").await }
    });

    rsx! {
        Card { class: "space-y-4",
            div { class: "flex items-start justify-between",
                div {
                    div { class: "text-sm font-bold text-zinc-900 dark:text-white", "Stake #{stake_id}" }
                    match &*info.read() {
                        Some(Ok(s)) => rsx! {
                            div { class: "text-xs text-zinc-400", "created {crate::chain::date_string(s.created_at)}" }
                        },
                        _ => rsx! { div { class: "text-xs text-zinc-400" } },
                    }
                }
                match &*info.read() {
                    Some(Ok(s)) if s.retired => rsx! {
                        Badge { tone: BadgeTone::Zinc, class: "".to_string(), "retired" }
                    },
                    Some(Ok(s)) if s.amount.is_zero() => rsx! {
                        Badge { tone: BadgeTone::Amber, class: "".to_string(), "withdrawn" }
                    },
                    _ => rsx! {
                        Badge { tone: BadgeTone::Green, class: "".to_string(), "active" }
                    },
                }
            }
            div { class: "grid grid-cols-2 gap-3",
                div { class: "rounded-xl bg-zinc-50 p-3 dark:bg-zinc-800",
                    div { class: "text-xs text-zinc-400", "Escrowed amount" }
                    div { class: "text-sm font-semibold text-zinc-900 dark:text-white",
                        "{esc_amount(&info)}"
                    }
                }
                div { class: "rounded-xl bg-zinc-50 p-3 dark:bg-zinc-800",
                    div { class: "text-xs text-zinc-400", "Owner" }
                    div { class: "text-sm font-mono font-semibold text-zinc-900 dark:text-white",
                        "{esc_owner(&info)}"
                    }
                }
            }
            p { class: "text-xs text-zinc-400",
                "Stakes may serve up to {num2(&eval_count)} evaluations at once while their confidence decays and rewards accrue."
            }
        }
    }
}

fn num2(res: &Res<u32>) -> String {
    show(res)
}

fn short(addr: &str) -> String {
    short_address(addr, 6)
}

fn esc_amount(res: &Resource<Result<crate::types::Stake, String>>) -> String {
    match &*res.read() {
        Some(Ok(s)) => format_amount(s.amount, 10),
        _ => "…".to_string(),
    }
}

fn esc_owner(res: &Resource<Result<crate::types::Stake, String>>) -> String {
    match &*res.read() {
        Some(Ok(s)) => short(&s.owner),
        _ => "…".to_string(),
    }
}
