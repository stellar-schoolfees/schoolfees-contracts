#![cfg(test)]

//! Happy-path and integration tests. Error paths live in `error_paths.rs`.

use super::*;
use soroban_sdk::{
    map,
    testutils::{Address as _, Events as _, Ledger as _, MockAuth, MockAuthInvoke},
    token::TokenClient,
    vec, Address, Env, IntoVal, Map, Symbol, Val,
};

use crate::storage::{DataKey, MIN_TTL_LEDGERS, SECONDS_PER_LEDGER};
use crate::test_helpers::{
    advance_time, instance_ttl, mint, mock_school_auth_for_create_fee, persistent_ttl, setup,
    setup_token, synthetic_reference, to_val, DAY_SECONDS,
};

const TOTAL: i128 = 100;

#[test]
fn initialize_records_the_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(client.admin(), admin);
}

#[test]
fn initialize_extends_the_instance_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let ttl = instance_ttl(&env, &contract_id);
    assert!(
        ttl >= storage::INSTANCE_TTL_EXTEND_TO - 1,
        "expected the instance TTL to be extended to at least {} ledgers, got {}",
        storage::INSTANCE_TTL_EXTEND_TO,
        ttl
    );
}

#[test]
fn admin_read_extends_the_instance_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Move far enough forward that the remaining TTL falls below the
    // threshold, without letting the entry lapse.
    let decayed_by = storage::INSTANCE_TTL_EXTEND_TO - storage::INSTANCE_TTL_THRESHOLD + 1;
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + decayed_by);

    let before = instance_ttl(&env, &contract_id);
    assert!(
        before < storage::INSTANCE_TTL_THRESHOLD,
        "test setup expected a TTL below the threshold, got {}",
        before
    );

    assert_eq!(client.admin(), admin);

    let after = instance_ttl(&env, &contract_id);
    assert!(
        after >= storage::INSTANCE_TTL_EXTEND_TO - 1,
        "expected the read to extend the instance TTL to at least {} ledgers, got {}",
        storage::INSTANCE_TTL_EXTEND_TO,
        after
    );
}

#[test]
fn initialize_publishes_an_initialized_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    // Event layout is documented in `docs/events.md`: topics are the event
    // name symbol followed by the admin address, and there are no data fields.
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "initialized"), admin.clone()).into_val(&env),
                Map::<Symbol, Val>::new(&env).into_val(&env),
            ),
        ]
    );
}

#[test]
fn create_fee_records_an_open_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 1);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;

    let fee_id = client.create_fee(&school, &token, &reference, &TOTAL, &due_at);

    let fee = client.get_fee(&fee_id);
    assert_eq!(fee_id, 1);
    assert_eq!(fee.id, fee_id);
    assert_eq!(fee.school, school);
    assert_eq!(fee.token, token);
    assert_eq!(fee.reference, reference);
    assert_eq!(fee.total, TOTAL);
    assert_eq!(fee.due_at, due_at);
    assert_eq!(fee.paid_total, 0);
    assert_eq!(fee.refunded_total, 0);
    assert!(!fee.closed);
    assert_eq!(client.status(&fee_id), FeeStatus::Open);
}

#[test]
fn create_fee_assigns_sequential_ids() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;

    let first = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );
    let second = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 2),
        &TOTAL,
        &due_at,
    );

    assert_eq!(first, 1);
    assert_eq!(second, 2);
}

#[test]
fn create_fee_extends_the_fee_and_reference_ttls() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 1);
    // Due in 30 days, plus the 30-day settlement margin: a 60-day horizon.
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;
    let expected = ((60 * DAY_SECONDS) / SECONDS_PER_LEDGER) as u32;

    let fee_id = client.create_fee(&school, &token, &reference, &TOTAL, &due_at);

    let fee_ttl = persistent_ttl(&env, &contract_id, &DataKey::Fee(fee_id));
    assert!(
        fee_ttl >= expected - 2,
        "expected the fee record TTL to reach about {expected} ledgers, got {fee_ttl}"
    );

    let reference_ttl = persistent_ttl(
        &env,
        &contract_id,
        &DataKey::Reference(school.clone(), reference.clone()),
    );
    assert!(
        reference_ttl >= expected - 2,
        "expected the reference index TTL to reach about {expected} ledgers, got {reference_ttl}"
    );
}

