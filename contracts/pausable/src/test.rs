#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_pausable_contract<'a>(env: &Env) -> (PausableContractClient<'a>, Address) {
    let contract_id = env.register_contract(None, PausableContract);
    let client = PausableContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    
    client.initialize(&admin);
    
    (client, admin)
}

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, PausableContract);
    let client = PausableContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, PausableContract);
    let client = PausableContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn test_pause_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);

    assert_eq!(client.is_paused(), false);

    client.pause(&admin);

    assert_eq!(client.is_paused(), true);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_pause_already_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);

    client.pause(&admin);
    client.pause(&admin); // Should panic
}

#[test]
fn test_unpause_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);

    client.pause(&admin);
    assert_eq!(client.is_paused(), true);

    client.unpause(&admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_unpause_not_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);

    client.unpause(&admin); // Should panic
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_pause_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = create_pausable_contract(&env);
    let unauthorized = Address::generate(&env);

    client.pause(&unauthorized); // Should panic
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_unpause_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);
    let unauthorized = Address::generate(&env);

    client.pause(&admin);
    client.unpause(&unauthorized); // Should panic
}

#[test]
fn test_set_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_set_admin_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = create_pausable_contract(&env);
    let unauthorized = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&unauthorized, &new_admin); // Should panic
}

#[test]
fn test_pause_unpause_events() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);

    client.pause(&admin);
    
    let events = env.events().all();
    let pause_event = events.iter().find(|e| {
        e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("pausable")
            && e.topics.get(1).unwrap() == &soroban_sdk::symbol_short!("paused")
    });
    assert!(pause_event.is_some());

    client.unpause(&admin);
    
    let events = env.events().all();
    let unpause_event = events.iter().find(|e| {
        e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("pausable")
            && e.topics.get(1).unwrap() == &soroban_sdk::symbol_short!("unpaused")
    });
    assert!(unpause_event.is_some());
}

#[test]
fn test_admin_changed_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);
    
    let events = env.events().all();
    let admin_changed_event = events.iter().find(|e| {
        e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("pausable")
            && e.topics.get(1).unwrap() == &soroban_sdk::symbol_short!("admin_changed")
    });
    assert!(admin_changed_event.is_some());
}

#[test]
fn test_multiple_pause_unpause_cycles() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = create_pausable_contract(&env);

    for _ in 0..3 {
        client.pause(&admin);
        assert_eq!(client.is_paused(), true);
        
        client.unpause(&admin);
        assert_eq!(client.is_paused(), false);
    }
}
