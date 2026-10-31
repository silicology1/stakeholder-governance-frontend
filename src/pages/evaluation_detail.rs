//! Evaluation detail: phase-aware jury actions (apply, draw, commit, reveal,
//! slash, finalize, claim, unstake bond).

use dioxus::prelude::*;
use starknet_rust::core::types::Felt;

use crate::chain::{countdown, format_amount, short_address};
use crate::contracts;
use crate::wallet;
use crate::state::*;
use crate::types::{Amount, Evaluation};
use crate::ui::*;

/// Calldata helpers: every u256 is a (low, high) felt pair.
fn cal_u256(v: u64) -> Vec<Felt> {
    vec![Felt::from(v as u128), Felt::ZERO]
}

fn cal_pair(a: u64, b: u64) -> Vec<Felt> {
    [cal_u256(a), cal_u256(b)].concat()
}

/// Readable phase names for the on-chain `PHASE_*` constants.
#[derive(Clone, Copy, PartialEq)]
pub struct Phase {
    pub name: &'static str,
    pub tone: BadgeTone,
}

pub fn phase_label(phase: u8) -> Phase {
    use crate::types::*;
    match phase {
        PHASE_APPLYING => Phase { name: "Applying", tone: BadgeTone::Orange },
        PHASE_DRAWING => Phase { name: "Drawing jurors", tone: BadgeTone::Blue },
        PHASE_COMMIT => Phase { name: "Commit", tone: BadgeTone::Amber },
        PHASE_REVEAL => Phase { name: "Reveal", tone: BadgeTone::Amber },
        PHASE_FINALIZED => Phase { name: "Finalized", tone: BadgeTone::Green },
        PHASE_DISMISSED => Phase { name: "Dismissed", tone: BadgeTone::Red },
        PHASE_SLASH => Phase { name: "Slash window", tone: BadgeTone::Red },
        _ => Phase { name: "Unknown", tone: BadgeTone::Zinc },
    }
}

impl Phase {
    pub fn render(self) -> Element {
        rsx! {
            Badge { tone: self.tone, class: String::new(), "{self.name}" }
        }
    }
}

#[component]
pub fn EvaluationDetail(id: u64) -> Element {
    let app: App = use_context();
    let net = app.network;

    let ev = use_resource(move || {
        let net = *net.read();
        async move { contracts::get_evaluation(net, id).await }
    });
    let evidence = use_resource(move || {
        let net = *net.read();
        async move { contracts::evidence_of_evaluation(net, id).await }
    });

    rsx! {
        div { class: "animate-fade-in-up mx-auto max-w-6xl space-y-8",
            Link {
                to: crate::Route::Evaluations {},
                class: "inline-flex items-center gap-1.5 text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white",
                Icon { name: IconName::ArrowLeft, class: "h-4 w-4" }
                "Back to evaluations"
            }

            match &*ev.read() {
                Some(Ok(e)) => rsx! {
                    Header { ev: e.clone() }
                    Timeline { ev: e.clone() }
                    EvidenceBlock { key: "{evidence.read().is_some()}", evidence: evidence.clone() }
                    div { class: "grid gap-6 lg:grid-cols-2",
                        PhasePanel { ev: e.clone() }
                        JuryPanel { key: "{app.network.read().key()}-{id}", ev: e.clone() }
                    }
                    DrawnJurors { key: "{app.network.read().key()}-{id}", evaluation_id: id }
                },
                Some(Err(err)) => rsx! {
                    Notice { tone: NoticeTone::Error, title: "Failed to load evaluation", "{err}" }
                },
                None => rsx! { Skeleton { class: "h-64 w-full rounded-2xl" } },
            }
        }
    }
}

// ──────────────────────────────── sections ──────────────────────────────────

