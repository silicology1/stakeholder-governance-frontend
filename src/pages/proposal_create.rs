//! Proposal creation form (Governance module).

use dioxus::prelude::*;

use crate::chain::format_amount;
use crate::contracts;
use crate::wallet;
use crate::state::*;
use crate::types::Amount;
use crate::ui::*;

#[component]
pub fn ProposalCreate() -> Element {
    let app: App = use_context();
    let net = app.network;
    let mut tx = use_signal(|| TxState::Idle);
    let mut evidence = use_signal(String::new);
    let mut funding_wallet = use_signal(String::new);

    let deposit = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u256(net, "get_reputation_stake").await }
    });
    let voting = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u64(net, "get_proposal_voting_period").await }
    });
    let balance = use_resource(move || {
        let net = *net.read();
        let addr = app.normalized_address().unwrap_or_default();
        async move { contracts::token_balance_of(net, &addr).await }
    });

    // prefill funding wallet with the connected address
    use_effect(move || {
        if funding_wallet.read().is_empty() {
            if let Some(a) = app.normalized_address() {
                funding_wallet.set(a);
            }
        }
    });

    let submit = move |_| {
        let text = evidence.read().clone();
        let wallet = funding_wallet.read().clone();
        if text.trim().is_empty() {
            tx.set(TxState::Failed("Evidence is required — describe what will be funded.".to_string()));
            return;
        }
        if text.len() > crate::config::MAX_EVIDENCE_LEN {
            tx.set(TxState::Failed(format!("Evidence is limited to {} bytes.", crate::config::MAX_EVIDENCE_LEN)));
            return;
        }
        let Ok(a) = contracts::addr(*net.read(), contracts::Module::Governance) else { return; };
        let Ok(felt_wallet) = crate::chain::parse_felt(&wallet) else {
            tx.set(TxState::Failed("Funding wallet must be a valid 0x address.".to_string()));
            return;
        };
        let cd = match contracts::enc_byte_array(&text) {
            Ok(mut f) => {
                let mut out = vec![felt_wallet];
                out.append(&mut f);
                contracts::calldata_json(&out)
            }
            Err(e) => {
                tx.set(TxState::Failed(e));
                return;
            }
        };
        let mut evidence = evidence.clone();
        spawn(async move {
            tx.set(TxState::Pending);
            match wallet::invoke(&a, "create_proposal", &cd).await {
                Ok(_h) => {
                    evidence.set(String::new());
                    tx.set(TxState::Sent(
                        String::new(),
                        "Proposal created — the deposit is escrowed and risked until execution clears it.".to_string(),
                    ))
                }
                Err(e) => tx.set(TxState::Failed(e)),
            }
        });
    };

    rsx! {
        div { class: "animate-fade-in-up mx-auto max-w-2xl space-y-6",
            Link {
                to: crate::Route::Proposals {},
                class: "inline-flex items-center gap-1.5 text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white",
                Icon { name: IconName::ArrowLeft, class: "h-4 w-4" }
                "Back to proposals"
            }
            Card { class: "space-y-4",
                h1 { class: "flex items-center gap-2 text-xl font-black text-zinc-900 dark:text-white",
                    Icon { name: IconName::FileText, class: "h-5 w-5 text-orange-500" }
                    "Create proposal"
                }
                p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                    "Escrow a deposit of {amt(&deposit)} STK, publish evidence, and let conviction-weighted votes decide over {secs(&voting)}. Your funding wallet receives the payout on execution."
                }
                if app.connected() {
                    div { class: "rounded-xl bg-zinc-50 p-3 text-sm dark:bg-zinc-800",
                        span { class: "text-zinc-500 dark:text-zinc-400", "STK balance: " }
                        strong { class: "text-zinc-900 dark:text-white", "{amt(&balance)}" }
                    }
                    Label { hint: "receives the funding on execution", "Funding wallet" }
                    Input {
                        oninput: on_input(text_setter(funding_wallet)),
                        value: "{funding_wallet.read()}",
                        placeholder: "0x…".to_string(),
                        r#type: "text".to_string(),
                    }
                    Label { hint: "plain text, ≤ {crate::config::MAX_EVIDENCE_LEN} bytes", "Evidence" }
                    Textarea {
                        oninput: on_input(text_setter(evidence)),
                        value: "{evidence.read()}",
                        placeholder: "Describe the project, the work, and the requested amount…".to_string(),
                        rows: 9u32,
                    }
                    p { class: "text-xs text-zinc-400",
                        "{evidence.read().chars().count()} / {crate::config::MAX_EVIDENCE_LEN} characters"
                    }
                    Button {
                        onclick: click(Box::new(submit)),
                        variant: BtnVariant::Primary,
                        size: "md".to_string(),
                        disabled: Some(tx.read().is_pending() || evidence.read().trim().is_empty()),
                        class: String::new(),
                        Icon { name: IconName::Send, class: "h-4 w-4" }
                        "Submit proposal"
                    }
                } else {
                    ConnectRequired { "Connect a wallet to submit a proposal." }
                }
                div { TxStatus { state: tx.read().clone() } }
            }
        }
    }
}

fn text_setter(mut sig: Signal<String>) -> Box<dyn FnMut(Event<FormData>)> {
    Box::new(move |e: Event<FormData>| sig.set(e.value()))
}

fn amt(res: &Resource<Result<Amount, String>>) -> String {
    match &*res.read() {
        Some(Ok(a)) => format_amount(*a, 10),
        _ => "…".to_string(),
    }
}

fn secs(res: &Resource<Result<u64, String>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => crate::chain::format_duration(*v),
        _ => "…".to_string(),
    }
}
