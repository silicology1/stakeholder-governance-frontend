//! Evaluation list + request form (Selection module).

use dioxus::prelude::*;

use crate::chain::{format_amount, short_address};
use crate::contracts;
use crate::wallet;
use crate::pages::evaluation_detail::{phase_label, Phase};
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;

#[component]
pub fn Evaluations() -> Element {
    let app: App = use_context();
    let net = app.network;

    let ids = use_resource(move || {
        let net = *net.read();
        async move { contracts::list_evaluation_ids(net).await }
    });

    let loaded = match &*ids.read() {
        Some(Ok(list)) => Some((net.read().key().to_string(), list.clone())),
        _ => None,
    };

    rsx! {
        div { class: "animate-fade-in-up space-y-8",
            div { class: "flex flex-wrap items-end justify-between gap-4",
                div {
                    h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white", "Evaluations" }
                    p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                        "Candidates request an evaluation by escrowing a bond, stake-backed jurors are drawn, and commit-reveal voting produces a final score for governance minting."
                    }
                }
                Link {
                    to: crate::Route::EvaluationCreate {},
                    class: "inline-flex items-center gap-2 rounded-full bg-zinc-900 px-5 py-2.5 text-sm font-bold text-white transition-colors hover:bg-orange-600 dark:bg-white dark:text-zinc-900 dark:hover:bg-orange-400",
                    Icon { name: IconName::Plus, class: "h-4 w-4" }
                    "Request evaluation"
                }
            }

            Notice {
                tone: NoticeTone::Info,
                title: "Event-driven list".to_string(),
                p { "IDs come from EvaluationRequested events in the last {crate::config::EVENT_LOOKBACK} blocks of {app.network.read().label()}." }
            }

            match loaded {
                Some((nkey, list)) if !list.is_empty() => rsx! {
                    div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                        for id in list {
                            EvalCard { key: "{nkey}-{id}", evaluation_id: id }
                        }
                    }
                },
                Some(_) => rsx! {
                    EmptyState {
                        title: "No evaluations in the recent window".to_string(),
                        description: "Newer requests appear here automatically once events exist in the scanned range.".to_string(),
                        action_icon: Some(IconName::Gavel),
}
                },
                None => rsx! {
                    div { class: "grid gap-4 md:grid-cols-3", for _ in 0..3 { Skeleton { class: "h-36 rounded-2xl" } } }
                },
            }
        }
    }
}

#[component]
fn EvalCard(evaluation_id: u64) -> Element {
    let app: App = use_context();
    let net = app.network;

    let ev = use_resource(move || {
        let net = *net.read();
        async move { contracts::get_evaluation(net, evaluation_id).await }
    });

    let to = crate::Route::EvaluationDetail { id: evaluation_id };
    let (nth, _nkey) = (0usize, ());

    rsx! {
        Link {
            to: to,
            key: "{nth}",
            class: "flex flex-col rounded-2xl border border-zinc-200 bg-white p-5 shadow-sm transition-colors hover:border-orange-400 dark:border-zinc-800 dark:bg-zinc-900 dark:hover:border-orange-500",
            match &*ev.read() {
                Some(Ok(e)) => rsx! {
                    div { class: "mb-3 flex items-center justify-between",
                        code { class: "text-xs font-bold text-zinc-900 dark:text-white", "Eval #{evaluation_id}" }
                        {phase_label(e.phase).render()}
                    }
                    div { class: "text-sm text-zinc-600 dark:text-zinc-300",
                        "Candidate {short_address(&e.candidate, 4)}"
                    }
                    div { class: "mt-1 text-xs text-zinc-400",
                        "requested by {short_address(&e.requester, 4)}"
                    }
                    div { class: "mt-3 grid grid-cols-2 gap-2 text-xs text-zinc-500 dark:text-zinc-400",
                        span { "Bond {format_amount(e.bond_amount, 10)}" }
                        span { "Jurors {e.drawn_count}/{e.min_jurors}" }
                        span { "Applied {e.applied_count}" }
                        span { "{phase_label(e.phase).name}" }
                    }
                },
                Some(Err(e)) => rsx! {
                    code { class: "text-xs", "Eval #{evaluation_id}" }
                    p { class: "mt-2 text-xs text-red-500", "{e}" }
                },
                None => rsx! { Skeleton { class: "h-24 w-full" } },
            }
        }
    }
}

