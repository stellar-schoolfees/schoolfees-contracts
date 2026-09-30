#![cfg(test)]

//! Happy-path and integration tests. Error paths live in `error_paths.rs`.

use super::*;
use soroban_sdk::{
    testutils::{storage::Instance as _, Address as _, Events as _, Ledger as _},
    vec, Address, Env, IntoVal, Map, Symbol, Val,
};

/// Instance TTLs can only be read inside contract context.
fn instance_ttl(env: &Env, contract_id: &Address) -> u32 {
    env.as_contract(contract_id, || env.storage().instance().get_ttl())
}

#[test]
fn initialize_records_the_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(client.admin(), admin);
}

#[test]
fn initialize_extends_the_instance_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
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
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
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
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
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
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn initialize_requires_the_admin_signature() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    // No auths are mocked, so the contract's `require_auth` must fail.
    client.initialize(&admin);
}
