#![cfg(test)]

//! Exactly one test per `Error` variant, named `error_path_<variant>` in
//! snake_case. Each test triggers the real failure path rather than
//! constructing the error value directly.
//!
//! Add a test here in the same commit that adds a variant to `src/types.rs`.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::test_helpers::{mint, setup, setup_token, synthetic_reference, DAY_SECONDS};

#[test]
fn error_path_not_initialized() {
    let env = Env::default();
    let (_, client) = setup(&env);

    assert_eq!(client.try_admin(), Err(Ok(Error::NotInitialized)));
}

#[test]
fn error_path_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(
        client.try_initialize(&admin),
        Err(Ok(Error::AlreadyInitialized))
    );
}

#[test]
fn error_path_fee_not_found() {
    let env = Env::default();
    let (_, client) = setup(&env);

    assert_eq!(client.try_get_fee(&1), Err(Ok(Error::FeeNotFound)));
    assert_eq!(client.try_status(&1), Err(Ok(Error::FeeNotFound)));
}

#[test]
fn error_path_payer_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 1);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    let fee_id = client.create_fee(&school, &token, &reference, &100, &due_at);

    let stranger = Address::generate(&env);

    assert_eq!(
        client.try_refund(&fee_id, &stranger, &1),
        Err(Ok(Error::PayerNotFound))
    );
}

#[test]
fn error_path_fee_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 2);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    let fee_id = client.create_fee(&school, &token, &reference, &100, &due_at);
    client.close_fee(&fee_id);

    let payer = Address::generate(&env);

    assert_eq!(
        client.try_pay(&fee_id, &payer, &1),
        Err(Ok(Error::FeeClosed))
    );
    assert_eq!(
        client.try_refund(&fee_id, &payer, &1),
        Err(Ok(Error::FeeClosed))
    );
    assert_eq!(client.try_close_fee(&fee_id), Err(Ok(Error::FeeClosed)));
}

#[test]
fn error_path_close_not_allowed() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, 1_000);
    let reference = synthetic_reference(&env, 3);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    let fee_id = client.create_fee(&school, &token, &reference, &100, &due_at);
    client.pay(&fee_id, &payer, &40);

    assert_eq!(
        client.try_close_fee(&fee_id),
        Err(Ok(Error::CloseNotAllowed))
    );
}

#[test]
fn error_path_due_date_in_past() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 4);
    let now = env.ledger().timestamp();

    assert_eq!(
        client.try_create_fee(&school, &token, &reference, &100, &now),
        Err(Ok(Error::DueDateInPast))
    );
}

#[test]
fn error_path_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 5);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;

    assert_eq!(
        client.try_create_fee(&school, &token, &reference, &0, &due_at),
        Err(Ok(Error::InvalidAmount))
    );

    let fee_id = client.create_fee(&school, &token, &reference, &100, &due_at);

    assert_eq!(
        client.try_pay(&fee_id, &payer, &0),
        Err(Ok(Error::InvalidAmount))
    );
    assert_eq!(
        client.try_refund(&fee_id, &payer, &0),
        Err(Ok(Error::InvalidAmount))
    );
}

#[test]
fn error_path_overpayment() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, 1_000);
    let reference = synthetic_reference(&env, 6);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    let fee_id = client.create_fee(&school, &token, &reference, &100, &due_at);

    assert_eq!(
        client.try_pay(&fee_id, &payer, &101),
        Err(Ok(Error::Overpayment))
    );
}

#[test]
fn error_path_duplicate_reference() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 7);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    client.create_fee(&school, &token, &reference, &100, &due_at);

    assert_eq!(
        client.try_create_fee(&school, &token, &reference, &50, &due_at),
        Err(Ok(Error::DuplicateReference))
    );
}

#[test]
fn error_path_refund_exceeds_paid() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, 1_000);
    let reference = synthetic_reference(&env, 8);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    let fee_id = client.create_fee(&school, &token, &reference, &100, &due_at);
    client.pay(&fee_id, &payer, &10);

    assert_eq!(
        client.try_refund(&fee_id, &payer, &11),
        Err(Ok(Error::RefundExceedsPaid))
    );
}
