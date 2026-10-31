//! AdminTimelock: parameter tables, protected addresses, roster, and upgrades
//! — everything queued by `propose_set_*` and executed after the 3-day
//! (7-day for upgrades) timelock.

use dioxus::prelude::*;
use starknet_rust::core::types::Felt;

use crate::chain::{format_amount, short_address};
use crate::contracts::{self, Module};
use crate::wallet;
use crate::state::*;
use crate::types::PendingChange;
use crate::ui::*;

type Res<T> = Resource<Result<T, String>>;

// ───────────────────────────── param metadata ───────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// u256 token amount (18-decimals decimal input).
    Token,
    /// u256 raw value (integer input, low limb).
    Raw,
    /// seconds.
    Secs,
    /// plain count.
    Count,
    /// basis points.
    Bps,
    /// u8 small value.
    Band,
}

#[derive(Clone, Copy, PartialEq)]
struct Param {
    label: &'static str,
    module: Module,
    key: &'static str,
    getter: &'static str,
    propose: &'static str,
    execute: &'static str,
    kind: Kind,
}

const fn p(label: &'static str, key: &'static str, getter: &'static str, propose: &'static str, execute: &'static str, kind: Kind) -> Param {
    Param { label, module: Module::Selection, key, getter, propose, execute, kind }
}

impl Param {
    const fn gov(mut self) -> Param {
        self.module = Module::Governance;
        self
    }
    const fn token(mut self) -> Param {
        self.module = Module::Token;
        self
    }
}

const SELECTION_PARAMS: &[Param] = &[
    p("Evaluation bond", "EVAL_BOND_AMT", "get_evaluation_bond_amount", "propose_set_evaluation_bond_amount", "execute_set_evaluation_bond_amount", Kind::Token),
    p("Min stake amount", "STAKE_AMOUNT", "get_stake_amount", "propose_set_stake_amount", "execute_set_stake_amount", Kind::Token),
    p("Juror reward", "JUROR_REWARD", "get_juror_reward_amount", "propose_set_juror_reward_amount", "execute_set_juror_reward_amount", Kind::Token),
    p("Applying window", "APPLY_DURATION", "get_applying_duration", "propose_set_applying_duration", "execute_set_applying_duration", Kind::Secs),
    p("Drawing window", "DRAW_DURATION", "get_drawing_duration", "propose_set_drawing_duration", "execute_set_drawing_duration", Kind::Secs),
    p("Commit window", "COMMIT_DURATION", "get_commit_duration", "propose_set_commit_duration", "execute_set_commit_duration", Kind::Secs),
    p("Reveal window", "REVEAL_DURATION", "get_reveal_duration", "propose_set_reveal_duration", "execute_set_reveal_duration", Kind::Secs),
    p("Slashing window", "SLASH_DURATION", "get_slashing_duration", "propose_set_slashing_duration", "execute_set_slashing_duration", Kind::Secs),
    p("Evaluation cooldown", "EVAL_COOLDOWN", "get_evaluation_cooldown", "propose_set_evaluation_cooldown", "execute_set_evaluation_cooldown", Kind::Secs),
    p("Non-reveal slash", "NON_REVEAL_SLASH_BPS", "get_non_reveal_slash_bps", "propose_set_non_reveal_slash_bps", "execute_set_non_reveal_slash_bps", Kind::Bps),
    p("Incoherence slash", "INCOHERENCE_SLASH_BPS", "get_incoherence_slash_bps", "propose_set_incoherence_slash_bps", "execute_set_incoherence_slash_bps", Kind::Bps),
    p("Coherence band", "COHERENCE_BAND", "get_coherence_band", "propose_set_coherence_band", "execute_set_coherence_band", Kind::Band),
    p("Min stake IDs", "MIN_STAKE_IDS", "get_min_stake_ids", "propose_set_min_stake_ids", "execute_set_min_stake_ids", Kind::Count),
    p("Min jurors", "MIN_JURORS", "get_min_jurors", "propose_set_min_jurors", "execute_set_min_jurors", Kind::Count),
    p("Max evals / stake", "MAX_EVAL_PER_STAKE", "get_max_evaluations_per_stake", "propose_set_max_evaluations_per_stake", "execute_set_max_evaluations_per_stake", Kind::Count),
];

