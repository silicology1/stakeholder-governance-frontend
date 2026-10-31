//! Network configuration, per-chain module addresses, and protocol constants
//! mirrored from the modular contract's `src/constants.cairo`.
//!
//! # Deployment
//!
//! The four module classes are declared separately (`scarb build` on the
//! contracts workspace produces four class hashes). After deploying each one,
//! paste the resulting addresses below for each network. A zero address means
//! "not configured" and every module page shows a configuration notice
//! instead of attempting (failing) RPC calls.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Network {
    Sepolia,
    Mainnet,
}

impl Network {
    pub const ALL: [Network; 2] = [Network::Sepolia, Network::Mainnet];

    pub fn key(self) -> &'static str {
        match self {
            Network::Sepolia => "sepolia",
            Network::Mainnet => "mainnet",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Network::Sepolia => "Starknet Sepolia",
            Network::Mainnet => "Starknet Mainnet",
        }
    }

    /// Felt-represented chain id returned by `wallet_requestChainId`.
    pub fn chain_id_hex(self) -> &'static str {
        match self {
            Network::Sepolia => "0x534e5f5345504f4c4941", // SN_SEPOLIA
            Network::Mainnet => "0x534e5f4d41494e",       // SN_MAIN
        }
    }

    pub fn rpc_url(self) -> &'static str {
        match self {
            Network::Sepolia => SEPOLIA_RPC,
            Network::Mainnet => MAINNET_RPC,
        }
    }

    pub fn explorer_base(self) -> &'static str {
        match self {
            Network::Sepolia => "https://sepolia.voyager.online",
            Network::Mainnet => "https://voyager.online",
        }
    }

    pub fn config_prefix(self) -> &'static str {
        match self {
            Network::Sepolia => "SEPOLIA_",
            Network::Mainnet => "MAINNET_",
        }
    }
}

/// Public Sepolia JSON-RPC endpoint.
pub const SEPOLIA_RPC: &str = "https://starknet-sepolia.drpc.org";
/// Public Mainnet JSON-RPC endpoint.
pub const MAINNET_RPC: &str = "https://starknet-mainnet.drpc.org";

/// How many blocks of history a page scans when indexing `#[key]`-ed event
/// IDs. Public nodes cap deep scans, so ids older than this window simply do
/// not appear in list views.
pub const EVENT_LOOKBACK: u64 = 45_000;

/// Placeholder bridge address (same on Sepolia and Mainnet) used to pay gas.
pub const STRK: &str = "0x04718f5a0Fc34cC1AF16A1cdee98fFB20C31f5cD61D6Ab07201858f4287c938D";

// ─────────────────────────── module addresses ───────────────────────────────
//
// Replace each zero address after `scarb declare` + `deploy`. The app treats
// zero as "not configured" and disables live reads/writes on that module.

/// ConvictionToken — transferable ERC20 with decay, monthly budget, escrow hub.
pub const SEPOLIA_CONVICTION_TOKEN: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
/// Selection — juror staking, sortition, commit/reveal, slashing.
pub const SEPOLIA_SELECTION: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
/// Governance — grants, convictions, funding proposals.
pub const SEPOLIA_GOVERNANCE: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
/// AdminTimelock — parameter timelock, roster, upgrade orchestration.
pub const SEPOLIA_ADMIN_TIMELOCK: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

/// ConvictionToken (mainnet deployment).
pub const MAINNET_CONVICTION_TOKEN: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
/// Selection (mainnet deployment).
pub const MAINNET_SELECTION: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
/// Governance (mainnet deployment).
pub const MAINNET_GOVERNANCE: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";
/// AdminTimelock (mainnet deployment).
pub const MAINNET_ADMIN_TIMELOCK: &str =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

/// Explorer URL for a contract address or tx hash on this network.
pub fn explorer_address_url(net: Network, address: &str) -> String {
    format!(
        "{}/contract/{}",
        net.explorer_base(),
        crate::chain::normalized_hex(address)
    )
}

pub fn explorer_tx_url(net: Network, tx_hash: &str) -> String {
    format!("{}/tx/{}", net.explorer_base(), crate::chain::normalized_hex(tx_hash))
}

// ─────────────────────────── protocol constants ─────────────────────────────

pub const TOKEN_SYMBOL: &str = "STK";
/// Symbol of the native gas token whose balance is shown on the account card.
pub const GAS_SYMBOL: &str = "STRK";
pub const TOKEN_DECIMALS: u32 = 18;
pub const MAX_EVIDENCE_LEN: usize = 1024;
pub const MAX_CONVICT_LEVEL: u32 = 10;
pub const MAX_BATCH_DECAY: u32 = 50;
pub const MAX_COHERENCE_BAND: u32 = 3;

/// 3-day parameter timelock.
pub const TIMELOCK_SECS: u64 = 259_200;
/// 7-day upgrade timelock.
pub const UPGRADE_TIMELOCK_SECS: u64 = 604_800;