// ─────────────────────────── request evaluation ─────────────────────────────

#[component]
pub fn EvaluationCreate() -> Element {
    let app: App = use_context();
    let net = app.network;
    let mut tx = use_signal(|| TxState::Idle);
    let mut evidence = use_signal(String::new);

    let bond = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u256(net, "get_evaluation_bond_amount").await }
    });
    let balance = use_resource(move || {
        let net = *net.read();
        let addr = app.normalized_address().unwrap_or_default();
        async move { contracts::token_balance_of(net, &addr).await }
    });
    let applying = use_resource(move || {
        let net = *net.read();
        async move { contracts::sel_scalar_u64(net, "get_applying_duration").await }
    });

    let submit = move |_| {
        let text = evidence.read().clone();
        if text.trim().is_empty() {
            tx.set(TxState::Failed("Evidence is required (why this candidate deserves a score).".to_string()));
            return;
        }
        let Ok(a) = contracts::addr(*net.read(), contracts::Module::Selection) else { return; };
        let cd = match contracts::enc_byte_array(&text) {
            Ok(f) => contracts::calldata_json(&f),
            Err(e) => {
                tx.set(TxState::Failed(e));
                return;
            }
        };
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, "request_evaluation", &cd).await {
                Ok(_h) => {
                    evidence.set(String::new());
                    tx.set(TxState::Sent(String::new(), "Evaluation requested — the bond was escrowed; find the new evaluation in the list.".to_string()))
                }
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    rsx! {
        div { class: "animate-fade-in-up mx-auto max-w-2xl space-y-6",
            Link {
                to: crate::Route::Evaluations {},
                class: "inline-flex items-center gap-1.5 text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white",
                Icon { name: IconName::ArrowLeft, class: "h-4 w-4" } "Back to evaluations"
            }
            Card { class: "space-y-4",
                h1 { class: "flex items-center gap-2 text-xl font-black text-zinc-900 dark:text-white",
                    Icon { name: IconName::Gavel, class: "h-5 w-5 text-orange-500" }
                    "Request an evaluation"
                }
                p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                    "Escrows a bond of {amt(&bond)} STK (released back when the evaluation settles or slashed if dismissed for too few jurors). Jurors then have {crate::chain::format_duration(crate_value(&applying))} of applying before drawing begins."
                }
                if app.connected() {
                    div { class: "rounded-xl bg-zinc-50 p-3 text-sm dark:bg-zinc-800",
                        span { class: "text-zinc-500 dark:text-zinc-400", "STK balance: " }
                        strong { class: "text-zinc-900 dark:text-white", "{amt(&balance)}" }
                    }
                    Label { hint: "", "Evidence" }
                    Textarea {
                        oninput: on_input(move |e: Event<FormData>| evidence.set(e.value())),
                        value: "{evidence.read()}",
                        placeholder: "Describe the candidate's public-goods work…".to_string(),
                        rows: 8u32,
                    }
                    p { class: "text-xs text-zinc-400",
                        "{evidence.read().chars().count()} / {crate::config::MAX_EVIDENCE_LEN} characters"
                    }
                    Button {
                        onclick: click(Box::new(submit)),
                        variant: BtnVariant::Primary,
                        size: "md".to_string(),
                        disabled: Some(tx.read().is_pending() || evidence.read().trim().is_empty()),
                        class: "".to_string(),
                        Icon { name: IconName::Send, class: "h-4 w-4" }
                        "Request evaluation"
                    }
                } else {
                    ConnectRequired { "Connect a wallet to request an evaluation." }
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

fn crate_value(res: &Resource<Result<u64, String>>) -> u64 {
    res.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or(0)
}