#[component]
fn Header(ev: Evaluation) -> Element {
    let app: App = use_context();
    let phase = phase_label(ev.phase);
    let is_requester = app
        .normalized_address()
        .map(|a| a.eq_ignore_ascii_case(&ev.requester))
        .unwrap_or(false);
    rsx! {
        div { class: "flex flex-wrap items-start justify-between gap-4",
            div {
                div { class: "mb-2 flex items-center gap-3",
                    code { class: "font-mono text-2xl font-black text-zinc-900 dark:text-white", "Eval #{ev.id}" }
                    {phase.render()}
                }
                div { class: "flex flex-wrap gap-x-6 gap-y-1 text-sm text-zinc-500 dark:text-zinc-400",
                    span {
                        "Candidate "
                        code { class: "font-mono", "{short_address(&ev.candidate, 6)}" }
                    }
                    span {
                        "Requested by "
                        code { class: "font-mono", "{short_address(&ev.requester, 6)}" }
                    }
                    if is_requester {
                        Badge { tone: BadgeTone::Orange, class: String::new(), "you are the requester" }
                    }
                }
            }
            a {
                href: crate::config::explorer_address_url(*app.network.read(), &ev.candidate),
                target: "_blank",
                class: "text-sm font-semibold text-orange-600 dark:text-orange-400",
                "candidate on explorer ↗"
            }
        }
    }
}

#[component]
fn Timeline(ev: Evaluation) -> Element {
    let app: App = use_context();
    let now = (*app.now.read()) as u64;
    rsx! {
        Card { class: String::new(),
            SectionTitle {
                eyebrow: "Schedule".to_string(),
                title: "Phase deadlines".to_string(),
                description: "Each phase closes on its own clock built from the applying, drawing, commit, reveal, and slashing durations.".to_string(),
                class: "!mb-4".to_string(),
            }
            div { class: "grid gap-2 md:grid-cols-2",
                DeadlineRow { label: "Apply window ends", at: ev.apply_deadline, now }
                DeadlineRow { label: "Drawing ends", at: ev.draw_deadline, now }
                DeadlineRow { label: "Commit window ends", at: ev.commit_deadline, now }
                DeadlineRow { label: "Reveal window ends", at: ev.reveal_deadline, now }
                DeadlineRow { label: "Slash window ends", at: ev.slash_deadline, now }
            }
            div { class: "mt-3 grid gap-2 sm:grid-cols-3",
                Metric { label: "Applied", value: "{ev.applied_count}" }
                Metric { label: "Drawn", value: "{ev.drawn_count}" }
                Metric { label: "Minimum jurors required", value: "{ev.min_jurors}" }
            }
        }
    }
}

#[component]
fn DeadlineRow(label: &'static str, at: u64, now: u64) -> Element {
    rsx! {
        div { class: "flex items-center justify-between rounded-xl bg-zinc-50 px-3 py-2.5 dark:bg-zinc-800",
            span { class: "text-sm text-zinc-500 dark:text-zinc-400", "{label}" }
            span { class: "text-sm font-semibold text-zinc-900 dark:text-white",
                if at == 0 { "—" } else { "{countdown(at, now)}" }
            }
        }
    }
}

#[component]
fn EvidenceBlock(evidence: Resource<Result<String, String>>) -> Element {
    let text = match &*evidence.read() {
        Some(Ok(t)) => t.clone(),
        Some(Err(e)) => format!("(failed to load: {e})"),
        None => String::new(),
    };
    let empty = text.trim().is_empty();
    rsx! {
        Card { class: String::new(),
            SectionTitle {
                eyebrow: "Context".to_string(),
                title: "Evaluation evidence".to_string(),
                description: "The requester's statement for why this candidate deserves a juror score.".to_string(),
                class: "!mb-3".to_string(),
            }
            if empty {
                p { class: "text-sm italic text-zinc-400", "loading…" }
            } else {
                p { class: "whitespace-pre-wrap text-sm text-zinc-700 dark:text-zinc-200", "{text}" }
            }
        }
    }
}

