#![cfg(test)]

//! Shared setup helpers for `test.rs` and `error_paths.rs`.

use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, Ledger as _, MockAuth, MockAuthInvoke,
    },
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, Val,
};

use crate::{
    storage::{DataKey, SECONDS_PER_LEDGER},
    Contract, ContractClient,
};

/// Seconds in a day, for readable test deadlines.
pub const DAY_SECONDS: u64 = 24 * 60 * 60;

/// Registers the contract and returns its address and a client for it.
pub fn setup(env: &Env) -> (Address, ContractClient<'_>) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    (contract_id, client)
}

/// A synthetic 32-byte reference. Tests never use a value derived from a real
/// person.
pub fn synthetic_reference(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// Converts a value to a `Val` with the target type pinned, for building
/// expected event data maps (plain `into_val` is ambiguous there).
pub fn to_val<T: IntoVal<Env, Val>>(env: &Env, value: T) -> Val {
    value.into_val(env)
}

/// Registers a Stellar Asset Contract for tests and returns its address.
pub fn setup_token(env: &Env) -> Address {
    let admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(admin).address()
}

/// Mints test tokens. Requires mocked auth (the SAC admin's signature).
pub fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

/// Moves the ledger forward by `seconds`, keeping sequence and timestamp in
/// step (at roughly 5 seconds per ledger).
pub fn advance_time(env: &Env, seconds: u64) {
    let ledgers = u32::try_from(seconds / SECONDS_PER_LEDGER).unwrap();
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + ledgers);
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + seconds);
}

/// Remaining TTL of a persistent entry, read inside the contract's context.
pub fn persistent_ttl(env: &Env, contract_id: &Address, key: &DataKey) -> u32 {
    env.as_contract(contract_id, || env.storage().persistent().get_ttl(key))
}

/// Remaining TTL of the contract instance.
pub fn instance_ttl(env: &Env, contract_id: &Address) -> u32 {
    env.as_contract(contract_id, || env.storage().instance().get_ttl())
}

/// Mocks only the school's signature for the `create_fee` invocation.
pub fn mock_school_auth_for_create_fee(
    env: &Env,
    contract_id: &Address,
    school: &Address,
    token: &Address,
    reference: &BytesN<32>,
    total: i128,
    due_at: u64,
) {
    env.mock_auths(&[MockAuth {
        address: school,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "create_fee",
            args: (
                school.clone(),
                token.clone(),
                reference.clone(),
                total,
                due_at,
            )
                .into_val(env),
            sub_invokes: &[],
        },
    }]);
}
