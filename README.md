# Stakeholder Governance — Frontend (Dioxus)

A Rust/web frontend for the [`stakeholder-governance-modular`](https://github.com/silicology1/stakeholder-governance) Cairo
contract suite, built with [Dioxus 0.7](https://dioxuslabs.com/learn/0.7/) and Tailwind CSS v4
(standalone CLI, no Play CDN).

## Pages

The UI is designed around the four modular contracts rather than ported from the legacy app:

| Route | Purpose |
|---|---|
| `/` | Landing page + overview of the four modules |
| `/dashboard` | Portfolio: token, stake, bonding, conviction & reputation at a glance |
| `/token` | Governance token balances / mint info |
| `/stake` | Stake bonding (open, extend, exit) |
| `/proposals` | Proposal list |
| `/proposals/create` | Submit `create_proposal` (funding wallet + evidence) |
| `/proposals/:id` | Proposal detail, conviction voting, execute |
| `/convictions` | conviction list |
| `/convictions/create` | `create_conviction(level, amount)` |
| `/evaluations` | Evaluation list |
| `/evaluations/create` | `apply_to_evaluation` |
| `/evaluations/:id` | Juror flow: draw → commit → reveal → finalize, rewards, disputes |
| `/governance` | Token/conviction module governance parameters |
| `/admin` | Admin timelock: protected params, add-admin roster, upgrades |

## Architecture

- **Reads** go through `starknet-rust 0.20` (`provider()` JSON-RPC via `chain.rs`) — no wallet needed.
- **Writes** are built as raw calldata in `contracts.rs` (hex-felt JSON) and **signed by the browser
  wallet** through `assets/wallet.js` (`wallet_addInvokeTransaction`), wrapped by `src/wallet.rs`.
- `src/state.rs` holds the app context (network, connected account, 1 Hz clock) shared via
  `use_context`.
- Network switcher: **Starknet Sepolia / Mainnet** (persisted in `localStorage` as `sg_network`).
  See `src/config.rs`; the chain id is verified against the wallet after connect.
- Compile-time poseidon hashing (commit phase) is done with `starknet-types-core` on the wasm side.

### Contract addresses

All four module addresses live in `src/config.rs` as **zero placeholders** — nothing is deployed yet.
While an address is `0x0`, that page shows an amber "not configured" notice and disables reads;
writes are also refused. Paste real addresses there when the contracts are deployed.

## Development

```sh
npm install            # one-time: Tailwind standalone CLI
npm run tailwind:watch # recompile assets/tailwind.css on change (optional)
dx serve               # full dev loop with hot reload on http://localhost:8080
```

`dx serve` compiles to `wasm32-unknown-unknown`, runs wasm-bindgen (the `assets/wallet.js` shim is
bundled automatically as a snippet) and serves the app.

Production bundle:

```sh
npm run tailwind:build
dx build               # output in target/dx/<app>/release/web/public
```

Type-check only:

```sh
cargo check --target wasm32-unknown-unknown
```

```bash
dx serve
```


```bash
tailwindcss -i ./input.css -o ./assets/tailwind.css --watch
```

Format
```bash
dx fmt
```

```bash
dx bundle --release
```


## OG image compression

```bash
convert og-image.png  -quality 10 og-image.jpg
```

## Styling

- Source: `assets/css/input.css` → compiled output: `assets/tailwind.css`
  (referenced via `document::Link` in `src/main.rs`).
- Design language carried over from the legacy app: amber/orange accents on zinc, dark mode
  (class-based, toggled in the navbar, persisted), `rounded-2xl` cards and pill buttons —
  see `src/ui.rs` for the shared components (`Card`, `Button`, `Input`, `Badge`, `Notice`, …).