#[component]
fn PhasePanel(ev: Evaluation) -> Element {
    let app: App = use_context();
    rsx! {
        Card { class: "space-y-4",
            SectionTitle {
                eyebrow: "Outcome".to_string(),
                title: "Score & settlement".to_string(),
                description: "A final score mints governance tokens for the candidate; coherent jurors earn rewards; the bond settles with the evaluation.".to_string(),
                class: "!mb-3".to_string(),
            }
            div { class: "grid grid-cols-2 gap-3",
                Metric { label: "Phase", value: phase_label(ev.phase).name.to_string() }
                Metric {
                    label: "Final score",
                    value: if ev.final_score_set { ev.final_score.to_string() } else { "—".to_string() },
                }
                Metric { label: "Slash pool", value: format_amount(ev.slash_pool, 10) }
                Metric { label: "Governance minted", value: format_amount(ev.governance_tokens_minted, 10) }
                Metric { label: "Bond settled", value: if ev.bond_settled { "yes".to_string() } else { "no".to_string() } }
                Metric { label: "Bond amount", value: format_amount(ev.bond_amount, 10) }
            }
            p { class: "text-xs break-all text-zinc-400",
                "Contract: {contracts::addr(*app.network.read(), contracts::Module::Selection).unwrap_or_else(|_| \"not configured\".into())}"
            }
        }
    }
}

#[component]
fn DrawnJurors(evaluation_id: u64) -> Element {
    let app: App = use_context();
    let net = app.network;

    let drawn = use_resource(move || {
        let net = *net.read();
        async move { contracts::drawn_jurors(net, evaluation_id).await }
    });

    let rows: Option<Vec<(u64, String)>> = match &*drawn.read() {
        Some(Ok(l)) => Some(l.clone()),
        _ => None,
    };

    rsx! {
        Card { class: String::new(),
            SectionTitle {
                eyebrow: "Jury".to_string(),
                title: "Drawn jurors".to_string(),
                description: "Stake IDs selected by the VRF-backed sortition for this evaluation (from JurorDrawn events in the recent window).".to_string(),
                class: "!mb-3".to_string(),
            }
            match rows {
                Some(list) if !list.is_empty() => rsx! {
                    div { class: "space-y-2",
                        for (sid, juror) in list {
                            div { key: "{sid}", class: "flex items-center justify-between rounded-xl bg-zinc-50 px-3 py-2 text-sm dark:bg-zinc-800",
                                code { class: "font-mono text-zinc-900 dark:text-white", "Stake #{sid}" }
                                code { class: "font-mono text-xs text-zinc-500 dark:text-zinc-400", "{short_address(&juror, 6)}" }
                            }
                        }
                    }
                },
                Some(_) => rsx! { p { class: "text-sm text-zinc-400", "No jurors drawn in the recent window." } },
                None => rsx! { Skeleton { class: "h-16 w-full" } },
            }
        }
    }
}

// ─────────────────────────── juror actions panel ────────────────────────────

