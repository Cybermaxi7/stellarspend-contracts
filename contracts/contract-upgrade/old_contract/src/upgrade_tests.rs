#![cfg(test)]
extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, BytesN, Env,
};

mod old_contract {
    soroban_sdk::contractimport!(
        file = "/home/ncdndjdj/stellarspend-contracts/target/wasm32v1-none/release/soroban_upgradeable_contract_old_contract.wasm"
    );
}

mod new_contract {
    soroban_sdk::contractimport!(
        file = "/home/ncdndjdj/stellarspend-contracts/target/wasm32v1-none/release/soroban_upgradeable_contract_new_contract.wasm"
    );
}

fn install_new_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(new_contract::WASM)
}

fn setup(e: &Env) -> (Address, Address) {
    let admin = Address::generate(e);
    let contract_id = e.register(old_contract::WASM, (&admin,));
    (admin, contract_id)
}

// Test 1: Upgrade authorization - only admin can upgrade
#[test]
#[should_panic]
fn test_unauthorized_upgrade_fails() {
    let env = Env::default();
    // no mock_all_auths - auth will fail
    let admin = Address::generate(&env);
    let contract_id = env.register(old_contract::WASM, (&admin,));
    let client = old_contract::Client::new(&env, &contract_id);
    let new_wasm_hash = install_new_wasm(&env);
    client.upgrade(&new_wasm_hash);
}

// Test 2: Upgrade emits event
#[test]
fn test_upgrade_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, contract_id) = setup(&env);
    let client = old_contract::Client::new(&env, &contract_id);
    let new_wasm_hash = install_new_wasm(&env);
    client.upgrade(&new_wasm_hash);
    let events = env.events().all();
    assert!(!events.is_empty(), "upgrade should emit an event");
}

// Test 3: State is preserved after upgrade
#[test]
fn test_state_preserved_after_upgrade() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, contract_id) = setup(&env);
    let client = old_contract::Client::new(&env, &contract_id);
    assert_eq!(1, client.version());
    let new_wasm_hash = install_new_wasm(&env);
    client.upgrade(&new_wasm_hash);
    // version should now be 2
    let new_client = new_contract::Client::new(&env, &contract_id);
    assert_eq!(2, new_client.version());
}

// Test 4: Upgrade without handle_upgrade fails second upgrade
#[test]
fn test_second_upgrade_fails_without_migration() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, contract_id) = setup(&env);
    let client = old_contract::Client::new(&env, &contract_id);
    let new_wasm_hash = install_new_wasm(&env);
    client.upgrade(&new_wasm_hash);
    let new_client = new_contract::Client::new(&env, &contract_id);
    // NewAdmin key not set yet, second upgrade should fail
    let result = new_client.try_upgrade(&new_wasm_hash);
    assert!(result.is_err(), "upgrade should fail without handle_upgrade");
}

// Test 5: handle_upgrade properly migrates state
#[test]
fn test_handle_upgrade_migrates_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, contract_id) = setup(&env);
    let client = old_contract::Client::new(&env, &contract_id);
    let new_wasm_hash = install_new_wasm(&env);
    client.upgrade(&new_wasm_hash);
    let new_client = new_contract::Client::new(&env, &contract_id);
    new_client.handle_upgrade();
    // after migration, upgrade should succeed
    let result = new_client.try_upgrade(&new_wasm_hash);
    assert!(result.is_ok(), "upgrade should succeed after handle_upgrade");
}