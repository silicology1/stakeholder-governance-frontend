//! Proposal detail: read back fields, execute when the voting window has
//! elapsed, link to conviction voting.

use dioxus::prelude::*;

use crate::chain::format_amount;
use crate::contracts;
use crate::wallet;
use crate::state::*;
use crate::ui::*;

#[component]
pub fn ProposalDetail(id: u64) -> Element {
    let app: App = use_context();
    let net = app.network;

    let p = use_resource(move || {
        let net = *net.read();
        async move { contracts::get_proposal(net, id).await }
    });
    let voting = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u64(net, "get_proposal_voting_period").await }
    });
    let total_power = use_resource(move || {
        let net = *net.read();
        async move { contracts::gov_scalar_u256(net, "get_proposal_total_power").await }
    });

    let mut tx = use_signal(|| TxState::Idle);
    let do_execute = {
        let net = net.clone();
        Box::new(move |_| {
            let Ok(a) = contracts::addr(*net.read(), contracts::Module::Governance) else { return };
            let cd = contracts::calldata_json(&contracts::enc_u256(crate::types::Amount::from_low(id as u128)));
            let mut tx = tx.clone();
            spawn(async move {
                tx.set(TxState::Pending);
                match wallet::invoke(&a, "execute_proposal", &cd).await {
                    Ok(_h) => tx.set(TxState::Sent(
                        String::new(),
                        "Execution submitted — funding transfers to the wallet when it lands.".to_string(),
                    )),
                    Err(e) => tx.set(TxState::Failed(e)),
                }
            });
        })
    };

    rsx! {
        div { class: "animate-fade-in-up mx-auto max-w-4xl space-y-8",
            Link {
                to: crate::Route::Proposals {},
                class: "inline-flex items-center gap-1.5 text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-white",
                Icon { name: IconName::ArrowLeft, class: "h-4 w-4" }
                "Back to proposals"
            }

            match &*p.read() {
                Some(Ok(prop)) => {
                    let elapsed = created_elapsed(prop.created_at, votes_val(&voting));
                    let (status, tone) = prop.status_label(elapsed);
                    rsx! {
                        Card { class: "space-y-5",
                            div { class: "flex flex-wrap items-center justify-between gap-3",
                                h1 { class: "font-mono text-2xl font-black text-zinc-900 dark:text-white", "Proposal #{id}" }
                                Badge { tone: match tone {
                                        crate::types::ProposalTone::Green => BadgeTone::Green,
                                        crate::types::ProposalTone::Blue => BadgeTone::Blue,
                                        crate::types::ProposalTone::Amber => BadgeTone::Amber,
                                    }, class: String::new(), "{status}" }
                            }

                            div { class: "grid gap-3 sm:grid-cols-2 lg:grid-cols-4",
                                Metric { label: "Author", value: crate::chain::short_address(&prop.author, 6) }
                                Metric { label: "Funding wallet", value: crate::chain::short_address(&prop.funding_wallet, 6) }
                                Metric { label: "Deposit escrowed", value: format_amount(prop.deposit_amount, 10) }
                                Metric {
                                    label: "Final score",
                                    value: if prop.final_score_set { prop.final_score.to_string() } else { "—".to_string() },
                                }
                            }

                            div {
                                h3 { class: "mb-2 text-sm font-bold text-zinc-900 dark:text-white", "Evidence" }
                                div { class: "rounded-xl bg-zinc-50 p-4 dark:bg-zinc-800",
                                    p { class: "whitespace-pre-wrap text-sm text-zinc-700 dark:text-zinc-200",
                                        "{prop.evidence}"
                                    }
                                }
                            }

                            div { class: "grid gap-3 sm:grid-cols-2",
                                Metric { label: "Created", value: crate::chain::date_string(prop.created_at) }
                                Metric {
                                    label: "Voting window",
                                    value: if prop.executed { "closed".to_string() } else if elapsed { "elapsed — ready to execute".to_string() } else { format!("{} left", crate::chain::countdown(prop.created_at + votes_val(&voting), (*app.now.read()) as u64)) },
                                }
                            }

                            if app.connected() && !prop.executed && elapsed {
                                Button {
                                    onclick: click(do_execute),
                                    variant: BtnVariant::Primary,
                                    size: "md".to_string(),
                                    disabled: Some(tx.read().is_pending()),
                                    class: String::new(),
                                    Icon { name: IconName::Hammer, class: "h-4 w-4" }
                                    "Execute proposal"
                                }
                            } else if !elapsed && !prop.executed {
                                p { class: "text-xs text-zinc-400",
                                    "Execution unlocks once the voting period of {votes_str(&voting)} has fully elapsed."
                                }
                            }
                            div { TxStatus { state: tx.read().clone() } }
                        }

                        Card { class: "space-y-3",
                            h3 { class: "text-sm font-bold text-zinc-900 dark:text-white", "Voting power" }
                            div { class: "grid gap-3 sm:grid-cols-2",
                                Metric { label: "Total power recorded on-chain", value: format_amount(display_amount(&total_power), 10) }
                                Metric { label: "Conviction window", value: votes_str(&voting) }
                            }
                            Link {
                                to: crate::Route::Convictions {},
                                class: "inline-flex items-center gap-1.5 text-sm font-semibold text-orange-600 dark:text-orange-400",
                                "Vote with a conviction " Icon { name: IconName::ArrowRight, class: "h-4 w-4" }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { Notice { tone: NoticeTone::Error, title: "Failed to load proposal", "{e}" } },
                None => rsx! { Skeleton { class: "h-64 w-full rounded-2xl" } },
            }
        }
    }
}

fn display_amount(res: &Resource<Result<crate::types::Amount, String>>) -> crate::types::Amount {
    res.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or_default()
}

fn votes_val(res: &Resource<Result<u64, String>>) -> u64 {
    res.read().as_ref().and_then(|r| r.as_ref().ok()).copied().unwrap_or(0)
}

fn votes_str(res: &Resource<Result<u64, String>>) -> String {
    match &*res.read() {
        Some(Ok(v)) => crate::chain::format_duration(*v),
        _ => "…".to_string(),
    }
}

fn created_elapsed(created_at: u64, period: u64) -> bool {
    if created_at == 0 {
        return false;
    }
    crate::chain::now_seconds() >= created_at.saturating_add(period)
}
