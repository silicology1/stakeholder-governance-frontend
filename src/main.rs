//! Application shell: router, navbar, connect-wallet modal, dark mode, and
//! the shared 1-second wall-clock ticker.

mod chain;
mod config;
mod contracts;
mod pages;
mod state;
mod types;
mod ui;
mod wallet;

use dioxus::prelude::*;

use config::Network;
use pages::admin::Admin;
use pages::conviction_create::ConvictionsCreate;
use pages::convictions::Convictions;
use pages::dashboard::Dashboard;
use pages::evaluation_detail::EvaluationDetail;
use pages::evaluations::{EvaluationCreate, Evaluations};
use pages::governance::Governance;
use pages::home::Home;
use pages::proposal_create::ProposalCreate as ProposalsCreate;
use pages::proposal_detail::ProposalDetail;
use pages::proposals::Proposals;
use pages::stake::StakePage as Stake;
use pages::token::TokenPage;

use ui::*;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Home {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/token")]
    TokenPage {},
    #[route("/stake")]
    Stake {},
    #[route("/proposals")]
    Proposals {},
    #[route("/proposals/create")]
    ProposalsCreate {},
    #[route("/proposals/:id")]
    ProposalDetail { id: u64 },
    #[route("/convictions")]
    Convictions {},
    #[route("/convictions/create")]
    ConvictionsCreate {},
    #[route("/evaluations")]
    Evaluations {},
    #[route("/evaluations/create")]
    EvaluationCreate {},
    #[route("/evaluations/:id")]
    EvaluationDetail { id: u64 },
    #[route("/governance")]
    Governance {},
    #[route("/admin")]
    Admin {},
}

const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

const NAV_LINKS: &[(Route, &str)] = &[
    (Route::Home {}, "Home"),
    (Route::Dashboard {}, "Dashboard"),
    (Route::TokenPage {}, "Token"),
    (Route::Stake {}, "Stake"),
    (Route::Evaluations {}, "Evaluations"),
    (Route::Proposals {}, "Proposals"),
    (Route::Convictions {}, "Convictions"),
    (Route::Governance {}, "Governance"),
    (Route::Admin {}, "Admin"),
];

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // shared state
    let mut network = use_signal(|| Network::Sepolia);
    let account: Signal<Option<wallet::Connected>> = use_signal(|| None);
    let mut now = use_signal(chain::now_seconds);
    let connect_open: Signal<bool> = use_signal(|| false);
    use_context_provider(|| state::App { network, account, now });
    // some components consume the network signal directly as its own context
    use_context_provider(move || network);
    use_context_provider(move || connect_open);

    // dark mode sync (persisted via localStorage; toggled by the navbar)
    use_effect(move || {
        document::eval(
            "if (localStorage.getItem('sg_theme') === 'dark' || (!localStorage.getItem('sg_theme') && window.matchMedia('(prefers-color-scheme: dark)').matches)) { document.documentElement.classList.add('dark'); } else { document.documentElement.classList.remove('dark'); }",
        );
    });

    // restore + persist the network selection
    let mut network_rehydrate = network;
    use_effect(move || {
        spawn(async move {
            let value: String = document::eval("localStorage.getItem('sg_network') || ''")
                .join()
                .await
                .unwrap_or_default();
            if value == "mainnet" {
                network_rehydrate.set(Network::Mainnet);
            }
        });
    });
    use_effect(move || {
        let key = network.read().key().to_string();
        document::eval(&format!("localStorage.setItem('sg_network', '{key}')"));
    });

    // 1-second ticker for countdowns
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1_000).await;
            now.set(chain::now_seconds());
        }
    });

    rsx! {
        Router::<Route> {}
    }
}

/// Layout wrapper for every route: navbar, modal, content outlet, footer.
/// Lives inside the router tree so `[Link]`, `[use_route]` and `[Outlet]`
/// have access to the routing context.
#[component]
fn AppShell() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "min-h-screen bg-amber-50 text-zinc-900 antialiased dark:bg-zinc-950 dark:text-zinc-100",
            Navbar {}
            ConnectModal {}
            main { class: "mx-auto w-full max-w-7xl px-4 pb-24 pt-24 sm:px-6 lg:px-8",
                Outlet::<Route> {}
            }
            Footer {}
        }
    }
}

// ─────────────────────────────── navbar ─────────────────────────────────────

