//! Governance module economics: thresholds, rewards, locks, and wiring.

use dioxus::prelude::*;

use crate::chain::{format_amount, short_address};
use crate::contracts;
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;

#[component]
pub fn Governance() -> Element {
    let app: App = use_context();
    let net = app.network;

    let threshold = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u256(net, "get_conviction_threshold").await }
    });
    let rep_stake = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u256(net, "get_reputation_stake").await }
    });
    let reward = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u256(net, "get_reward_multiplier").await }
    });
    let mini = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u256(net, "get_mini_reward_multiplier").await }
    });
    let base_lock = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u64(net, "get_conviction_base_lock").await }
    });
    let max_votes = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u64(net, "get_max_votes_per_conviction").await }
    });
    let voting_period = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u64(net, "get_proposal_voting_period").await }
    });
    let admin = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_address_of(net, "get_admin").await }
    });
    let selection = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_address_of(net, "get_selection").await }
    });

    rsx! {
        div { class: "animate-fade-in-up space-y-8",
            div {
                h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white", "Governance economics" }
                p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                    "The Governance module grants non-transferable tokens for coherent jurors, and turns conviction-locked balances into proposal power."
                }
            }

            div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-3",
                Stat { label: "Conviction threshold", value: "{amt(&threshold)}", sub: "power needed for execution-grade votes", icon: IconName::Gauge, accent: Accent::Orange }
                Stat { label: "Proposal deposit", value: "{amt(&rep_stake)} STK", sub: "escrowed per proposal, at risk", icon: IconName::Shield, accent: Accent::Blue }
                Stat { label: "Voting period", value: "{dur(&voting_period)}", sub: "before proposals can execute", icon: IconName::Clock, accent: Accent::Violet }
                Stat { label: "Reward multiplier", value: "{amt(&reward)}", sub: "governance minted per coherent juror", icon: coins(), accent: Accent::Emerald }
                Stat { label: "Mini reward multiplier", value: "{amt(&mini)}", sub: "partial reward for near-coherent votes", icon: IconName::PieChart, accent: Accent::Pink }
                Stat { label: "Base conviction lock", value: "{dur(&base_lock)}", sub: "level 1 lock, scaled by level", icon: IconName::Lock, accent: Accent::Orange }
            }

            Card { class: "",
                SectionTitle {
                    eyebrow: "How power accrues".to_string(),
                    title: "Conviction over time".to_string(),
                    description: "A conviction's power grows with its level and the time locked, up to the caps. Each conviction can cast at most {votes(&max_votes)} votes; releasing early forfeits accrued power.".to_string(),
                    class: "!mb-4".to_string(),
                }
                div { class: "grid gap-3 sm:grid-cols-3",
                    Metric { label: "Level range", value: "1 – 10" }
                    Metric { label: "Max votes / conviction", value: "{votes(&max_votes)}" }
                    Metric { label: "Proposal score range", value: "−5 … +5" }
                }
            }

            Card { class: "",
                SectionTitle {
                    eyebrow: "Module wiring".to_string(),
                    title: "Who controls what".to_string(),
                    description: "Selection mints governance tokens directly inside Governance; the AdminTimelock admin is the only writer of parameters.".to_string(),
                    class: "!mb-4".to_string(),
                }
                div { class: "grid gap-2 md:grid-cols-2",
                    WiredRow { role: "Selection module address", res: selection.clone() }
                    WiredRow { role: "Timelock admin", res: admin.clone() }
                }
                p { class: "mt-3 text-xs text-zinc-400",
                    "Governance contract: {contracts::addr(*net.read(), contracts::Module::Governance).unwrap_or_else(|_| \"not configured\".into())}"
                }
            }
        }
    }
}

fn coins() -> IconName { IconName::Coins }

fn amt(res: &Resource<Result<Amount, String>>) -> String {
    match &*res.read() {
        Some(Ok(a)) => format_amount(*a, 10),
        _ => "…".to_string(),
    }
}

fn dur(res: &Resource<Result<u64, String>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => crate::chain::format_duration(*v),
        _ => "…".to_string(),
    }
}

fn votes(res: &Resource<Result<u64, String>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => v.to_string(),
        _ => "…".to_string(),
    }
}

#[component]
fn WiredRow(role: &'static str, res: Resource<Result<String, String>>) -> Element {
    rsx! {
        div { class: "flex items-center justify-between rounded-xl bg-zinc-50 px-3 py-2.5 dark:bg-zinc-800",
            span { class: "text-sm font-semibold text-zinc-900 dark:text-white", "{role}" }
            match &*res.read() {
                Some(Ok(addr)) => rsx! {
                    code { class: "text-xs font-mono text-zinc-500 dark:text-zinc-300", "{short_address(addr, 6)}" }
                },
                Some(Err(e)) => rsx! { code { class: "text-xs text-amber-500", "{short_error(&e)}" } },
                None => rsx! { Skeleton { class: "h-4 w-32" } },
            }
        }
    }
}

fn short_error(e_arg: &str) -> String {
    let e = e_arg;
    if e.contains("not configured") {
        "not configured".to_string()
    } else {
        e.chars().take(60).collect()
    }
}
