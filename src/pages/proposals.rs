//! Funding proposal list (Governance module).

use dioxus::prelude::*;

use crate::chain::{short_address};
use crate::contracts;
use crate::state::*;
use crate::ui::*;

#[component]
pub fn Proposals() -> Element {
    let app: App = use_context();
    let net = app.network;

    let ids = use_resource(move || {
        let net = *net.read();
        async move { contracts::list_proposal_ids(net).await }
    });

    let loaded = match &*ids.read() {
        Some(Ok(list)) => Some((net.read().key().to_string(), list.clone())),
        _ => None,
    };

    rsx! {
        div { class: "animate-fade-in-up space-y-8",
            div { class: "flex flex-wrap items-end justify-between gap-4",
                div {
                    h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white", "Funding proposals" }
                    p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                        "Each proposal escrows a deposit, collects conviction-weighted votes over the voting period, and funds on execution after clearing its score reward."
                    }
                }
                Link {
                    to: crate::Route::ProposalsCreate {},
                    class: "inline-flex items-center gap-2 rounded-full bg-zinc-900 px-5 py-2.5 text-sm font-bold text-white transition-colors hover:bg-orange-600 dark:bg-white dark:text-zinc-900 dark:hover:bg-orange-400",
                    Icon { name: IconName::Plus, class: "h-4 w-4" }
                    "Create proposal"
                }
            }

            match loaded {
                Some((nkey, list)) if !list.is_empty() => rsx! {
                    div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                        for id in list {
                            ProposalCard { key: "{nkey}-{id}", proposal_id: id }
                        }
                    }
                },
                Some(_) => rsx! {
                    EmptyState {
                        title: "No proposals in the recent window".to_string(),
                        description: "Event-scanned list: proposals from the last few tens of thousands of blocks appear here.".to_string(),
                        action_icon: Some(IconName::Vote),
}
                },
                None => rsx! {
                    div { class: "grid gap-4 md:grid-cols-3", for _ in 0..3 { Skeleton { class: "h-32 rounded-2xl" } } }
                },
            }
        }
    }
}

#[component]
fn ProposalCard(proposal_id: u64) -> Element {
    let app: App = use_context();
    let net = app.network;

    let p = use_resource(move || {
        let net = *net.read();
        async move { contracts::get_proposal(net, proposal_id).await }
    });

    rsx! {
        Link {
            to: crate::Route::ProposalDetail { id: proposal_id },
            class: "flex flex-col rounded-2xl border border-zinc-200 bg-white p-5 shadow-sm transition-colors hover:border-orange-400 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-orange-500",
            match &*p.read() {
                Some(Ok(prop)) => rsx! {
                    div { class: "mb-3 flex items-center justify-between",
                        code { class: "text-xs font-bold text-zinc-900 dark:text-white", "Proposal #{proposal_id}" }
                        if prop.executed {
                            Badge { tone: BadgeTone::Green, class: String::new(), "executed" }
                        } else if prop.final_score_set {
                            Badge { tone: BadgeTone::Blue, class: String::new(), "scored" }
                        } else {
                            Badge { tone: BadgeTone::Amber, class: String::new(), "open" }
                        }
                    }
                    p { class: "line-clamp-3 text-sm text-zinc-700 dark:text-zinc-200",
                        "{evidence_line(&prop.evidence)}"
                    }
                    div { class: "mt-3 grid grid-cols-2 gap-2 text-xs text-zinc-500 dark:text-zinc-400",
                        span { "by {short_address(&prop.author, 4)}" }
                        span { "deposit {crate::chain::compact_amount(prop.deposit_amount)} STK" }
                        span { "power {crate::chain::compact_amount(prop.power_total)}" }
                        span {
                            "funds → {short_address(&prop.funding_wallet, 4)}"
                            " {score_txt(prop.final_score, prop.final_score_set)}"
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    code { class: "text-xs", "Proposal #{proposal_id}" }
                    p { class: "mt-2 text-xs text-red-500", "{e}" }
                },
                None => rsx! { Skeleton { class: "h-20 w-full" } },
            }
        }
    }
}

fn evidence_line(s: &str) -> String {
    let t = s.trim();
    if t.is_empty() { "(no evidence)".to_string() } else { t.chars().take(140).collect::<String>() }
}

fn score_txt(score: i64, set: bool) -> String {
    if set { format!("final {score:+}") } else { String::new() }
}