#[component]
fn Navbar() -> Element {
    let route = use_route::<Route>();
    let mut mobile_open = use_signal(|| false);
    let app: state::App = use_context();

    let net = app.network;
    let mut net = net;
    let set_network = Callback::new(move |value: String| {
        net.set(if value == "mainnet" { Network::Mainnet } else { Network::Sepolia });
    });

    rsx! {
        header { class: "fixed inset-x-0 top-0 z-40 border-b border-zinc-200/70 bg-white/80 backdrop-blur dark:border-zinc-800/70 dark:bg-zinc-950/80",
            div { class: "mx-auto flex h-16 max-w-7xl items-center justify-between gap-4 px-4 sm:px-6 lg:px-8",
                Link { to: Route::Home {}, class: "flex items-center gap-2 font-black tracking-tight text-zinc-900 dark:text-white",
                    span { class: "flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-orange-500 to-amber-500 text-white",
                        Icon { name: IconName::Flame, class: "h-5 w-5" }
                    }
                    span { class: "bg-gradient-to-r from-orange-500 to-amber-500 bg-clip-text text-lg text-transparent",
                        "Stakeholder"
                    }
                }

                nav { class: "hidden items-center gap-1 lg:flex",
                    for (to, label) in NAV_LINKS.iter() {
                        NavLink { to: to.clone(), label, active: route == *to }
                    }
                }

                div { class: "flex items-center gap-1.5",
                    select {
                        class: "hidden appearance-none rounded-full border border-zinc-300 bg-white px-3 py-1.5 text-xs font-semibold text-zinc-700 outline-none dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200 sm:block",
                        value: "{net.read().key()}",
                        onchange: move |e: Event<FormData>| set_network.call(e.value()),
                        option { value: "sepolia", "Sepolia" }
                        option { value: "mainnet", "Mainnet" }
                    }
                    button {
                        class: "rounded-full p-2 text-zinc-600 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-white",
                        onclick: move |_| {
                            document::eval(
                                "if (document.documentElement.classList.contains('dark')) { document.documentElement.classList.remove('dark'); localStorage.setItem('sg_theme', 'light'); } else { document.documentElement.classList.add('dark'); localStorage.setItem('sg_theme', 'dark'); }",
                            );
                        },
                        Icon { name: IconName::Moon, class: "h-5 w-5" }
                    }
                    ConnectButton {}
                    button {
                        class: "rounded-full p-2 text-zinc-600 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-300 dark:hover:bg-zinc-800 dark:hover:text-white lg:hidden",
                        onclick: move |_| mobile_open.toggle(),
                        if *mobile_open.read() {
                            Icon { name: IconName::X, class: "h-5 w-5" }
                        } else {
                            Icon { name: IconName::Menu, class: "h-5 w-5" }
                        }
                    }
                }
            }

            if *mobile_open.read() {
                nav { class: "border-t border-zinc-200 px-4 py-3 dark:border-zinc-800 lg:hidden",
                    div { class: "grid gap-1",
                        for (to, label) in NAV_LINKS.iter() {
                            Link {
                                to: to.clone(),
                                class: if route == *to {
                                    "rounded-xl bg-orange-100 px-3 py-2 text-sm font-bold text-orange-700 dark:bg-orange-950 dark:text-orange-300".to_string()
                                } else {
                                    "rounded-xl px-3 py-2 text-sm font-semibold text-zinc-500 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-800".to_string()
                                },
                                "{label}"
                            }
                        }
                    }
                    div { class: "mt-2 sm:hidden",
                        select {
                            class: "w-full appearance-none rounded-full border border-zinc-300 bg-white px-3 py-1.5 text-xs font-semibold text-zinc-700 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200",
                            value: "{net.read().key()}",
                            onchange: move |e: Event<FormData>| set_network.call(e.value()),
                            option { value: "sepolia", "Sepolia" }
                            option { value: "mainnet", "Mainnet" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavLink(to: Route, label: &'static str, active: bool) -> Element {
    rsx! {
        Link {
            to: to,
            class: if active {
                "rounded-full bg-orange-100 px-3 py-1.5 text-sm font-bold text-orange-700 dark:bg-orange-950 dark:text-orange-300"
            } else {
                "rounded-full px-3 py-1.5 text-sm font-semibold text-zinc-500 transition-colors hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-white"
            },
            "{label}"
        }
    }
}

// ───────────────────────── connect wallet pieces ────────────────────────────

#[component]
fn ConnectButton() -> Element {
    let mut app: state::App = use_context();
    let mut open: Signal<bool> = use_context();

    if app.connected() {
        let connected = app.account.read().as_ref().unwrap().clone();
        rsx! {
            Button {
                onclick: click(Box::new(move |_| open.set(true))),
                variant: BtnVariant::Secondary,
                size: "sm".to_string(),
                disabled: Some(false),
                class: String::new(),
                Icon { name: IconName::Wallet, class: "h-4 w-4" }
                "{chain::short_address(&connected.address, 4)}"
            }
        }
    } else {
        rsx! {
            Button {
                onclick: click(Box::new(move |_| open.set(true))),
                variant: BtnVariant::Primary,
                size: "sm".to_string(),
                disabled: Some(false),
                class: String::new(),
                Icon { name: IconName::Wallet, class: "h-4 w-4" }
                "Connect"
            }
        }
    }
}

#[component]
fn ConnectModal() -> Element {
    let mut app: state::App = use_context();
    let mut open: Signal<bool> = use_context();
    let mut wallets = use_signal(Vec::<wallet::WalletOption>::new);
    let mut status = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    // (re)discover wallets each time the dialog opens
    use_effect(move || {
        if *open.read() {
            spawn(async move {
                match wallet::list().await {
                    Ok(list) => wallets.set(list),
                    Err(e) => status.set(Some(e)),
                }
            });
        }
    });

    if !*open.read() {
        return rsx! { span { } };
    }

    let connect_to = {
        move |id: String| {
            spawn(async move {
                busy.set(true);
                match wallet::connect(&id).await {
                    Ok(c) => {
                        app.account.set(Some(c));
                        status.set(None);
                        open.set(false);
                    }
                    Err(e) => status.set(Some(e)),
                }
                busy.set(false);
            });
        }
    };

    let close = Box::new(move |_| open.set(false));

    let disconnect = Box::new(move |_| {
        app.account.set(None);
        open.set(false);
    });

    let connected = app.account.read().as_ref().map(|c| c.clone());

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-zinc-950/60 p-4",
            div { class: "w-full max-w-sm rounded-2xl border border-zinc-200 bg-white p-6 shadow-xl dark:border-zinc-800 dark:bg-zinc-900",
                div { class: "mb-4 flex items-center justify-between",
                    h2 { class: "text-lg font-black text-zinc-900 dark:text-white", "Wallet" }
                    Button { onclick: click(close), variant: BtnVariant::Ghost, size: "sm".to_string(), disabled: Some(false), class: String::new(),
                        Icon { name: IconName::X, class: "h-5 w-5" }
                    }
                }

                if let Some(c) = connected {
                    div { class: "space-y-3",
                        div { class: "rounded-xl bg-zinc-50 p-3 dark:bg-zinc-800",
                            div { class: "text-xs text-zinc-400", "Connected with {c.wallet}" }
                            div { class: "break-all font-mono text-sm font-semibold text-zinc-900 dark:text-white",
                                "{chain::short_address(&c.address, 8)}"
                            }
                            div { class: "mt-1 text-xs",
                                if c.chain_id.eq_ignore_ascii_case(app.network.read().chain_id_hex()) {
                                    span { class: "text-emerald-500", "on {app.network.read().label()}" }
                                } else {
                                    span { class: "text-red-500", "wrong network: {c.chain_id}" }
                                }
                            }
                        }
                        if app.chain_mismatch() {
                            Notice { tone: NoticeTone::Warning, title: "Network mismatch",
                                p { "Switch the network selector in the navbar (or reconnect) so reads and writes target the same chain." }
                            }
                        }
                        Button { onclick: click(disconnect), variant: BtnVariant::Danger, size: "md".to_string(), disabled: Some(false), class: String::new(),
                            Icon { name: IconName::LogOut, class: "h-4 w-4" }
                            "Disconnect"
                        }
                    }
                } else {
                    div { class: "space-y-2",
                        if wallets.read().is_empty() && !*busy.read() {
                            p { class: "text-sm text-zinc-500 dark:text-zinc-400",
                                "Looking for injected Starknet wallets (Ready / Braavos)…"
                            }
                        }
                        for w in wallets.read().iter() {
                            Button {
                                key: "{w.id}",
                                onclick: click({
                                    let id = w.id.clone();
                                    Box::new(move |_| connect_to(id.clone()))
                                }),
                                variant: BtnVariant::Secondary,
                                size: "md".to_string(),
                                disabled: Some(*busy.read()),
                                class: "w-full justify-start".to_string(),
                                Icon { name: IconName::Wallet, class: "h-4 w-4" }
                                "{w.name}"
                            }
                        }
                        if let Some(s) = status.read().as_ref() {
                            Notice { tone: NoticeTone::Error, title: String::new(), "{s}" }
                        }
                        p { class: "text-xs text-zinc-400",
                            "Install Ready (Argent) or Braavos — your wallet signs every transaction; this app only reads chain state."
                        }
                    }
                }
            }
        }
    }
}

// ──────────────────────────────── footer ────────────────────────────────────

#[component]
fn Footer() -> Element {
    let app: state::App = use_context();
    rsx! {
        footer { class: "border-t border-zinc-200 py-8 dark:border-zinc-800",
            div { class: "mx-auto flex max-w-7xl flex-wrap items-center justify-between gap-2 px-4 text-xs text-zinc-400 sm:px-6 lg:px-8",
                span { "Modular stakeholder governance · Cairo contracts + Dioxus" }
                span { "reads: {app.network.read().label()}" }
            }
        }
    }
}
