#![no_std]

// Keep this file thin: `#[contract]` and `#[contractimpl]` only. Logic,
// types, and storage rules live in the modules below.
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::storage::{extend_instance_ttl, DataKey};
pub use crate::types::{Error, Initialized};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// One-time setup: records `admin` as the contract administrator and
    /// requires `admin`'s signature.
    ///
    /// Errors:
    /// - [`Error::AlreadyInitialized`] if the contract already has an admin.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        let key = DataKey::Admin;
        if env.storage().instance().has(&key) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&key, &admin);

        extend_instance_ttl(&env);
        Initialized { admin }.publish(&env);
        Ok(())
    }

    /// Returns the recorded administrator address.
    ///
    /// Errors:
    /// - [`Error::NotInitialized`] if `initialize` has not been called yet.
    pub fn admin(env: Env) -> Result<Address, Error> {
        let key = DataKey::Admin;
        let admin: Address = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::NotInitialized)?;

        extend_instance_ttl(&env);
        Ok(admin)
    }
}

mod error_paths;
mod test;
