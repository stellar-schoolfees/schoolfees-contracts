//! Contract error codes, stored types, and events.
//!
//! Codes are grouped into ranges by category so that new variants can be added
//! without renumbering the ones that already exist:
//!
//! ```text
//! 1-9    Initialization & lookup
//! 10-29  Lifecycle & timing
//! 30-49  Validation & authorization
//! ```
//!
//! Every variant here has exactly one row in `ERRORS.md` at the repository
//! root and exactly one test in `src/error_paths.rs`. `scripts/check-errors.mjs`
//! fails CI if this enum and `ERRORS.md` drift apart.

use soroban_sdk::{contracterror, contractevent, contracttype, Address, BytesN};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // 1-9: Initialization & lookup
    /// The contract has not been initialized yet.
    NotInitialized = 1,
    /// The contract has already been initialized. Initialization is one-time.
    AlreadyInitialized = 2,
    /// No fee record exists for the given id.
    FeeNotFound = 3,
    /// No payer record exists for the given fee and payer.
    PayerNotFound = 4,
    // 10-29: Lifecycle & timing
    /// The fee is closed; no further payment, refund or close is possible.
    FeeClosed = 10,
    /// Only a fee with nothing owed or nothing paid can be closed.
    CloseNotAllowed = 11,
    /// `create_fee` was called with a due date that is not in the future.
    DueDateInPast = 12,
    // 30-49: Validation & authorization
    /// An amount was zero or negative.
    InvalidAmount = 30,
    /// A payment exceeded the amount still owed.
    Overpayment = 31,
    /// The school already has a fee with this reference.
    DuplicateReference = 32,
    /// A refund exceeded what the payer still has paid.
    RefundExceedsPaid = 33,
}

/// A fee obligation recorded on-chain.
///
/// The reference is opaque: the client derives it off-chain (for example as a
/// hash) and it must never encode personal data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fee {
    /// Fee id, starting at 1.
    pub id: u64,
    /// The school that created the fee and receives payments.
    pub school: Address,
    /// The SEP-41 token the fee is denominated in.
    pub token: Address,
    /// Opaque 32-byte reference chosen by the school.
    pub reference: BytesN<32>,
    /// Total amount owed.
    pub total: i128,
    /// Due date, in Unix seconds (the same clock as `env.ledger().timestamp()`).
    pub due_at: u64,
    /// Sum of every payer's payments.
    pub paid_total: i128,
    /// Sum of every payer's refunds.
    pub refunded_total: i128,
    /// Set once by `close_fee`; terminal.
    pub closed: bool,
}

/// What a single payer has paid into one fee, and got back.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayerRecord {
    /// Everything this payer paid.
    pub paid: i128,
    /// Everything this payer has been refunded.
    pub refunded: i128,
}

/// Derived state of a fee; computed on read, never stored.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeeStatus {
    /// Before the due date, with a balance still owed.
    Open,
    /// Fully paid (`paid_total - refunded_total == total`) and not closed.
    Paid,
    /// Past the due date with a balance still owed.
    Overdue,
    /// Closed by the school; terminal.
    Closed,
}

/// Emitted by `initialize` once the contract administrator is recorded.
///
/// Layout (see `docs/events.md`): topic `admin`, no data fields.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialized {
    /// The account recorded as the contract administrator.
    #[topic]
    pub admin: Address,
}

/// Emitted by `create_fee` once a fee is recorded.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeCreated {
    /// The new fee's id.
    #[topic]
    pub fee_id: u64,
    /// The school that created the fee.
    pub school: Address,
    /// The token the fee is denominated in.
    pub token: Address,
    /// The opaque reference.
    pub reference: BytesN<32>,
    /// Total amount owed.
    pub total: i128,
    /// Due date, in Unix seconds.
    pub due_at: u64,
}

/// Emitted by `pay` for every successful payment.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeePaid {
    /// The fee that received the payment.
    #[topic]
    pub fee_id: u64,
    /// The payer whose signature authorized the transfer.
    pub payer: Address,
    /// Amount paid, in the fee's token.
    pub amount: i128,
}

/// Emitted by `close_fee` once the fee is closed.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeClosed {
    /// The fee that was closed.
    #[topic]
    pub fee_id: u64,
}

/// Emitted by `refund` for every successful refund.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeRefunded {
    /// The fee the refund belongs to.
    #[topic]
    pub fee_id: u64,
    /// The payer who received the refund.
    pub payer: Address,
    /// Amount refunded, in the fee's token.
    pub amount: i128,
}