const GOVERNANCE_PARAMS: &[Param] = &[
    p("Reputation stake", "REPUTATION_STAKE", "get_reputation_stake", "propose_set_reputation_stake", "execute_set_reputation_stake", Kind::Token).gov(),
    p("Conviction threshold", "CONVICTION_THRESHOLD", "get_conviction_threshold", "propose_set_conviction_threshold", "execute_set_conviction_threshold", Kind::Token).gov(),
    p("Reward multiplier", "REWARD_MULTIPLIER", "get_reward_multiplier", "propose_set_reward_multiplier", "execute_set_reward_multiplier", Kind::Token).gov(),
    p("Mini reward multiplier", "MINI_REWARD_MULT", "get_mini_reward_multiplier", "propose_set_mini_reward_multiplier", "execute_set_mini_reward_multiplier", Kind::Token).gov(),
    p("Conviction base lock", "CONVICTION_BASE_LOCK", "get_conviction_base_lock", "propose_set_conviction_base_lock", "execute_set_conviction_base_lock", Kind::Secs).gov(),
    p("Max votes / conviction", "MAX_VOTES_CONVICTION", "get_max_votes_per_conviction", "propose_set_max_votes_per_conviction", "execute_set_max_votes_per_conviction", Kind::Count).gov(),
    p("Proposal voting period", "PROPOSAL_VOTE_WINDOW", "get_proposal_voting_period", "propose_set_proposal_voting_period", "execute_set_proposal_voting_period", Kind::Secs).gov(),
];

const TOKEN_PARAMS: &[Param] = &[
    p("Decay rate", "DECAY_RATE_BPS", "get_decay_rate_bps", "propose_set_decay_rate_bps", "execute_set_decay_rate_bps", Kind::Bps).token(),
    p("Inflation rate", "INFLATION_RATE", "get_inflation_rate_bps", "propose_set_inflation_rate_bps", "execute_set_inflation_rate_bps", Kind::Bps).token(),
    p("Min decay for reward", "MIN_DECAY_FOR_REWARD", "get_min_decay_for_reward", "propose_set_min_decay_for_reward", "execute_set_min_decay_for_reward", Kind::Token).token(),
    p("Caller reward amount", "CALLER_REWARD_AMOUNT", "get_caller_reward_amount", "propose_set_caller_reward_amount", "execute_set_caller_reward_amount", Kind::Token).token(),
];

#[component]
pub fn Admin() -> Element {
    let app: App = use_context();
    let net = app.network;

    let admin_count = use_resource(move || {
        let net = *net.read();
        async move { contracts::admin_count(net).await }
    });

    rsx! {
        div { class: "animate-fade-in-up space-y-8",
            div { class: "flex flex-wrap items-end justify-between gap-4",
                div {
                    h1 { class: "text-3xl font-black tracking-tight text-zinc-900 dark:text-white", "AdminTimelock" }
                    p { class: "mt-2 text-zinc-500 dark:text-zinc-400",
                        "Every parameter change, module rewiring, and upgrade is queued for {crate::chain::format_duration(crate::config::TIMELOCK_SECS)} (upgrades: {crate::chain::format_duration(crate::config::UPGRADE_TIMELOCK_SECS)}) before anyone can execute it."
                    }
                }
                Badge { tone: BadgeTone::Zinc, class: String::new(), "{count_str(&admin_count)} admins" }
            }

            ParamGroup { key: "sel", title: "Selection parameters", params: SELECTION_PARAMS }
            ParamGroup { key: "gov-{net.read().key()}", title: "Governance parameters", params: GOVERNANCE_PARAMS }
            ParamGroup { key: "tok-{net.read().key()}", title: "Token parameters", params: TOKEN_PARAMS }
            ProtectedCard { key: "prot-{net.read().key()}" }
            RosterCard { key: "rost-{net.read().key()}" }
            UpgradeCard { key: "upg-{net.read().key()}" }
        }
    }
}

