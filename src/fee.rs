//! Fee lifecycle logic.
//!
//! `src/lib.rs` exposes these functions through `#[contractimpl]`; everything
//! that validates input, reads or writes storage, or moves tokens lives here.
//!
//! Arithmetic: amounts are validated first (`total > 0`, `amount <= remaining`,
//! refunds capped at what a payer paid), and the release profile builds with
//! `overflow-checks = true` (see `Cargo.toml`), so an overflow traps instead of
//! wrapping silently.

use soroban_sdk::{token::TokenClient, Address, BytesN, Env};

use crate::storage::{extend_instance_ttl, extend_record_ttl, DataKey};
use crate::types::{
    Error, Fee, FeeClosed, FeeCreated, FeePaid, FeeRefunded, FeeStatus, PayerRecord,
};

/// Adds two amounts that have already been validated as positive and bounded.
///
/// Overflow is unreachable in practice: each successful update is bounded by
/// the fee's `total` (at most `i128::MAX`) or by what a payer already paid, and
/// exhausting `i128` would take more transactions than a ledger sequence can
/// ever hold. Written as a checked add so the failure mode is a trap, never a
/// silent wrap.
fn add(a: i128, b: i128) -> i128 {
    a.checked_add(b)
        .unwrap_or_else(|| panic!("amount overflow"))
}

/// Loads a fee or reports its absence.
fn load_fee(env: &Env, fee_id: u64) -> Result<Fee, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Fee(fee_id))
        .ok_or(Error::FeeNotFound)
}

/// Everything a fee still owes after refunds: `paid_total - refunded_total`.
fn net_paid(fee: &Fee) -> i128 {
    fee.paid_total - fee.refunded_total
}

/// Records a new fee for `school` against an opaque reference.
pub fn create_fee(
    env: &Env,
    school: Address,
    token: Address,
    reference: BytesN<32>,
    total: i128,
    due_at: u64,
) -> Result<u64, Error> {
    school.require_auth();

    if total <= 0 {
        return Err(Error::InvalidAmount);
    }
    let now = env.ledger().timestamp();
    if due_at <= now {
        return Err(Error::DueDateInPast);
    }

    let reference_key = DataKey::Reference(school.clone(), reference.clone());
    if env.storage().persistent().has(&reference_key) {
        return Err(Error::DuplicateReference);
    }

    let counter_key = DataKey::NextFeeId;
    let fee_id: u64 = env.storage().instance().get(&counter_key).unwrap_or(1);
    let next_id = fee_id
        .checked_add(1)
        .unwrap_or_else(|| panic!("fee id overflow"));

    let fee = Fee {
        id: fee_id,
        school: school.clone(),
        token: token.clone(),
        reference: reference.clone(),
        total,
        due_at,
        paid_total: 0,
        refunded_total: 0,
        closed: false,
    };
    let fee_key = DataKey::Fee(fee_id);
    env.storage().instance().set(&counter_key, &next_id);
    env.storage().persistent().set(&fee_key, &fee);
    env.storage().persistent().set(&reference_key, &fee_id);

    extend_instance_ttl(env);
    extend_record_ttl(env, &fee_key, due_at);
    extend_record_ttl(env, &reference_key, due_at);

    FeeCreated {
        fee_id,
        school,
        token,
        reference,
        total,
        due_at,
    }
    .publish(env);

    Ok(fee_id)
}

/// Pays `amount` toward a fee, moving tokens from `payer` straight to the school.
pub fn pay(env: &Env, fee_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
    payer.require_auth();

    let mut fee = load_fee(env, fee_id)?;
    if fee.closed {
        return Err(Error::FeeClosed);
    }
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    let remaining = fee.total - net_paid(&fee);
    if amount > remaining {
        return Err(Error::Overpayment);
    }

    let payer_key = DataKey::Payer(fee_id, payer.clone());
    let mut record: PayerRecord =
        env.storage()
            .persistent()
            .get(&payer_key)
            .unwrap_or(PayerRecord {
                paid: 0,
                refunded: 0,
            });
    record.paid = add(record.paid, amount);
    env.storage().persistent().set(&payer_key, &record);

    fee.paid_total = add(fee.paid_total, amount);
    let fee_key = DataKey::Fee(fee_id);
    env.storage().persistent().set(&fee_key, &fee);

    TokenClient::new(env, &fee.token).transfer(&payer, &fee.school, &amount);

    extend_record_ttl(env, &fee_key, fee.due_at);
    extend_record_ttl(env, &payer_key, fee.due_at);

    FeePaid {
        fee_id,
        payer,
        amount,
    }
    .publish(env);

    Ok(())
}

/// Closes a fee that has nothing owed (fully paid) or nothing paid.
pub fn close_fee(env: &Env, fee_id: u64) -> Result<(), Error> {
    let mut fee = load_fee(env, fee_id)?;
    fee.school.require_auth();

    if fee.closed {
        return Err(Error::FeeClosed);
    }
    let net = net_paid(&fee);
    if net != 0 && net != fee.total {
        return Err(Error::CloseNotAllowed);
    }

    fee.closed = true;
    let fee_key = DataKey::Fee(fee_id);
    env.storage().persistent().set(&fee_key, &fee);
    extend_record_ttl(env, &fee_key, fee.due_at);

    FeeClosed { fee_id }.publish(env);

    Ok(())
}

/// Refunds a payer from the school's own balance, capped at what that payer
/// still has paid.
pub fn refund(env: &Env, fee_id: u64, payer: Address, amount: i128) -> Result<(), Error> {
    let mut fee = load_fee(env, fee_id)?;
    fee.school.require_auth();

    if fee.closed {
        return Err(Error::FeeClosed);
    }
    if amount <= 0 {
        return Err(Error::InvalidAmount);
    }

    let payer_key = DataKey::Payer(fee_id, payer.clone());
    let mut record: PayerRecord = env
        .storage()
        .persistent()
        .get(&payer_key)
        .ok_or(Error::PayerNotFound)?;
    let refundable = record.paid - record.refunded;
    if amount > refundable {
        return Err(Error::RefundExceedsPaid);
    }

    record.refunded = add(record.refunded, amount);
    env.storage().persistent().set(&payer_key, &record);

    fee.refunded_total = add(fee.refunded_total, amount);
    let fee_key = DataKey::Fee(fee_id);
    env.storage().persistent().set(&fee_key, &fee);

    TokenClient::new(env, &fee.token).transfer(&fee.school, &payer, &amount);

    extend_record_ttl(env, &fee_key, fee.due_at);
    extend_record_ttl(env, &payer_key, fee.due_at);

    FeeRefunded {
        fee_id,
        payer,
        amount,
    }
    .publish(env);

    Ok(())
}

/// Returns a fee record, extending its TTL.
pub fn get_fee(env: &Env, fee_id: u64) -> Result<Fee, Error> {
    let fee = load_fee(env, fee_id)?;
    extend_record_ttl(env, &DataKey::Fee(fee_id), fee.due_at);
    Ok(fee)
}

/// Derives a fee's status, extending the record's TTL.
pub fn status(env: &Env, fee_id: u64) -> Result<FeeStatus, Error> {
    let fee = load_fee(env, fee_id)?;
    extend_record_ttl(env, &DataKey::Fee(fee_id), fee.due_at);

    let status = if fee.closed {
        FeeStatus::Closed
    } else if net_paid(&fee) >= fee.total {
        FeeStatus::Paid
    } else if env.ledger().timestamp() > fee.due_at {
        FeeStatus::Overdue
    } else {
        FeeStatus::Open
    };

    Ok(status)
}
