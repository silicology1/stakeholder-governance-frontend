//! Design system: cards, buttons, badges, inputs, notices, stats, icons.
//! Tailwind classes mirror the reference app (amber/orange + zinc, dark mode).

use dioxus::prelude::*;

use crate::config::Network;

// ─────────────────────────────── Icons (lucide) ─────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IconName {
    ArrowRight,
    ArrowLeft,
    ArrowUpToLine,
    ArrowDownToLine,
    Coins,
    Vote,
    Gavel,
    Landmark,
    ShieldCheck,
    Shield,
    Clock,
    Calendar,
    Menu,
    X,
    Sun,
    Moon,
    Wallet,
    Plus,
    Check,
    Copy,
    LogOut,
    Lock,
    Unlock,
    Send,
    Gauge,
    Percent,
    Flame,
    PieChart,
    Waves,
    Key,
    Play,
    Undo,
    Upload,
    Hammer,
    Alert,
    Dices,
    FileText,
    ChevronDown,
    Cpu,
}

#[derive(Clone, Default, Debug)]
struct Ic {
    d1: &'static str,
    d2: &'static str,
    d3: &'static str,
    circle: Option<(f64, f64, f64)>,
}

fn icon_data(name: IconName) -> Ic {
    // Lucide icon paths (24×24, stroke-based).
    use IconName::*;
    match name {
        ArrowRight => Ic { d1: "M5 12h14", d2: "m12 5 7 7-7 7", ..Default::default() },
        ArrowLeft => Ic { d1: "M19 12H5", d2: "m12 19-7-7 7-7", ..Default::default() },
        ArrowUpToLine => Ic { d1: "M18 20V10", d2: "m12 10 6 6", d3: "m12 10 6-6", ..Default::default() },
        ArrowDownToLine => Ic { d1: "M18 4v12", d2: "m12 14 6-6", d3: "m12 14 6 6", ..Default::default() },
        Coins => Ic { circle: Some((8.0, 8.0, 6.0)), d1: "M18.09 10.37a6 6 0 1 0-7.1 8.36", d2: "M19 5H5v10", ..Default::default() },
        Vote => Ic { d1: "m9 12 2 2 4-4", d2: "M5 7c1.5 1.5 3 3.5 3 5.5A6.5 6.5 0 0 1 8.5 19c1.5.5 4 1.2 5.5 1.2s4-.7 5.5-1.2a6.5 6.5 0 0 1 0-6.49C19.5 11.5 21 9.5 22.5 8L12 3l-7 4z", ..Default::default() },
        Gavel => Ic { d1: "m14 13-7.5 7.5c-.83.83-2.17.83-3 0a2.1 2.1 0 0 1 0-3l7.5-7.5", d2: "m16 16 6-6", d3: "m8 8 6-6 M9 7 8 8 4 16l10-4 2-2z", ..Default::default() },
        Landmark => Ic { d1: "M18 20v-9 m-6 9v-9 m-6 9v-9", d2: "m12 2 9 5v3H3V7z", ..Default::default() },
        ShieldCheck => Ic { d1: "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z", d2: "m9 12 2 2 4-4", ..Default::default() },
        Shield => Ic { d1: "M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z", ..Default::default() },
        Clock => Ic { circle: Some((12.0, 12.0, 10.0)), d1: "M12 6v6l4 2", ..Default::default() },
        Calendar => Ic { d1: "M8 2v4 m8-4v4 M3 10h18", d2: "M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z", ..Default::default() },
        Menu => Ic { d1: "M4 6h16 M4 12h16 M4 18h16", ..Default::default() },
        X => Ic { d1: "M18 6 6 18", d2: "m6 6 12 12", ..Default::default() },
        Sun => Ic { circle: Some((12.0, 12.0, 4.0)), d1: "M12 2v2 M12 20v2 M2 12h2 M20 12h2 M4.93 4.93l1.41 1.41 M17.66 17.66l1.41 1.41 M2 20l1.41-1.41 M20 4l-1.41 1.41", ..Default::default() },
        Moon => Ic { d1: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9z", ..Default::default() },
        Wallet => Ic { d1: "M19 7V4a1 1 0 0 0-1-1H5a2 2 0 0 0 0 4h15a1 1 0 0 1 1 1v4h-3a2 2 0 0 0 0 4h3a1 1 0 0 0 1-1v-2", d2: "M3 5v14a2 2 0 0 0 2 2h15a1 1 0 0 0 1-1v-4", ..Default::default() },
        Plus => Ic { d1: "M5 12h14", d2: "M12 5v14", ..Default::default() },
        Check => Ic { d1: "M20 6 9 17l-5-5", ..Default::default() },
        Copy => Ic { d1: "M8 2h12a2 2 0 0 1 2 2v12 M4 8h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V10a2 2 0 0 1 2-2z", ..Default::default() },
        LogOut => Ic { d1: "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4 m7 14 5-5-5-5 M12 19H5 M12 5H6 M16 12H3", ..Default::default() },
        Lock => Ic { d1: "M5 11h14a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2z", d2: "M8 11V7a4 4 0 0 1 8 0v4", ..Default::default() },
        Unlock => Ic { d1: "M5 11h14a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2z", d2: "M16 11V7a4 4 0 0 0-7.92-.8", ..Default::default() },
        Send => Ic { d1: "m22 2-7 20-4-9-9-4z", d2: "M22 2 11 13", ..Default::default() },
        Gauge => Ic { d1: "m12 14 4-4", d2: "M3.34 19a10 10 0 1 1 17.32 0", ..Default::default() },
        Percent => Ic { d1: "M19 5 5 19", d2: "M6.5 9a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z M17.5 20a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5z", ..Default::default() },
        Flame => Ic { d1: "M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z", ..Default::default() },
        PieChart => Ic { d1: "M21 12A9 9 0 1 1 12 3v9z", d2: "M21 3v6a9 9 0 0 0-9-6z", ..Default::default() },
        Waves => Ic { d1: "M2 6c.6.5 1.2 1 2.5 1C7 7 7 5 9.5 5c2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1 M2 12c.6.5 1.2 1 2.5 1 2.5 0 2.5-2 5-2 2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1 M2 18c.6.5 1.2 1 2.5 1 2.5 0 2.5-2 5-2 2.6 0 2.4 2 5 2 2.5 0 2.5-2 5-2 1.3 0 1.9.5 2.5 1", ..Default::default() },
        Key => Ic { circle: Some((7.5, 15.5, 5.5)), d1: "m21 2-9.6 9.6 m-4.4 2.6a5.5 5.5 0 1 1-5.5 5.5", d2: "M15.5 7.5l3 3 L22 7l-3-3", ..Default::default() },
        Play => Ic { d1: "M6 4l14 8-14 8z", ..Default::default() },
        Undo => Ic { d1: "M3 7v6h6", d2: "M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13", ..Default::default() },
        Upload => Ic { d1: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4 M17 8l-5-5-5 5 M12 3v12", ..Default::default() },
        Hammer => Ic { d1: "m15 12-8.373 6.843a21.245 21.245 0 0 0 0 0", d2: "M18 10a2 2 0 0 1 2 2v10 M4 4h6l1 3 6 6-3 3-6-6-3-1z", ..Default::default() },
        Alert => Ic { d1: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3z M12 9v4 M12 17h.01", ..Default::default() },
        Dices => Ic { d1: "M6 2.5c1.5 0 3 .09 4.41.25 M21 19.5V6l-9 4 M3 16l7-3", d2: "M6 13.5V3.75a1.25 1.25 0 0 1 2.5.75L11 2.08 21 6v13a1 1 0 0 1-1 1H6z", ..Default::default() },
        FileText => Ic { d1: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7z M14 2v5h5 M16 13H8 M16 17H8 M10 9H8", ..Default::default() },
        ChevronDown => Ic { d1: "m6 9 6 6 6-6", ..Default::default() },
        Cpu => Ic { d1: "M4 4h6v6H4z M14 4h6v6h-6z M4 14h6v6H4z M14 14h6v6h-6z", ..Default::default() },
    }
}

#[component]
pub fn Icon(name: IconName, class: String) -> Element {
    let ic = icon_data(name);
    let circle = ic.circle.map(|(cx, cy, r)| {
        rsx! { circle { cx: "{cx}", cy: "{cy}", r: "{r}" } }
    });
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            class,
            {circle}
            path { d: "{ic.d1}" }
            path { d: "{ic.d2}" }
            path { d: "{ic.d3}" }
        }
    }
}

// ─────────────────────────────── helpers ────────────────────────────────────

/// Join class strings, skipping empty ones.
pub fn cx(classes: &[&str]) -> String {
    classes
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| c.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

// ─────────────────────────────── components ────────────────────────────────

#[component]
pub fn Card(class: String, children: Element) -> Element {
    rsx! {
        div { class: cx(&[
            "rounded-2xl border border-zinc-200 bg-white p-6 shadow-sm transition-colors dark:border-zinc-800 dark:bg-zinc-900",
            &class,
        ]),
            {children}
        }
    }
}

#[component]
pub fn SectionTitle(eyebrow: String, title: String, description: String, class: String) -> Element {
    rsx! {
        div { class: cx(&["mb-6", &class]),
            if !eyebrow.is_empty() {
                p { class: "mb-2 font-mono text-xs uppercase tracking-widest text-orange-500 dark:text-orange-400", "{eyebrow}" }
            }
            h2 { class: "text-2xl font-black tracking-tight text-zinc-900 dark:text-white", "{title}" }
            if !description.is_empty() {
                p { class: "mt-2 max-w-2xl text-sm text-zinc-500 dark:text-zinc-400", "{description}" }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BtnVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
    Outline,
}

fn btn_classes(variant: BtnVariant, size: &str) -> String {
    let v = match variant {
        BtnVariant::Primary =>
            "bg-zinc-900 text-white hover:bg-orange-600 dark:bg-white dark:text-zinc-900 dark:hover:bg-orange-400",
        BtnVariant::Secondary =>
            "bg-orange-100 text-orange-700 hover:bg-orange-200 dark:bg-orange-950 dark:text-orange-300 dark:hover:bg-orange-900",
        BtnVariant::Ghost =>
            "text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-800 dark:hover:text-white",
        BtnVariant::Danger => "bg-red-600 text-white hover:bg-red-700",
        BtnVariant::Outline =>
            "border border-zinc-300 text-zinc-700 hover:border-orange-400 hover:text-orange-600 dark:border-zinc-700 dark:text-zinc-300 dark:hover:text-orange-400",
    };
    let s = match size {
        "sm" => "px-3 py-1.5 text-xs",
        "lg" => "px-8 py-3.5 text-sm",
        _ => "px-5 py-2.5 text-sm",
    };
    cx(&[
        "inline-flex items-center justify-center gap-2 rounded-full font-bold tracking-wide transition-colors disabled:cursor-not-allowed disabled:opacity-50",
        v,
        s,
    ])
}

/// A boxed mouse-event handler for a Button.
/// Wrap any callable (closure or callback) into a Dioxus `EventHandler`.
pub fn click<F: FnMut(Event<MouseData>) + 'static>(f: F) -> EventHandler<Event<MouseData>> {
    Callback::new(f)
}

/// Wrap any callable into a Dioxus form-input `EventHandler`.
pub fn on_input<F: FnMut(Event<FormData>) + 'static>(f: F) -> EventHandler<Event<FormData>> {
    Callback::new(f)
}

#[component]
pub fn Button(
    onclick: EventHandler<Event<MouseData>>,
    variant: BtnVariant,
    size: String,
    disabled: Option<bool>,
    class: String,
    children: Element,
) -> Element {
    let disabled = disabled.unwrap_or(false);
    rsx! {
        button {
            class: cx(&[&btn_classes(variant, &size), &class]),
            disabled: "{disabled}",
            onclick: move |evt: Event<MouseData>| onclick.call(evt),
            {children}
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BadgeTone {
    Zinc,
    Orange,
    Green,
    Red,
    Blue,
    Amber,
}

fn badge_classes(tone: BadgeTone) -> &'static str {
    match tone {
        BadgeTone::Zinc => "bg-zinc-100 text-zinc-600 dark:bg-zinc-800 dark:text-zinc-300",
        BadgeTone::Orange => "bg-orange-100 text-orange-700 dark:bg-orange-950 dark:text-orange-300",
        BadgeTone::Green => "bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300",
        BadgeTone::Red => "bg-red-100 text-red-700 dark:bg-red-950 dark:text-red-300",
        BadgeTone::Blue => "bg-sky-100 text-sky-700 dark:bg-sky-950 dark:text-sky-300",
        BadgeTone::Amber => "bg-amber-100 text-amber-700 dark:bg-amber-950 dark:text-amber-300",
    }
}

#[component]
pub fn Badge(tone: BadgeTone, class: String, children: Element) -> Element {
    rsx! {
        span { class: cx(&[
            "inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-semibold",
            badge_classes(tone),
            &class,
        ]),
            {children}
        }
    }
}

#[component]
pub fn Label(children: Element, hint: String) -> Element {
    rsx! {
        label { class: "mb-1.5 block text-sm font-medium text-zinc-800 dark:text-zinc-200",
            {children}
            if !hint.is_empty() {
                span { class: "ml-1 text-xs text-zinc-400", "{hint}" }
            }
        }
    }
}

#[component]
pub fn Input(
    oninput: EventHandler<Event<FormData>>,
    value: String,
    placeholder: String,
    r#type: String,
) -> Element {
    rsx! {
        input {
            class: "w-full rounded-xl border border-zinc-300 bg-white px-4 py-2.5 text-sm text-zinc-900 placeholder-zinc-400 outline-none transition-colors focus:border-orange-500 focus:ring-2 focus:ring-orange-500/30 disabled:opacity-60 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50 dark:placeholder-zinc-500",
            value: "{value}",
            placeholder: "{placeholder}",
            r#type: r#type,
            oninput: move |e: Event<FormData>| oninput.call(e),
        }
    }
}

#[component]
pub fn Textarea(
    oninput: EventHandler<Event<FormData>>,
    value: String,
    placeholder: String,
    rows: u32,
) -> Element {
    rsx! {
        textarea {
            class: "w-full resize-none rounded-xl border border-zinc-300 bg-white px-4 py-2.5 text-sm text-zinc-900 placeholder-zinc-400 outline-none transition-colors focus:border-orange-500 focus:ring-2 focus:ring-orange-500/30 disabled:opacity-60 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50 dark:placeholder-zinc-500",
            value: "{value}",
            placeholder: "{placeholder}",
            rows: "{rows}",
            oninput: move |e: Event<FormData>| oninput.call(e),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[component]
pub fn Select(options: Vec<SelectOption>, onchange: EventHandler<String>, value: String) -> Element {
    let items = options;
    rsx! {
        select {
            class: "w-full appearance-none rounded-xl border border-zinc-300 bg-white px-4 py-2.5 text-sm text-zinc-900 outline-none transition-colors focus:border-orange-500 focus:ring-2 focus:ring-orange-500/30 dark:border-zinc-700 dark:bg-zinc-800 dark:text-zinc-50",
            value: "{value}",
            onchange: move |e: Event<FormData>| onchange.call(e.value()),
            for opt in items {
                option { value: "{opt.value}", "{opt.label}" }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum NoticeTone {
    Info,
    Success,
    Error,
    Warning,
}

fn notice_classes(tone: NoticeTone) -> &'static str {
    match tone {
        NoticeTone::Info =>
            "border-sky-200 bg-sky-50 text-sky-800 dark:border-sky-900 dark:bg-sky-950/40 dark:text-sky-200",
        NoticeTone::Success =>
            "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-200",
        NoticeTone::Error =>
            "border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200",
        NoticeTone::Warning =>
            "border-amber-200 bg-amber-50 text-amber-800 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-200",
    }
}

#[component]
pub fn Notice(tone: NoticeTone, title: String, children: Element) -> Element {
    rsx! {
        div { class: cx(&["rounded-xl border p-3 text-sm", notice_classes(tone)]),
            if !title.is_empty() {
                p { class: "font-semibold", "{title}" }
            }
            {children}
        }
    }
}

/// Orange→emerald→blue→violet→pink gradient accents for Stat cards.
#[derive(Clone, Copy, PartialEq)]
pub enum Accent {
    Orange,
    Emerald,
    Blue,
    Violet,
    Pink,
}

fn accent_classes(a: Accent) -> &'static str {
    match a {
        Accent::Orange => "from-orange-500 to-amber-500",
        Accent::Emerald => "from-emerald-500 to-teal-500",
        Accent::Blue => "from-sky-500 to-blue-600",
        Accent::Violet => "from-violet-500 to-purple-600",
        Accent::Pink => "from-pink-500 to-rose-600",
    }
}

#[component]
pub fn Stat(label: String, value: String, sub: String, icon: IconName, accent: Accent) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-zinc-200 bg-white p-5 dark:border-zinc-800 dark:bg-zinc-900",
            div { class: "mb-3 flex items-center justify-between",
                span { class: "text-xs font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400", "{label}" }
                span { class: cx(&["flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br text-white", accent_classes(accent)]),
                    Icon { name: icon, class: "h-4 w-4" }
                }
            }
            div { class: "text-2xl font-black text-zinc-900 dark:text-white", "{value}" }
            if !sub.is_empty() {
                div { class: "mt-1 text-xs text-zinc-400", "{sub}" }
            }
        }
    }
}

#[component]
pub fn Spinner(class: String) -> Element {
    rsx! {
        svg {
            class: cx(&["animate-spin", &class]),
            view_box: "0 0 24 24",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",
            circle { cx: "12", cy: "12", r: "10", stroke: "currentColor", "stroke-width": "4", class: "opacity-25" }
            path { d: "M4 12a8 8 0 018-8", stroke: "currentColor", "stroke-width": "4", "stroke-linecap": "round", class: "opacity-90" }
        }
    }
}

#[component]
pub fn Skeleton(class: String) -> Element {
    rsx! {
        div { class: cx(&["animate-pulse rounded-lg bg-zinc-200 dark:bg-zinc-800", &class]) }
    }
}

#[component]
pub fn EmptyState(title: String, description: String, action_icon: Option<IconName>, children: Option<Element>) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center rounded-2xl border border-dashed border-zinc-300 bg-white px-6 py-14 text-center dark:border-zinc-700 dark:bg-zinc-900",
            if let Some(icon) = action_icon {
                Icon { name: icon, class: "mb-4 h-8 w-8 text-orange-500 opacity-90" }
            }
            h3 { class: "text-lg font-bold text-zinc-900 dark:text-white", "{title}" }
            if !description.is_empty() {
                p { class: "mt-1 max-w-md text-sm text-zinc-500 dark:text-zinc-400", "{description}" }
            }
            if let Some(c) = children {
                {c}
            }
        }
    }
}

/// Small metric tile (label above bold value).
#[component]
pub fn Metric(label: String, value: String) -> Element {
    rsx! {
        div { class: "rounded-xl bg-zinc-50 p-3 dark:bg-zinc-800",
            div { class: "text-xs text-zinc-400", "{label}" }
            div { class: "text-sm font-semibold text-zinc-900 dark:text-white truncate", title: "{value}", "{value}" }
        }
    }
}

/// Notice shown where a module address is not configured.
#[component]
pub fn ConnectRequired(children: Element) -> Element {
    rsx! {
        Notice { tone: NoticeTone::Warning, title: "Wallet connection required",
            {children}
        }
    }
}

/// Notice shown where a module address is not configured.
#[component]
pub fn NotConfigured(module: String, net: Network, constant_name: String) -> Element {
    rsx! {
        div { class: "rounded-xl border p-3 text-sm border-amber-200 bg-amber-50 text-amber-800 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-200",
            p { class: "font-semibold", "{module} is not configured for {net.label()}" }
            p { class: "mt-1",
                "Paste the deployed contract address into "
                code { class: "rounded bg-amber-100 px-1 font-mono dark:bg-amber-900/50", "src/config.rs" }
                " at "
                code { class: "rounded bg-amber-100 px-1 font-mono dark:bg-amber-900/50", "pub const {constant_name}" }
                "."
            }
        }
    }
}

// ───────────────────────────── transaction flow ─────────────────────────────

/// Lifecycle of one page-level write action driven through the wallet.
#[derive(Clone, PartialEq)]
pub enum TxState {
    Idle,
    Pending,
    Sent(String, String), // hash, status text
    Failed(String),
}

impl TxState {
    pub fn is_pending(&self) -> bool {
        matches!(self, TxState::Pending)
    }
}

/// Status line with the tx hash linked to the configured explorer.
#[component]
pub fn TxStatus(state: TxState) -> Element {
    match state {
        TxState::Idle => rsx! { span { } },
        TxState::Pending => rsx! {
            div { class: "flex items-center gap-2 text-sm text-zinc-500 dark:text-zinc-400",
                Spinner { class: "h-4 w-4" }
                "Waiting for the wallet and the network…"
            }
        },
        TxState::Sent(hash, text) => rsx! {
            div { class: "rounded-xl border p-3 text-sm border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-200",
                p { class: "font-semibold", "{text}" }
                if !hash.is_empty() {
                    p { class: "mt-1 font-mono text-xs break-all", "tx {hash}" }
                }
            }
        },
        TxState::Failed(err) => rsx! {
            div { class: "rounded-xl border p-3 text-sm border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200",
                p { class: "font-semibold", "Transaction failed" }
                p { class: "mt-1 font-mono text-xs break-all", "{err}" }
            }
        },
    }
}
