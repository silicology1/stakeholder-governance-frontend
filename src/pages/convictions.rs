//! Conviction list per connected wallet + inline voting / release.

use dioxus::prelude::*;

use crate::chain::{countdown, format_amount};
use crate::contracts;
use crate::wallet;
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;

#[component]
pub fn Convictions() -> Element {
    let app: App = use_context();
    let net = app.network;

    let ids = use_resource(move || {
        let net = *net.read();
        let owner = app.normalized_address().unwrap_or_default();
        async move { contracts::list_conviction_ids(net, &owner).await }
    });

    let balances = use_resource(move || {
        let net = *net.read();
        let owner = app.normalized_address().unwrap_or_default();
        async move {
            let bal = contracts::gov_wallet_scalar(net, "governance_balance", &owner).await?;
            let av = contracts::gov_wallet_scalar(net, "governance_available", &owner).await?;
            Ok::<(Amount, Amount), String>((bal, av))
        }
    });

    let loaded = match &*ids.read() {
        Some(Ok(list)) => Some((net.read().key().to_string(), list.clone())),
        _ => None,
    };

    rsx! {
        div { class: "animate-fade-in-up space-y-8",
            div { class: "flex flex-wrap items-end justify-between gap-4",
                div {
                    h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white", "Convictions" }
                    p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                        "Lock governance tokens to gain conviction power. Higher levels lock longer and multiply the power your votes carry."
                    }
                }
                Link {
                    to: crate::Route::ConvictionsCreate {},
                    class: "inline-flex items-center gap-2 rounded-full bg-zinc-900 px-5 py-2.5 text-sm font-bold text-white transition-colors hover:bg-orange-600 dark:bg-white dark:text-zinc-900 dark:hover:bg-orange-400",
                    Icon { name: IconName::Plus, class: "h-4 w-4" }
                    "Create conviction"
                }
            }

            if app.connected() {
                Card { class: "",
                    div { class: "grid gap-3 sm:grid-cols-3",
                        Metric { label: "Governance balance", value: pair_val(&balances, 2) }
                        Metric { label: "Available to lock", value: pair_val(&balances, 4) }
                        Metric { label: "Convictions held", value: count_of(&ids) }
                    }
                }
            } else {
                ConnectRequired { "Convictions are personal: connect the wallet that holds your governance tokens." }
            }

            if app.connected() {
                match loaded {
                    Some((nkey, list)) if !list.is_empty() => rsx! {
                        div { class: "grid gap-4 md:grid-cols-2",
                            for id in list {
                                ConvictionCard { key: "{nkey}-{id}", app_key: "", conviction_id: id }
                            }
                        }
                    },
                    Some(_) => rsx! {
                        EmptyState {
                            title: "No convictions yet".to_string(),
                            description: "Create a conviction to start voting on funding proposals with your frozen governance tokens.".to_string(),
                            action_icon: Some(IconName::Vote),
}
                    },
                    None => rsx! { Skeleton { class: "h-40 w-full rounded-2xl" } },
                }
            }
        }
    }
}

fn pair_val(res: &Resource<Result<(Amount, Amount), String>>, idx: usize) -> String {
    match &*res.read() {
        Some(Ok((a, b))) => format_amount(if idx == 2 { *a } else { *b }, 10),
        _ => "…".to_string(),
    }
}

fn count_of(res: &Resource<Result<Vec<u64>, String>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => v.len().to_string(),
        _ => "…".to_string(),
    }
}