fn count_str(res: &Res<u32>) -> String {
    match &*res.read() {
        Some(Ok(v)) => v.to_string(),
        _ => "—".to_string(),
    }
}

// ─────────────────────────── value formatting/encoding ──────────────────────

fn fmt_felt(kind: Kind, out: &[Felt]) -> String {
    if out.is_empty() {
        return "—".to_string();
    }
    match kind {
        Kind::Token => format_amount(crate::types::Amount::new(
            crate::chain::felt_u128(&out[0]),
            crate::chain::felt_u128(out.get(1).unwrap_or(&Felt::ZERO)),
        ), 10),
        Kind::Raw | Kind::Count => crate::chain::felt_u128(&out[0]).to_string(),
        Kind::Secs => crate::chain::format_duration(crate::chain::felt_u128(&out[0]) as u64),
        Kind::Bps => format!("{} bps", crate::chain::felt_u128(&out[0])),
        Kind::Band => crate::chain::felt_u128(&out[0]).to_string(),
    }
}

fn fmt_pending(kind: Kind, change: &PendingChange) -> String {
    match kind {
        Kind::Token => format_amount(change.new_value, 10),
        Kind::Secs => {
            crate::chain::format_duration(change.new_value.low as u64)
        }
        _ => change.new_value.low.to_string(),
    }
}

fn enc_input(kind: Kind, text: &str) -> Result<Vec<Felt>, String> {
    let t = text.trim();
    match kind {
        Kind::Token => {
            let wei = crate::chain::parse_amount(t).ok_or_else(|| "invalid token amount".to_string())?;
            Ok(contracts::enc_u256(crate::types::Amount::from_low(wei as u128)))
        }
        Kind::Raw => {
            let v: u128 = t.parse().map_err(|_| "invalid number".to_string())?;
            Ok(contracts::enc_u256(crate::types::Amount::from_low(v as u128)))
        }
        Kind::Secs | Kind::Count => {
            let v: u64 = t.parse().map_err(|_| "invalid number".to_string())?;
            Ok(vec![Felt::from(v as u128)])
        }
        Kind::Bps => {
            let v: u16 = t.parse().map_err(|_| "invalid bps".to_string())?;
            Ok(vec![Felt::from(v as u128)])
        }
        Kind::Band => {
            let v: u8 = t.parse().map_err(|_| "0–3".to_string())?;
            Ok(vec![Felt::from(v as u128)])
        }
    }
}

fn kind_hint(kind: Kind) -> &'static str {
    match kind {
        Kind::Token => "STK amount",
        Kind::Raw => "integer",
        Kind::Secs => "seconds",
        Kind::Count => "count",
        Kind::Bps => "bps (1/10000)",
        Kind::Band => "0–3",
    }
}

fn simple_setter(mut sig: Signal<String>) -> Box<dyn FnMut(Event<FormData>)> {
    Box::new(move |e: Event<FormData>| sig.set(e.value()))
}

fn current_txt(param: &Param, res: &Res<Vec<Felt>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => fmt_felt(param.kind, v),
        Some(Err(_)) => "unavailable".to_string(),
        None => "…".to_string(),
    }
}

fn roster_txt(add: bool) -> &'static str {
    if add {
        "Add"
    } else {
        "Remove"
    }
}

// ─────────────────────────────── param rows ────────────────────────────────

