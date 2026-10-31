//! Landing page: module overview and quick links.

use dioxus::prelude::*;

use crate::state::*;
use crate::ui::*;

pub fn feature(
    icon: IconName,
    title: &'static str,
    body: &'static str,
) -> (IconName, &'static str, &'static str) {
    (icon, title, body)
}

#[component]
pub fn Home() -> Element {
    let app: App = use_context();
    let network = *app.network.read();

    rsx! {
        div { class: "fade-in-up space-y-20 animate-fade-in-up",
            // hero
            section { class: "relative overflow-hidden rounded-3xl border border-zinc-200 bg-white px-6 py-16 sm:px-12 dark:border-zinc-800 dark:bg-zinc-900",
                div { class: "pointer-events-none absolute -right-24 -top-24 h-72 w-72 rounded-full bg-gradient-to-br from-orange-400/30 to-amber-300/20 blur-3xl" }
                div { class: "pointer-events-none absolute -bottom-32 -left-16 h-72 w-72 rounded-full bg-gradient-to-br from-amber-300/20 to-orange-400/10 blur-3xl" }

                div { class: "relative max-w-3xl",
                    Badge { tone: BadgeTone::Orange, class: "mb-5".to_string(),
                        Icon { name: IconName::ShieldCheck, class: "h-3.5 w-3.5" }
                        "Stake-weighted public-goods funding on Starknet"
                    }
                    h1 { class: "text-4xl font-black leading-tight tracking-tight text-zinc-900 sm:text-6xl dark:text-white",
                        "Fund public goods "
                        span { class: "bg-gradient-to-r from-orange-500 to-amber-500 bg-clip-text text-transparent",
                            "without the guesswork"
                        }
                    }
                    p { class: "mt-6 max-w-2xl text-lg text-zinc-600 dark:text-zinc-300",
                        "Stakeholder Governance pairs modular juror selection, conviction voting, "
                        "and a timelocked administration so that funding decisions are transparent, "
                        "capture-resistant, and accountable to the people doing the work."
                    }
                    div { class: "mt-8 flex flex-wrap gap-3",
                        Link { to: crate::Route::Dashboard {},
                            class: "inline-flex items-center gap-2 rounded-full bg-zinc-900 px-7 py-3.5 text-sm font-bold text-white transition-colors hover:bg-orange-600 dark:bg-white dark:text-zinc-900 dark:hover:bg-orange-400",
                            "Explore the dashboard "
                            Icon { name: IconName::ArrowRight, class: "h-4 w-4" }
                        }
                        Link { to: crate::Route::Proposals {},
                            class: "inline-flex items-center gap-2 rounded-full border border-zinc-300 px-7 py-3.5 text-sm font-bold text-zinc-700 transition-colors hover:border-orange-400 hover:text-orange-600 dark:border-zinc-700 dark:text-zinc-200 dark:hover:text-orange-400",
                            "View proposals"
                        }
                    }
                    div { class: "mt-10 flex flex-wrap gap-x-8 gap-y-3 text-sm text-zinc-500 dark:text-zinc-400",
                        span {
                            strong { class: "text-zinc-900 dark:text-white", "Network: " }
                            "{network.label()}"
                        }
                        span {
                            strong { class: "text-zinc-900 dark:text-white", "STK max supply" " " }
                            "50M"
                        }
                        span {
                            strong { class: "text-zinc-900 dark:text-white", "10" " " }
                            "conviction levels"
                        }
                    }
                }
            }

            // module grid
            section {
                h2 { class: "mb-2 text-3xl font-black tracking-tight text-zinc-900 dark:text-white",
                    "Four mechanisms, one protocol"
                }
                p { class: "mb-8 max-w-2xl text-zinc-500 dark:text-zinc-400",
                    "Each module is its own Cairo class with its own upgrade lifecycle; the timelock admin orchestrates them all."
                }
                div { class: "grid gap-6 md:grid-cols-2",
                    ModuleFeature {
                        icon: IconName::Coins,
                        title: "ConvictionToken",
                        body: "Transferable STK that decays hourly, mints from a monthly inflation budget, and escrows every stake, bond, and proposal deposit.",
                        to: crate::Route::TokenPage {},
                        cta: "Token & budgets",
                    }
                    ModuleFeature {
                        icon: IconName::Gavel,
                        title: "Selection",
                        body: "Stake-backed jurors are drawn through a Fenwick tree with VRF randomness. Commit-reveal voting hides ballots; coherent jurors earn rewards.",
                        to: crate::Route::Evaluations {},
                        cta: "Evaluations",
                    }
                    ModuleFeature {
                        icon: IconName::Vote,
                        title: "Governance",
                        body: "Non-transferable governance grants accrue conviction over time. Proposals execute against frozen voting power and clear their deposits.",
                        to: crate::Route::Proposals {},
                        cta: "Proposals & convictions",
                    }
                    ModuleFeature {
                        icon: IconName::Landmark,
                        title: "AdminTimelock",
                        body: "Every parameter change, module rewiring, roster update, and upgrade is queued for 3 days (7 for upgrades) before it can execute.",
                        to: crate::Route::Admin {},
                        cta: "Admin queue",
                    }
                }
            }

            // CTA
            section { class: "rounded-3xl bg-gradient-to-br from-orange-500 to-amber-500 px-8 py-14 text-center",
                h2 { class: "text-3xl font-black text-white sm:text-4xl",
                    "Read first. Connect when you're ready."
                }
                p { class: "mx-auto mt-4 max-w-2xl text-white/90",
                    "Every proposal, evaluation, and conviction is public on-chain. Connecting a wallet is only required to stake, vote, or submit."
                }
                div { class: "mt-8 flex flex-wrap justify-center gap-3",
                    Link { to: crate::Route::Dashboard {},
                        class: "inline-flex items-center gap-2 rounded-full bg-white px-8 py-3.5 text-sm font-bold text-orange-600 transition-transform hover:scale-[1.02]",
                        "Go to dashboard "
                        Icon { name: IconName::ArrowRight, class: "h-4 w-4" }
                    }
                }
            }
        }
    }
}

#[component]
fn ModuleFeature(icon: IconName, title: &'static str, body: &'static str, to: crate::Route, cta: &'static str) -> Element {
    rsx! {
        div { class: "flex flex-col rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm transition-colors dark:border-zinc-800 dark:bg-zinc-900",
            div { class: "mb-4 flex h-11 w-11 items-center justify-center rounded-xl bg-gradient-to-br from-orange-500 to-amber-500 text-white",
                Icon { name: icon, class: "h-5 w-5" }
            }
            h3 { class: "text-lg font-bold text-zinc-900 dark:text-white", "{title}" }
            p { class: "mt-2 flex-1 text-sm text-zinc-500 dark:text-zinc-400", "{body}" }
            Link { to: to,
                class: "mt-4 inline-flex items-center gap-1.5 text-sm font-semibold text-orange-600 hover:gap-2.5 hover:underline dark:text-orange-400",
                "{cta} "
                Icon { name: IconName::ArrowRight, class: "h-4 w-4" }
            }
        }
    }
}