#[component]
fn ConvictionCard(app_key: &'static str, conviction_id: u64) -> Element {
    let app: App = use_context();
    let net = app.network;
    let owner = app.normalized_address().unwrap_or_default();
    let _ = app_key;

    let cl_owner = owner.clone();
    let info = use_resource(move || {
        let net = *net.read();
        let owner = cl_owner.clone();
        async move { contracts::get_conviction(net, &owner, conviction_id).await }
    });
    let cl_owner = owner.clone();
    let power = use_resource(move || {
        let net = *net.read();
        let owner = cl_owner.clone();
        async move { contracts::gov_conviction_power(net, &owner, conviction_id).await }
    });

    let mut tx = use_signal(|| TxState::Idle);
    let mut prop_input = use_signal(String::new);
    let mut score_input = use_signal(|| String::from("0"));
    let mut release_tx = use_signal(|| TxState::Idle);

    let do_vote = {
        let net = net.clone();
        Box::new(move |_| {
            let Ok(cid) = conviction_id_u64(conviction_id) else { return };
            let Ok(pid) = prop_input.read().parse::<u64>() else {
                tx.set(TxState::Failed("Enter the proposal id to vote on.".to_string()));
                return;
            };
            let Ok(score) = score_input.read().parse::<i8>() else {
                tx.set(TxState::Failed("Score must be −5…5.".to_string()));
                return;
            };
            let Ok(a) = contracts::addr(*net.read(), contracts::Module::Governance) else { return };
            let mut parts = contracts::enc_u256(Amount::from_low(pid as u128));
            parts.push(starknet_rust::core::types::Felt::from(cid as u128));
            parts.push(contracts::enc_score(score));
            let cd = contracts::calldata_json(&parts);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, "vote_with_conviction", &cd).await {
                    Ok(_h) => {
                        prop_input.set(String::new());
                        tx.set(TxState::Sent(String::new(), "Vote submitted.".to_string()))
                    }
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };

    let do_release = {
        let net = net.clone();
        Box::new(move |_| {
            let Ok(a) = contracts::addr(*net.read(), contracts::Module::Governance) else { return };
            let cd = contracts::calldata_json(&[starknet_rust::core::types::Felt::from(conviction_id as u128)]);
            spawn(async move {
                release_tx.set(TxState::Pending);
                match wallet::invoke(&a, "release_conviction", &cd).await {
                    Ok(_h) => release_tx.set(TxState::Sent(String::new(), "Conviction released — governance tokens are unlocked.".to_string())),
                    Err(e) => release_tx.set(TxState::Failed(e)),
                }
            });
        })
    };

    let pending = tx.read().is_pending();

    rsx! {
        Card { class: "space-y-4",
            div { class: "flex items-start justify-between",
                code { class: "font-mono text-sm font-black text-zinc-900 dark:text-white", "Conviction #{conviction_id}" }
                match &*info.read() {
                    Some(Ok(c)) => rsx! {
                        if c.released {
                            Badge { tone: BadgeTone::Zinc, class: String::new(), "released" }
                        } else {
                            Badge { tone: BadgeTone::Orange, class: String::new(), "level {c.level}" }
                        }
                    },
                    _ => rsx! { span { } },
                }
            }

            match &*info.read() {
                Some(Ok(c)) => rsx! {
                    div { class: "grid grid-cols-2 gap-3",
                        Metric { label: "Locked amount", value: format_amount(c.amount, 10) }
                        Metric { label: "Votes cast", value: "{c.votes_cast}" }
                        Metric { label: "Created", value: crate::chain::date_string(c.created_at) }
                        Metric {
                            label: "Unlock",
                            value: if c.released { "released".to_string() } else {
                                format!("{}", countdown(c.created_at + c.lock_duration, (*app.now.read()) as u64))
                            },
                        }
                    }
                },
                Some(Err(e)) => rsx! { p { class: "text-xs text-red-500", "{e}" } },
                None => rsx! { Skeleton { class: "h-20 w-full" } },
            }

            match &*power.read() {
                Some(Ok(p)) => rsx! {
                    p { class: "text-sm font-semibold text-orange-600 dark:text-orange-400",
                        "current power {format_amount(*p, 10)}"
                    }
                },
                _ => rsx! { span { } },
            }

            if app.connected() {
                div { class: "grid gap-3 sm:grid-cols-3 sm:items-end",
                    div {
                        Label { hint: "funding proposal id".to_string(), "Vote on proposal" }
                        Input { oninput: on_input(num_setter(prop_input)), value: "{prop_input.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                    }
                    div {
                        Label { hint: "−5 … 5".to_string(), "Score" }
                        Input { oninput: on_input(num_setter(score_input)), value: "{score_input.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                    }
                    div { class: "flex gap-2",
                        Button { onclick: click(do_vote), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Vote" }
                        Button { onclick: click(do_release), variant: BtnVariant::Outline, size: "sm".to_string(), disabled: Some(release_tx.read().is_pending()), class: String::new(), "Release" }
                    }
                }
                div { class: "space-y-2",
                    TxStatus { state: tx.read().clone() }
                    TxStatus { state: release_tx.read().clone() }
                }
            }
        }
    }
}

fn conviction_id_u64(id: u64) -> Result<u64, ()> {
    Ok(id)
}

fn num_setter(mut sig: Signal<String>) -> Box<dyn FnMut(Event<FormData>)> {
    Box::new(move |e: Event<FormData>| sig.set(e.value()))
}