#[component]
fn JuryPanel(ev: Evaluation) -> Element {
    let app: App = use_context();
    let id: u64 = ev.id.parse().unwrap_or(0);
    let net = app.network;
    let owner = app.normalized_address().unwrap_or_default();

    let cl_owner = owner.clone();
    let my_stakes = use_resource(move || {
        let net = *net.read();
        let owner = cl_owner.clone();
        async move { contracts::juror_stake_ids(net, &owner).await }
    });

    let mut apply_stake = use_signal(String::new);
    let mut commit_stake = use_signal(String::new);
    let mut commit_salt = use_signal(String::new);
    let mut commit_score = use_signal(|| String::from("0"));
    let mut slash_stake = use_signal(String::new);
    let mut tx = use_signal(|| TxState::Idle);

    let estake = use_resource(move || {
        let net = *net.read();
        let sid = commit_stake.read().parse::<u64>().unwrap_or(0);
        async move { contracts::is_stake_drawn(net, id, sid).await }
    });

    let send = move |entry: &'static str, parts: Vec<Felt>| {
        let Ok(a) = contracts::addr(*net.read(), contracts::Module::Selection) else {
            tx.set(TxState::Failed("Selection module is not configured for this network.".to_string()));
            return;
        };
        let cd = contracts::calldata_json(&parts);
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, entry, &cd).await {
                Ok(_h) => tx.set(TxState::Sent(
                    String::new(),
                    "Transaction submitted — data refreshes once it lands.".to_string(),
                )),
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    let computed_hash = move || -> Option<String> {
        let sid = commit_stake.read().parse::<u64>().ok()?;
        let score = commit_score.read().parse::<i8>().ok()?;
        let salt = commit_salt.read().trim().to_string();
        if salt.is_empty() {
            return None;
        }
        contracts::poseidon_commit_hash(id, sid, &app.normalized_address()?, score, &salt).ok()
    };

    let do_apply = {
        let mut send = send.clone();
        Box::new(move |_| {
            let Ok(sid) = apply_stake.read().parse::<u64>() else {
                tx.set(TxState::Failed("Pick one of your stake IDs.".to_string()));
                return;
            };
            send("apply_to_evaluation", cal_pair(id, sid));
        })
    };
    let do_remove = {
        let mut send = send.clone();
        Box::new(move |_| {
            let Ok(sid) = apply_stake.read().parse::<u64>() else { return };
            send("remove_from_evaluation", cal_pair(id, sid));
        })
    };
    let do_advance = {
        let mut send = send.clone();
        Box::new(move |_| send("advance_evaluation", cal_u256(id)))
    };
    let do_draw = {
        let mut send = send.clone();
        Box::new(move |_| send("draw_jurors", cal_u256(id)))
    };
    let do_commit = {
        let mut send = send.clone();
        Box::new(move |_| {
            let Ok(sid) = commit_stake.read().parse::<u64>() else {
                tx.set(TxState::Failed("Enter the stake ID that was drawn for you.".to_string()));
                return;
            };
            match computed_hash() {
                Some(hash) => {
                    let hash_felt = crate::chain::parse_felt(&hash).unwrap_or(Felt::ZERO);
                    let mut parts = cal_pair(id, sid);
                    parts.push(hash_felt);
                    send("commit_vote", parts);
                }
                None => tx.set(TxState::Failed(
                    "Enter a valid stake ID, score (−5…5), and salt first.".to_string(),
                )),
            }
        })
    };
    let do_gen_salt = {
        Box::new(move |_| commit_salt.set(new_salt()))
    };
    let do_reveal = {
        let mut send = send.clone();
        Box::new(move |_| {
            let Ok(sid) = commit_stake.read().parse::<u64>() else { return };
            let Ok(score) = commit_score.read().parse::<i8>() else {
                tx.set(TxState::Failed("Score must be an integer from −5 to 5.".to_string()));
                return;
            };
            let Ok(salt) = crate::chain::parse_felt(&commit_salt.read().clone()) else {
                tx.set(TxState::Failed("Salt must be a hex or decimal felt.".to_string()));
                return;
            };
            let mut parts = cal_pair(id, sid);
            parts.push(contracts::enc_score(score));
            parts.push(salt);
            send("reveal_vote", parts);
        })
    };
    let do_slash = {
        let mut send = send.clone();
        Box::new(move |_| {
            let Ok(sid) = slash_stake.read().parse::<u64>() else { return };
            send("slash_non_revealer", cal_pair(id, sid));
        })
    };
    let do_finalize = {
        let mut send = send.clone();
        Box::new(move |_| send("finalize_evaluation", cal_u256(id)))
    };
    let do_claim = {
        let mut send = send.clone();
        Box::new(move |_| {
            let Ok(sid) = commit_stake.read().parse::<u64>() else { return };
            send("claim_juror_reward", cal_pair(id, sid));
        })
    };
    let do_unstake_bond = {
        let mut send = send.clone();
        Callback::new(move |_| send("unstake_bond", cal_u256(id)))
    };

    let is_requester = app
        .normalized_address()
        .map(|a| a.eq_ignore_ascii_case(&ev.requester))
        .unwrap_or(false);
    let pending = tx.read().is_pending();

    rsx! {
        Card { class: "space-y-5",
            SectionTitle {
                eyebrow: "Phase actions".to_string(),
                title: "Your jury actions".to_string(),
                description: "Buttons show for the current phase; the contract enforces drawn-stake and deadline rules.".to_string(),
                class: "!mb-2".to_string(),
            }
            if app.connected() {
                if ev.phase == crate::types::PHASE_APPLYING {
                    div { class: "rounded-xl border border-zinc-200 p-4 space-y-3 dark:border-zinc-800",
                        h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Apply a stake to the jury pool" }
                        Label { hint: "yours: {my_ids_line(&my_stakes)}", "Stake ID" }
                        Input {
                            oninput: on_input(stake_setter(apply_stake)),
                            value: "{apply_stake.read()}",
                            placeholder: "0".to_string(),
                            r#type: "text".to_string(),
                        }
                        div { class: "flex flex-wrap gap-2",
                            Button { onclick: click(do_apply), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Apply stake" }
                            Button { onclick: click(do_remove), variant: BtnVariant::Ghost, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Remove" }
                            Button { onclick: click(do_advance), variant: BtnVariant::Outline, size: "sm".to_string(), disabled: Some(pending), class: String::new(),
                                Icon { name: IconName::Play, class: "h-4 w-4" }
                                "Advance phase"
                            }
                        }
                    }
                }

                if ev.phase == crate::types::PHASE_DRAWING {
                    div { class: "rounded-xl border border-zinc-200 p-4 space-y-3 dark:border-zinc-800",
                        h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Drawing" }
                        p { class: "text-xs text-zinc-400", "Anyone can trigger draws until the jury reaches the minimum size." }
                        Button { onclick: click(do_draw), variant: BtnVariant::Secondary, size: "sm".to_string(), disabled: Some(pending), class: String::new(),
                            Icon { name: IconName::Dices, class: "h-4 w-4" }
                            "Draw jurors"
                        }
                    }
                }

                if ev.phase == crate::types::PHASE_COMMIT {
                    div { class: "rounded-xl border border-zinc-200 p-4 space-y-3 dark:border-zinc-800",
                        h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Commit your vote" }
                        p { class: "text-xs text-zinc-400",
                            "hash = poseidon(stake, score, salt). Reveal the same score + salt during the reveal window; keep them safe."
                        }
                        div { class: "grid gap-3 sm:grid-cols-3",
                            div {
                                Label { hint: String::new(), "Stake ID" }
                                Input { oninput: on_input(stake_setter(commit_stake)), value: "{commit_stake.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                            }
                            div {
                                Label { hint: "−5 … 5".to_string(), "Score" }
                                Input { oninput: on_input(stake_setter(commit_score)), value: "{commit_score.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                            }
                            div {
                                Label { hint: "hex or decimal".to_string(), "Salt" }
                                Input { oninput: on_input(stake_setter(commit_salt)), value: "{commit_salt.read()}", placeholder: "0x…".to_string(), r#type: "text".to_string() }
                            }
                        }
                        match estake.read().clone() {
                            Some(Ok(drawn)) if drawn => rsx! {
                                Badge { tone: BadgeTone::Green, class: "mt-1".to_string(), "this stake is among drawn jurors" }
                            },
                            Some(Ok(_)) => rsx! {
                                Badge { tone: BadgeTone::Amber, class: "mt-1".to_string(), "this stake is not drawn for this evaluation" }
                            },
                            _ => rsx! { span { } },
                        }
                        if let Some(h) = computed_hash() {
                            p { class: "mt-2 break-all font-mono text-xs text-zinc-400", "commit hash {h}" }
                        }
                        div { class: "mt-2 flex flex-wrap items-center gap-2",
                            Button { onclick: click(do_commit), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Commit" }
                            Button { onclick: click(do_gen_salt), variant: BtnVariant::Ghost, size: "sm".to_string(), disabled: Some(false), class: String::new(), "Generate salt" }
                        }
                    }
                }

                if ev.phase == crate::types::PHASE_REVEAL {
                    div { class: "rounded-xl border border-zinc-200 p-4 space-y-3 dark:border-zinc-800",
                        h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Reveal your vote" }
                        p { class: "text-xs text-zinc-400", "The stake ID, score, and salt must match the commit exactly." }
                        div { class: "grid gap-3 sm:grid-cols-3",
                            div {
                                Label { hint: String::new(), "Stake ID" }
                                Input { oninput: on_input(stake_setter(commit_stake)), value: "{commit_stake.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                            }
                            div {
                                Label { hint: "−5 … 5".to_string(), "Score" }
                                Input { oninput: on_input(stake_setter(commit_score)), value: "{commit_score.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                            }
                            div {
                                Label { hint: "must match the commit".to_string(), "Salt" }
                                Input { oninput: on_input(stake_setter(commit_salt)), value: "{commit_salt.read()}", placeholder: "0x…".to_string(), r#type: "text".to_string() }
                            }
                        }
                        Button { onclick: click(do_reveal), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Reveal" }
                    }
                }

                if ev.phase == crate::types::PHASE_SLASH {
                    div { class: "rounded-xl border border-red-200 p-4 space-y-3 dark:border-red-900",
                        h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Slash non-revealers" }
                        Label { hint: "a drawn stake ID".to_string(), "Stake ID" }
                        Input { oninput: on_input(stake_setter(slash_stake)), value: "{slash_stake.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                        Button { onclick: click(do_slash), variant: BtnVariant::Danger, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Slash stake" }
                    }
                    Button { onclick: click(do_finalize), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(pending), class: String::new(),
                        "Finalize evaluation"
                    }
                }

                if ev.phase == crate::types::PHASE_FINALIZED {
                    div { class: "rounded-xl border border-zinc-200 p-4 space-y-3 dark:border-zinc-800",
                        h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Collect rewards" }
                        p { class: "text-xs text-zinc-400", "Enter your drawn stake ID if you revealed coherently." }
                        Label { hint: String::new(), "Stake ID" }
                        Input { oninput: on_input(stake_setter(commit_stake)), value: "{commit_stake.read()}", placeholder: "0".to_string(), r#type: "text".to_string() }
                        Button { onclick: click(do_claim), variant: BtnVariant::Secondary, size: "sm".to_string(), disabled: Some(pending), class: String::new(),
                            "Claim juror reward"
                        }
                        if is_requester {
                            Button { onclick: do_unstake_bond, variant: BtnVariant::Outline, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Unstake my bond" }
                        }
                    }
                }

                if ev.phase == crate::types::PHASE_DISMISSED || ev.bond_settled {
                    div { class: "rounded-xl border border-zinc-200 p-4 space-y-2 dark:border-zinc-800",
                        h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Bond settlement" }
                        if is_requester && !ev.bond_settled {
                            Button { onclick: do_unstake_bond, variant: BtnVariant::Outline, size: "sm".to_string(), disabled: Some(pending), class: String::new(), "Unstake bond" }
                        } else {
                            p { class: "text-xs text-zinc-400",
                                if ev.bond_settled { "The bond has already been settled." } else { "Eval dismissed: the requester can pull the bond back with unstake_bond." }
                            }
                        }
                    }
                }

                div { TxStatus { state: tx.read().clone() } }
            } else {
                ConnectRequired { "Connect a wallet to act on this evaluation." }
            }
        }
    }
}

fn stake_setter(mut sig: Signal<String>) -> Box<dyn FnMut(Event<FormData>)> {
    Box::new(move |e: Event<FormData>| sig.set(e.value()))
}

/// Random ~96-bit salt as a decimal string.
pub fn new_salt() -> String {
    let a = (js_sys::Math::random() * 4_294_967_295.0) as u128;
    let b = (js_sys::Math::random() * 4_294_967_295.0) as u128;
    let c = (js_sys::Math::random() * 4_294_967_295.0) as u128;
    ((a << 64) | (b << 32) | c).to_string()
}

fn my_ids_line(res: &Resource<Result<Vec<u64>, String>>) -> String {
    match &*res.read() {
        Some(Ok(ids)) if !ids.is_empty() => {
            let joined: Vec<String> = ids.iter().map(|i| i.to_string()).collect();
            joined.join(", ")
        }
        Some(Ok(_)) => "none — create a stake first".to_string(),
        _ => "…".to_string(),
    }
}