#[test]
fn create_fee_allows_the_same_reference_for_another_school() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 1);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;
    let first_school = Address::generate(&env);
    let second_school = Address::generate(&env);

    let first = client.create_fee(&first_school, &token, &reference, &TOTAL, &due_at);
    let second = client.create_fee(&second_school, &token, &reference, &TOTAL, &due_at);

    assert_eq!(first, 1);
    assert_eq!(second, 2);
}

#[test]
fn pay_records_installments_until_the_fee_is_paid() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, 1_000);
    let reference = synthetic_reference(&env, 1);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;
    let fee_id = client.create_fee(&school, &token, &reference, &TOTAL, &due_at);

    client.pay(&fee_id, &payer, &40);
    let fee = client.get_fee(&fee_id);
    assert_eq!(fee.paid_total, 40);
    assert_eq!(client.status(&fee_id), FeeStatus::Open);

    client.pay(&fee_id, &payer, &60);
    let fee = client.get_fee(&fee_id);
    assert_eq!(fee.paid_total, TOTAL);
    assert_eq!(client.status(&fee_id), FeeStatus::Paid);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&school), TOTAL);
    assert_eq!(token_client.balance(&payer), 900);
}

#[test]
fn pay_extends_the_payer_record_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, 1_000);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;
    let expected = ((60 * DAY_SECONDS) / SECONDS_PER_LEDGER) as u32;
    let fee_id = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );

    client.pay(&fee_id, &payer, &40);

    let ttl = persistent_ttl(&env, &contract_id, &DataKey::Payer(fee_id, payer.clone()));
    assert!(
        ttl >= expected - 2,
        "expected the payer record TTL to reach about {expected} ledgers, got {ttl}"
    );
}

#[test]
fn pay_after_due_date_is_allowed_and_status_becomes_overdue_then_paid() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, 1_000);
    let due_at = env.ledger().timestamp() + 100;
    let fee_id = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );

    env.ledger().set_timestamp(due_at + 1);
    assert_eq!(client.status(&fee_id), FeeStatus::Overdue);

    client.pay(&fee_id, &payer, &TOTAL);

    assert_eq!(client.status(&fee_id), FeeStatus::Paid);
    assert_eq!(TokenClient::new(&env, &token).balance(&school), TOTAL);
}

#[test]
fn get_fee_keeps_a_late_record_alive_with_the_floor_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    let fee_id = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );

    // Move to 30 days later: the 31-day horizon has one day left, so the
    // computed target drops to the 7-day floor.
    advance_time(&env, 30 * DAY_SECONDS);

    client.get_fee(&fee_id);

    let ttl = persistent_ttl(&env, &contract_id, &DataKey::Fee(fee_id));
    assert!(
        ttl >= MIN_TTL_LEDGERS - 2,
        "expected the floor bump of {MIN_TTL_LEDGERS} ledgers, got {ttl}"
    );
}

#[test]
fn close_fee_with_nothing_paid_marks_it_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;
    let fee_id = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );

    client.close_fee(&fee_id);

    assert!(client.get_fee(&fee_id).closed);
    assert_eq!(client.status(&fee_id), FeeStatus::Closed);
}

#[test]
fn close_fee_after_full_payment_marks_it_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, 1_000);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;
    let fee_id = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );
    client.pay(&fee_id, &payer, &TOTAL);

    client.close_fee(&fee_id);

    assert_eq!(client.status(&fee_id), FeeStatus::Closed);
}

#[test]
fn refund_returns_tokens_and_reopens_the_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;
    let fee_id = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );
    client.pay(&fee_id, &payer, &TOTAL);
    assert_eq!(client.status(&fee_id), FeeStatus::Paid);

    client.refund(&fee_id, &payer, &40);
    assert_eq!(client.get_fee(&fee_id).refunded_total, 40);
    assert_eq!(client.status(&fee_id), FeeStatus::Open);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&school), 60);
    assert_eq!(token_client.balance(&payer), 40);

    // Refunding the rest brings the fee back to nothing paid, so it can close.
    client.refund(&fee_id, &payer, &60);
    assert_eq!(client.status(&fee_id), FeeStatus::Open);
    client.close_fee(&fee_id);
    assert_eq!(client.status(&fee_id), FeeStatus::Closed);
}

#[test]
fn status_is_open_up_to_the_due_date_and_overdue_after() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let due_at = env.ledger().timestamp() + 1_000;
    let fee_id = client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );

    env.ledger().set_timestamp(due_at);
    assert_eq!(client.status(&fee_id), FeeStatus::Open);

    env.ledger().set_timestamp(due_at + 1);
    assert_eq!(client.status(&fee_id), FeeStatus::Overdue);
}

