//! Storage keys, TTL constants, and TTL-extension helpers.
//!
//! Contract-wide values (the admin address today, and the contract
//! configuration that lands with the v0 fee lifecycle) live in instance
//! storage. Each fee record will get its own persistent entry.
//!
//! Read the State Archival guide before changing anything here:
//! <https://developers.stellar.org/docs/learn/fundamentals/contract-development/storage/state-archival>

use soroban_sdk::{contracttype, Env};

/// Stellar ledgers close roughly every 5 seconds, so a day is about 17,280
/// ledgers. TTL values are expressed in ledgers.
pub const DAY_IN_LEDGERS: u32 = 17_280;

/// Instance storage is extended when its remaining TTL falls below this.
pub const INSTANCE_TTL_THRESHOLD: u32 = 7 * DAY_IN_LEDGERS;

/// Instance storage is extended to at least this much remaining TTL.
pub const INSTANCE_TTL_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;

/// Keys for contract-wide (instance) storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// The account allowed to administer the contract.
    Admin,
}

/// Extends the TTL of the contract instance and its instance entries.
///
/// Call this whenever instance storage is read or written, so contract-wide
/// values do not archive. A flat threshold is correct for these values because
/// they have no natural deadline; fee records added with the v0 lifecycle get
/// deadline-based helpers here, computed from the record's due date plus a
/// safety margin.
pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}
