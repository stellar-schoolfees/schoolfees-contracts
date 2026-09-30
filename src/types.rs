//! Contract error codes.
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
//!
//! NOTE: this enum currently covers only the initialization surface. The
//! lifecycle, timing, validation, and authorization variants land together
//! with the v0 fee lifecycle, once the contract design is fixed.

use soroban_sdk::{contracterror, contractevent, Address};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // 1-9: Initialization & lookup
    /// The contract has not been initialized yet.
    NotInitialized = 1,
    /// The contract has already been initialized. Initialization is one-time.
    AlreadyInitialized = 2,
    // No other variants yet. Ranges 10-29 (lifecycle & timing) and 30-49
    // (validation & authorization) are reserved for the v0 fee lifecycle.
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
