#![no_std]

// Keep this file thin: `#[contract]` and `#[contractimpl]` only. Logic,
// types, and storage rules live in the modules below.
mod fee;
mod storage;
mod types;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::storage::{extend_instance_ttl, DataKey};
pub use crate::types::{Error, Fee, FeeStatus, Initialized};

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

    /// Records a new fee obligation for `school` against an opaque 32-byte
    /// `reference`, denominated in `token`, due at `due_at` (Unix seconds).
    ///
    /// Errors: [`Error::InvalidAmount`] for `total <= 0`;
    /// [`Error::DueDateInPast`] when `due_at` is not in the future;
    /// [`Error::DuplicateReference`] when this school already used the
    /// reference.
    pub fn create_fee(
        env: Env,
        school: Address,
        token: Address,
        reference: BytesN<32>,
        total: i128,
        due_at: u64,
    ) -> Result<u64, Error> {
        fee::create_fee(&env, school, token, reference, total, due_at)
    }

    /// Pays `amount` toward fee `fee_id`, transferring from `payer` straight to
    /// the school's token balance.
    ///
    /// Errors: [`Error::FeeNotFound`], [`Error::FeeClosed`],
    /// [`Error::InvalidAmount`], [`Error::Overpayment`].
    pub fn pay(env: Env, fee_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
        fee::pay(&env, fee_id, payer, amount)
    }

    /// Closes a fee that has nothing owed or nothing paid.
    ///
    /// Errors: [`Error::FeeNotFound`], [`Error::FeeClosed`],
    /// [`Error::CloseNotAllowed`] for a partially paid fee.
    pub fn close_fee(env: Env, fee_id: u64) -> Result<(), Error> {
        fee::close_fee(&env, fee_id)
    }

    /// Refunds `amount` from the school's own balance to `payer`, capped at
    /// what that payer still has paid.
    ///
    /// Errors: [`Error::FeeNotFound`], [`Error::FeeClosed`],
    /// [`Error::InvalidAmount`], [`Error::PayerNotFound`],
    /// [`Error::RefundExceedsPaid`].
    pub fn refund(env: Env, fee_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
        fee::refund(&env, fee_id, payer, amount)
    }

    /// Returns the fee record.
    ///
    /// Errors: [`Error::FeeNotFound`].
    pub fn get_fee(env: Env, fee_id: u64) -> Result<Fee, Error> {
        fee::get_fee(&env, fee_id)
    }

    /// Returns the fee's derived status.
    ///
    /// Errors: [`Error::FeeNotFound`].
    pub fn status(env: Env, fee_id: u64) -> Result<FeeStatus, Error> {
        fee::status(&env, fee_id)
    }
}

mod error_paths;
mod test;
mod test_helpers;
