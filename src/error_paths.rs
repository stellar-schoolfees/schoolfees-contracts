#![cfg(test)]

//! Exactly one test per `Error` variant, named `error_path_<variant>` in
//! snake_case. Each test triggers the real failure path rather than
//! constructing the error value directly.
//!
//! Add a test here in the same commit that adds a variant to `src/types.rs`.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn error_path_not_initialized() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.try_admin(), Err(Ok(Error::NotInitialized)));
}

#[test]
fn error_path_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(
        client.try_initialize(&admin),
        Err(Ok(Error::AlreadyInitialized))
    );
}
