//! Frontend mirrors of the on-chain record types, shared by the reader in
//! `contracts.rs` and the pages.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Amount {
    pub low: u128,
    pub high: u128,
}

impl Amount {
    pub fn new(low: u128, high: u128) -> Self {
        Self { low, high }
    }

    pub fn from_low(low: u128) -> Self {
        Self { low, high: 0 }
    }

    pub fn is_zero(&self) -> bool {
        self.low == 0 && self.high == 0
    }
}

/// One juror stake deposit (per stake ID).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stake {
    pub id: String,
    pub owner: String,
    pub amount: Amount,
    pub created_at: u64,
    pub retired: bool,
}

/// Evaluation lifecycle phases stored on-chain (`constants.cairo`).
pub const PHASE_APPLYING: u8 = 1;
pub const PHASE_DRAWING: u8 = 2;
pub const PHASE_COMMIT: u8 = 3;
pub const PHASE_REVEAL: u8 = 4;
pub const PHASE_FINALIZED: u8 = 5;
pub const PHASE_DISMISSED: u8 = 6;
pub const PHASE_SLASH: u8 = 7;

/// Durable state for one candidate evaluation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Evaluation {
    pub id: String,
    pub candidate: String,
    pub requester: String,
    pub bond_amount: Amount,
    pub phase: u8,
    pub apply_deadline: u64,
    pub draw_deadline: u64,
    pub commit_deadline: u64,
    pub reveal_deadline: u64,
    pub slash_deadline: u64,
    pub applied_count: u32,
    pub drawn_count: u32,
    pub min_jurors: u32,
    pub final_score: i64,
    pub final_score_set: bool,
    pub slash_pool: Amount,
    pub governance_tokens_minted: Amount,
    pub bond_settled: bool,
}

/// Legacy dispute-shaped record (subset of Evaluation fields).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dispute {
    pub id: String,
    pub candidate: String,
    pub unique_juror_count: u32,
    pub commit_deadline: u64,
    pub reveal_deadline: u64,
    pub phase: u8,
    pub final_score: i64,
    pub final_score_set: bool,
    pub total_weight_revealed: u32,
    pub total_coherent_weight: u32,
    pub slash_pool: Amount,
    pub governance_tokens_minted: Amount,
    pub requester: String,
    pub bond_amount: Amount,
    pub bond_settled: bool,
}

/// One conviction created by a governance-token holder.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Conviction {
    pub id: u64,
    pub owner: String,
    pub level: u8,
    pub amount: Amount,
    pub created_at: u64,
    pub lock_duration: u64,
    pub votes_cast: u64,
    pub released: bool,
}

/// One funding proposal.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Proposal {
    pub author: String,
    pub funding_wallet: String,
    pub evidence: String,
    pub created_at: u64,
    pub executed: bool,
    pub power_total: Amount,
    pub deposit_amount: Amount,
    pub final_score: i64,
    pub final_score_set: bool,
}

/// A pending timelocked numeric/address change keyed by (target, param).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PendingChange {
    pub new_value: Amount,
    pub effective_at: u64,
    pub exists: bool,
}

/// A pending class-hash upgrade for a module target.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PendingUpgrade {
    pub class_hash: String,
    pub effective_at: u64,
    pub exists: bool,
}

/// Pending add/remove roster entry for an address.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PendingAdminChange {
    pub is_add: bool,
    pub effective_at: u64,
    pub exists: bool,
}

impl Evaluation {
    pub fn is_finalized(&self) -> bool {
        self.phase == PHASE_FINALIZED || self.phase == PHASE_DISMISSED
    }
}

impl Proposal {
    pub fn status_label(&self, window_elapsed: bool) -> (&'static str, ProposalTone) {
        if self.executed {
            ("Executed", ProposalTone::Green)
        } else if self.final_score_set {
            ("Scored", ProposalTone::Blue)
        } else if window_elapsed {
            ("Ready", ProposalTone::Amber)
        } else {
            ("Open", ProposalTone::Amber)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProposalTone {
    Green,
    Blue,
    Amber,
}