#[test]
fn lifecycle_publishes_documented_events() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    mint(&env, &token, &payer, TOTAL);
    let reference = synthetic_reference(&env, 9);
    let due_at = env.ledger().timestamp() + 30 * DAY_SECONDS;

    // `env.events().all()` returns the events of the last invocation - including
    // the token contract's own transfer events - so each step is asserted right
    // after its call, filtered to this contract.
    let fee_id = client.create_fee(&school, &token, &reference, &TOTAL, &due_at);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "fee_created"), fee_id).into_val(&env),
                map![
                    &env,
                    (Symbol::new(&env, "due_at"), to_val(&env, due_at)),
                    (
                        Symbol::new(&env, "reference"),
                        to_val(&env, reference.clone())
                    ),
                    (Symbol::new(&env, "school"), to_val(&env, school.clone())),
                    (Symbol::new(&env, "token"), to_val(&env, token.clone())),
                    (Symbol::new(&env, "total"), to_val(&env, TOTAL)),
                ]
                .into_val(&env),
            ),
        ]
    );

    client.pay(&fee_id, &payer, &40);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "fee_paid"), fee_id).into_val(&env),
                map![
                    &env,
                    (Symbol::new(&env, "amount"), to_val(&env, 40_i128)),
                    (Symbol::new(&env, "payer"), to_val(&env, payer.clone())),
                ]
                .into_val(&env),
            ),
        ]
    );

    client.refund(&fee_id, &payer, &40);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "fee_refunded"), fee_id).into_val(&env),
                map![
                    &env,
                    (Symbol::new(&env, "amount"), to_val(&env, 40_i128)),
                    (Symbol::new(&env, "payer"), to_val(&env, payer.clone())),
                ]
                .into_val(&env),
            ),
        ]
    );

    client.close_fee(&fee_id);
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        vec![
            &env,
            (
                contract_id.clone(),
                (Symbol::new(&env, "fee_closed"), fee_id).into_val(&env),
                Map::<Symbol, Val>::new(&env).into_val(&env),
            ),
        ]
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn initialize_requires_the_admin_signature() {
    let env = Env::default();
    let (_, client) = setup(&env);
    let admin = Address::generate(&env);

    // No auths are mocked, so the contract's `require_auth` must fail.
    client.initialize(&admin);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn create_fee_requires_the_school_signature() {
    let env = Env::default();
    let (_, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;

    client.create_fee(
        &school,
        &token,
        &synthetic_reference(&env, 1),
        &TOTAL,
        &due_at,
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn pay_requires_the_payer_signature() {
    let env = Env::default();
    let (_, client) = setup(&env);
    let payer = Address::generate(&env);

    // No auths are mocked, so the payer's `require_auth` must fail before the
    // fee lookup.
    client.pay(&1, &payer, &10);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn close_fee_requires_the_school_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let school = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 1);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    mock_school_auth_for_create_fee(
        &env,
        &contract_id,
        &school,
        &token,
        &reference,
        TOTAL,
        due_at,
    );
    let fee_id = client.create_fee(&school, &token, &reference, &TOTAL, &due_at);

    // No signature is mocked for `close_fee`.
    client.close_fee(&fee_id);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn refund_requires_the_school_signature() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 1);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    mock_school_auth_for_create_fee(
        &env,
        &contract_id,
        &school,
        &token,
        &reference,
        TOTAL,
        due_at,
    );
    let fee_id = client.create_fee(&school, &token, &reference, &TOTAL, &due_at);

    // No signature is mocked for `refund`.
    client.refund(&fee_id, &payer, &1);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn pay_rejects_a_signature_from_someone_other_than_the_payer() {
    let env = Env::default();
    let (contract_id, client) = setup(&env);
    let school = Address::generate(&env);
    let payer = Address::generate(&env);
    let stranger = Address::generate(&env);
    let token = setup_token(&env);
    let reference = synthetic_reference(&env, 1);
    let due_at = env.ledger().timestamp() + DAY_SECONDS;
    mock_school_auth_for_create_fee(
        &env,
        &contract_id,
        &school,
        &token,
        &reference,
        TOTAL,
        due_at,
    );
    let fee_id = client.create_fee(&school, &token, &reference, &TOTAL, &due_at);

    // Only the stranger's signature is provided for `pay`.
    env.mock_auths(&[MockAuth {
        address: &stranger,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "pay",
            args: (fee_id, payer.clone(), 10_i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.pay(&fee_id, &payer, &10);
}