#[component]
fn ParamGroup(title: String, params: &'static [Param]) -> Element {
    rsx! {
        Card { class: "space-y-3",
            SectionTitle {
                eyebrow: "Timelocked parameters".to_string(),
                title: title.to_string(),
                description: "Current value comes from the module getter; queued changes unlock after the timelock countdown, then anyone may execute.".to_string(),
                class: "!mb-2".to_string(),
            }
            for p in params.iter() {
                ParamRow { key: "{p.key}", param: p }
            }
        }
    }
}

#[component]
fn ParamRow(param: &'static Param) -> Element {
    let app: App = use_context();
    let net = app.network;

    let current = use_resource(move || {
        let net = *net.read();
        async move { contracts::module_call(net, param.module, param.getter, vec![]).await }
    });
    let pending = use_resource(move || {
        let net = *net.read();
        async move {
            let _ = contracts::addr(net, param.module)?;
            contracts::pending_change(net, param.module, param.key).await
        }
    });
    let mut input = use_signal(String::new);
    let mut tx = use_signal(|| TxState::Idle);

    let do_propose = {
        let net = net.clone();
        Box::new(move |_| {
            let cd = match enc_input(param.kind, &input.read().clone()) {
                Ok(c) => c,
                Err(e) => {
                    tx.set(TxState::Failed(e));
                    return;
                }
            };
            let Ok(a) = contracts::addr(*net.read(), Module::AdminTimelock) else { return };
            let cd = contracts::calldata_json(&cd);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, param.propose, &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), "Queued — the value unlocks after the timelock.".to_string())),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };
    let do_execute = {
        let net = net.clone();
        Box::new(move |_| {
            let Ok(a) = contracts::addr(*net.read(), Module::AdminTimelock) else { return };
            let cd = contracts::calldata_json(&[]);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, param.execute, &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), "Executed — the module parameter is updated.".to_string())),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };

    let pending_view: Option<PendingChange> = match &*pending.read() {
        Some(Ok(p)) => Some(p.clone()),
        _ => None,
    };
    let pending_unlocked = pending_view
        .as_ref()
        .map(|p| p.exists && crate::chain::now_seconds().saturating_sub(p.effective_at) < u64::MAX)
        .unwrap_or(false);

    rsx! {
        div { class: "rounded-xl border border-zinc-200 p-3 dark:border-zinc-800",
            div { class: "flex flex-wrap items-center justify-between gap-2",
                span { class: "text-sm font-bold text-zinc-900 dark:text-white", "{param.label}" }
                code { class: "text-xs font-mono text-zinc-400", "{param.key} → {param.module.label()}" }
            }
            div { class: "mt-2 flex flex-wrap items-center gap-3",
                code { class: "rounded-lg bg-zinc-100 px-2 py-1 text-xs font-semibold text-zinc-900 dark:bg-zinc-800 dark:text-white",
                    "{current_txt(param, &current)}"
                }
                if let Some(p) = pending_view {
                    if p.exists {
                        Badge { tone: BadgeTone::Amber, class: String::new(),
                            "queued {fmt_pending(param.kind, &p)} · {crate::chain::countdown(p.effective_at, crate::chain::now_seconds())}"
                        }
                        if pending_unlocked {
                            Button { onclick: click(do_execute), variant: BtnVariant::Secondary, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Execute" }
                        }
                    }
                }
            }
            div { class: "mt-2 flex flex-wrap items-end gap-2",
                div { class: "min-w-40 flex-1",
                    Input {
                        oninput: on_input(simple_setter(input)),
                        value: "{input.read()}",
                        placeholder: kind_hint(param.kind).to_string(),
                        r#type: "text".to_string(),
                    }
                }
                Button { onclick: click(do_propose), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Propose" }
            }
            div { class: "mt-2", TxStatus { state: tx.read().clone() } }
        }
    }
}

// ───────────────────────────── protected addresses ──────────────────────────

#[component]
fn ProtectedCard() -> Element {
    let nav = use_context::<Signal<crate::config::Network>>();
    let mut tx = use_signal(|| TxState::Idle);
    let mut target = use_signal(String::new);
    let mut protect = use_signal(|| String::from("true"));

    let results = use_resource(move || {
        let net = *nav.read();
        let target = target.read().trim().to_string();
        async move {
            let module = contracts::addr(net, Module::Token)?;
            if target.is_empty() {
                return Ok::<Option<(bool, u64)>, String>(None);
            }
            let addr_felt = crate::chain::parse_felt(&target)?;
            let out = contracts::module_call(net, Module::Token, "is_protected_address", vec![addr_felt]).await?;
            let pending = contracts::pending_protected_at(net, &target).await?;
            Ok(Some((
                out.get(0).map(|f| crate::chain::felt_u128(f) != 0).unwrap_or(false),
                pending,
            )))
        }
    });

    let do_propose = {
        let nav = nav.clone();
        Box::new(move |_| {
            let Ok(addr_f) = crate::chain::parse_felt(&target.read().clone()) else {
                tx.set(TxState::Failed("Enter a valid target address.".to_string()));
                return;
            };
            let flag = matches!(protect.read().as_str(), "true" | "1" | "yes");
            let Ok(a) = contracts::addr(*nav.read(), Module::AdminTimelock) else { return };
            let cd = contracts::calldata_json(&[addr_f, Felt::from(flag as u128)]);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, "propose_set_protected_address", &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), "Protected-state change queued.".to_string())),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };

    rsx! {
        Card { class: "space-y-3",
            SectionTitle {
                eyebrow: "Decay exemptions".to_string(),
                title: "Protected addresses".to_string(),
                description: "Protected accounts skip hourly decay (useful for treasuries). Changes queue per target with the same timelock.".to_string(),
                class: "!mb-2".to_string(),
            }
            div { class: "grid gap-3 sm:grid-cols-2",
                div {
                    Label { hint: "account to protect / unprotect".to_string(), "Target address" }
                    Input { oninput: on_input(simple_setter(target)), value: "{target.read()}", placeholder: "0x…".to_string(), r#type: "text".to_string() }
                }
                div {
                    Label { hint: String::new(), "New state (true / false)" }
                    Input { oninput: on_input(simple_setter(protect)), value: "{protect.read()}", placeholder: "true".to_string(), r#type: "text".to_string() }
                }
            }
            Button { onclick: click(do_propose), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Propose protected state" }
            if let Some(Ok(Some((is_protected, pending_at)))) = results.read().as_ref() {
                p { class: "text-xs text-zinc-400",
                    "currently protected: {is_protected}"
                    if *pending_at > 0 {
                        " · a change unlocks {crate::chain::countdown(*pending_at, crate::chain::now_seconds())}"
                    }
                }
            }
            div { TxStatus { state: tx.read().clone() } }
        }
    }
}

// ───────────────────────────────── roster ──────────────────────────────────

#[component]
fn RosterCard() -> Element {
    let nav = use_context::<Signal<crate::config::Network>>();
    let mut tx = use_signal(|| TxState::Idle);
    let mut addr = use_signal(String::new);

    let pending = use_resource(move || {
        let net = *nav.read();
        async move {
            let t = contracts::addr(net, Module::AdminTimelock)?;
            contracts::pending_admin_change(net, &t).await
        }
    });

    let act = {
        let nav = nav.clone();
        Callback::new(move |(add, propose): (bool, bool)| {
            let Ok(addr_f) = crate::chain::parse_felt(&addr.read().clone()) else {
                tx.set(TxState::Failed("Enter a valid admin address.".to_string()));
                return;
            };
            let Ok(a) = contracts::addr(*nav.read(), Module::AdminTimelock) else { return };
            let entrypoint: &str = match (add, propose) {
                (true, true) => "propose_add_admin",
                (true, false) => "execute_add_admin",
                (false, true) => "propose_remove_admin",
                (false, false) => "execute_remove_admin",
            };
            let cd = contracts::calldata_json(&[addr_f]);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, entrypoint, &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), if propose { "Roster change queued.".to_string() } else { "Roster change executed.".to_string() })),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };

    rsx! {
        Card { class: "space-y-3",
            SectionTitle {
                eyebrow: "Admin roster".to_string(),
                title: "Add / remove admins".to_string(),
                description: "Roster changes are proposed against the timelock itself and executed by re-passing the same address after the countdown.".to_string(),
                class: "!mb-2".to_string(),
            }
            Label { hint: "new admin address".to_string(), "Address" }
            Input { oninput: on_input(simple_setter(addr)), value: "{addr.read()}", placeholder: "0x…".to_string(), r#type: "text".to_string() }
            div { class: "flex flex-wrap gap-2",
                Button { onclick: clone_act(act, true, true), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Propose add" }
                Button { onclick: clone_act(act, true, false), variant: BtnVariant::Secondary, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Execute add" }
                Button { onclick: clone_act(act, false, true), variant: BtnVariant::Danger, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Propose remove" }
                Button { onclick: clone_act(act, false, false), variant: BtnVariant::Ghost, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Execute remove" }
            }
            div { class: "rounded-xl bg-zinc-50 p-3 text-xs text-zinc-500 dark:bg-zinc-800 dark:text-zinc-400",
                match &*pending.read() {
                    Some(Ok(p)) if p.exists => rsx! {
                        "{roster_txt(p.is_add)} queued · unlocks {crate::chain::countdown(p.effective_at, crate::chain::now_seconds())}"
                    },
                    _ => rsx! { "No pending roster change." },
                }
            }
            div { TxStatus { state: tx.read().clone() } }
        }
    }
}

fn clone_act(
    act: Callback<(bool, bool), ()>,
    add: bool,
    propose: bool,
) -> EventHandler<Event<MouseData>> {
    let mut act = act;
    Callback::new(move |_| act.call((add, propose)))
}

// ──────────────────────────────── upgrades ─────────────────────────────────

#[component]
fn UpgradeCard() -> Element {
    let mut tx = use_signal(|| TxState::Idle);
    let mut class_hash = use_signal(String::new);

    rsx! {
        Card { class: "space-y-3",
            SectionTitle {
                eyebrow: "Upgrades".to_string(),
                title: "Class upgrades (7-day timelock)".to_string(),
                description: "Queue a new class hash per module target, execute it after the upgrade timelock, or disable upgrades forever for that target.".to_string(),
                class: "!mb-2".to_string(),
            }
            Label { hint: "applies to the selected module below".to_string(), "New class hash" }
            Input { oninput: on_input(simple_setter(class_hash)), value: "{class_hash.read()}", placeholder: "0x…".to_string(), r#type: "text".to_string() }
            div { class: "grid gap-3 md:grid-cols-2",
                for module in [Module::Token, Module::Selection, Module::Governance, Module::AdminTimelock] {
                    UpgradeRow { key: "{module.label()}", tx, class_hash, module }
                }
            }
        }
    }
}

#[component]
fn UpgradeRow(tx: Signal<TxState>, class_hash: Signal<String>, module: Module) -> Element {
    let nav = use_context::<Signal<crate::config::Network>>();

    let pending = use_resource(move || {
        let net = *nav.read();
        async move {
            let t = contracts::addr(net, module)?;
            contracts::pending_upgrade(net, &t).await
        }
    });
    let disabled = use_resource(move || {
        let net = *nav.read();
        async move {
            let t = contracts::addr(net, module)?;
            contracts::upgrades_disabled(net, &t).await
        }
    });

    let mut tx = tx;
    let mut class_hash = class_hash;

    let propose = {
        let nav = nav.clone();
        Box::new(move |_| {
            let Ok(target) = contracts::addr(*nav.read(), module) else { return };
            let Ok(hash) = crate::chain::parse_felt(&class_hash.read().clone()) else {
                tx.set(TxState::Failed("Enter the new class hash above.".to_string()));
                return;
            };
            let Ok(a) = contracts::addr(*nav.read(), Module::AdminTimelock) else { return };
            let Ok(target_felt) = crate::chain::parse_felt(&target) else {
                tx.set(TxState::Failed("bad target address".to_string()));
                return;
            };
            let cd = contracts::calldata_json(&[target_felt, hash]);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, "propose_upgrade", &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), format!("{} upgrade queued.", module.label()))),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };
    let execute = {
        let nav = nav.clone();
        Box::new(move |_| {
            let Ok(target) = contracts::addr(*nav.read(), module) else { return };
            let Ok(a) = contracts::addr(*nav.read(), Module::AdminTimelock) else { return };
            let Ok(target_felt) = crate::chain::parse_felt(&target) else {
                tx.set(TxState::Failed("bad target address".to_string()));
                return;
            };
            let cd = contracts::calldata_json(&[target_felt]);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, "execute_upgrade", &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), format!("{} upgrade executed.", module.label()))),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };
    let disable_prop = {
        let nav = nav.clone();
        Box::new(move |_| {
            let Ok(target) = contracts::addr(*nav.read(), module) else { return };
            let Ok(a) = contracts::addr(*nav.read(), Module::AdminTimelock) else { return };
            let Ok(target_felt) = crate::chain::parse_felt(&target) else {
                tx.set(TxState::Failed("bad target address".to_string()));
                return;
            };
            let cd = contracts::calldata_json(&[target_felt]);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, "propose_disable_upgrades", &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), format!("{} disable-upgrades queued.", module.label()))),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };
    let disable_exec = {
        let nav = nav.clone();
        Box::new(move |_| {
            let Ok(target) = contracts::addr(*nav.read(), module) else { return };
            let Ok(a) = contracts::addr(*nav.read(), Module::AdminTimelock) else { return };
            let Ok(target_felt) = crate::chain::parse_felt(&target) else {
                tx.set(TxState::Failed("bad target address".to_string()));
                return;
            };
            let cd = contracts::calldata_json(&[target_felt]);
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, "execute_disable_upgrades", &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(String::new(), format!("{} upgrades permanently disabled.", module.label()))),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };

    rsx! {
        div { class: "rounded-xl border border-zinc-200 p-3 dark:border-zinc-800",
            div { class: "flex flex-wrap items-center justify-between gap-2",
                span { class: "text-sm font-bold text-zinc-900 dark:text-white", "{module.label()}" }
                match &*disabled.read() {
                    Some(Ok(true)) => rsx! { Badge { tone: BadgeTone::Red, class: String::new(), "upgrades disabled forever" } },
                    _ => rsx! { span { } },
                }
            }
            match &*pending.read() {
                Some(Ok(p)) if p.exists => rsx! {
                    p { class: "mt-1 text-xs text-amber-600 dark:text-amber-400",
                        "queued class {short_address(&p.class_hash, 6)} · unlocks {crate::chain::countdown(p.effective_at, crate::chain::now_seconds())}"
                    }
                },
                _ => rsx! { p { class: "mt-1 text-xs text-zinc-400", "no pending upgrade" } },
            }
            div { class: "mt-2 flex flex-wrap gap-2",
                Button { onclick: click(propose), variant: BtnVariant::Primary, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Propose upgrade" }
                Button { onclick: click(execute), variant: BtnVariant::Secondary, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Execute upgrade" }
                Button { onclick: click(disable_prop), variant: BtnVariant::Ghost, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Propose disable" }
                Button { onclick: click(disable_exec), variant: BtnVariant::Ghost, size: "sm".to_string(), disabled: Some(tx.read().is_pending()), class: String::new(), "Execute disable" }
            }
        }
    }
}
