//! Comprehensive tests for pausable contract functionality
//! Tests admin-only pause/unpause, event emission, and access restrictions

#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

// Mock contract structure for testing
mod pausable_mock {
    use soroban_sdk::{contract, contractimpl, contracttype, panic_with_error, Address, Env};

    #[contracttype]
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum DataKey {
        Admin,
        Paused,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum PausableError {
        NotInitialized = 1,
        Unauthorized = 2,
        ContractPaused = 3,
        ContractNotPaused = 4,
    }

    impl From<PausableError> for soroban_sdk::Error {
        fn from(e: PausableError) -> Self {
            soroban_sdk::Error::from_contract_error(e as u32)
        }
    }

    #[contract]
    pub struct PausableTestContract;

    #[contractimpl]
    impl PausableTestContract {
        pub fn initialize(env: Env, admin: Address) {
            if env.storage().instance().has(&DataKey::Admin) {
                panic!("Contract already initialized");
            }
            admin.require_auth();
            env.storage().instance().set(&DataKey::Admin, &admin);
            env.storage().instance().set(&DataKey::Paused, &false);
            env.events().publish(("pausable", "initialized"), admin);
        }

        pub fn pause(env: Env, caller: Address) {
            caller.require_auth();
            Self::require_admin(&env, &caller);

            let is_paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
            if is_paused {
                panic_with_error!(&env, PausableError::ContractPaused);
            }

            env.storage().instance().set(&DataKey::Paused, &true);
            env.events().publish(("pausable", "paused"), caller);
        }

        pub fn unpause(env: Env, caller: Address) {
            caller.require_auth();
            Self::require_admin(&env, &caller);

            let is_paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
            if !is_paused {
                panic_with_error!(&env, PausableError::ContractNotPaused);
            }

            env.storage().instance().set(&DataKey::Paused, &false);
            env.events().publish(("pausable", "unpaused"), caller);
        }

        pub fn is_paused(env: Env) -> bool {
            env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
        }

        pub fn get_admin(env: Env) -> Address {
            env.storage()
                .instance()
                .get(&DataKey::Admin)
                .expect("Contract not initialized")
        }

        pub fn set_admin(env: Env, current_admin: Address, new_admin: Address) {
            current_admin.require_auth();
            Self::require_admin(&env, &current_admin);
            env.storage().instance().set(&DataKey::Admin, &new_admin);
            env.events()
                .publish(("pausable", "admin_changed"), (current_admin, new_admin));
        }

        pub fn critical_operation(env: Env, caller: Address, value: i128) -> i128 {
            caller.require_auth();
            Self::require_not_paused(&env);
            value * 2
        }

        fn require_admin(env: &Env, caller: &Address) {
            let admin: Address = env
                .storage()
                .instance()
                .get(&DataKey::Admin)
                .expect("Contract not initialized");
            if *caller != admin {
                panic_with_error!(env, PausableError::Unauthorized);
            }
        }

        fn require_not_paused(env: &Env) {
            let is_paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
            if is_paused {
                panic_with_error!(env, PausableError::ContractPaused);
            }
        }
    }
}

use pausable_mock::{PausableTestContract, PausableTestContractClient};

fn setup_contract(env: &Env) -> (PausableTestContractClient, Address) {
    let contract_id = env.register_contract(None, PausableTestContract);
    let client = PausableTestContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin);
    (client, admin)
}

// ===== Initialization Tests =====

#[test]
fn test_initialize_sets_admin_and_unpaused_state() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);
    client.initialize(&admin);
}

// ===== Pause Tests =====

#[test]
fn test_admin_can_pause_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);

    assert_eq!(client.is_paused(), true);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_contract(&env);
    let non_admin = Address::generate(&env);

    client.pause(&non_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_cannot_pause_already_paused_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);
    client.pause(&admin);
}

// ===== Unpause Tests =====

#[test]
fn test_admin_can_unpause_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);
    assert_eq!(client.is_paused(), true);

    client.unpause(&admin);
    assert_eq!(client.is_paused(), false);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);
    let non_admin = Address::generate(&env);

    client.pause(&admin);
    client.unpause(&non_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_cannot_unpause_not_paused_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.unpause(&admin);
}

// ===== Critical Function Tests =====

#[test]
fn test_critical_function_works_when_not_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    let result = client.critical_operation(&admin, &100);
    assert_eq!(result, 200);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_critical_function_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);
    client.critical_operation(&admin, &100);
}

#[test]
fn test_critical_function_works_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);
    client.unpause(&admin);

    let result = client.critical_operation(&admin, &100);
    assert_eq!(result, 200);
}

// ===== Event Tests =====

#[test]
fn test_pause_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);

    let events = env.events().all();
    let pause_event = events.iter().find(|e| {
        e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("pausable")
            && e.topics.get(1).unwrap() == &soroban_sdk::symbol_short!("paused")
    });

    assert!(pause_event.is_some());
}

#[test]
fn test_unpause_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);
    client.unpause(&admin);

    let events = env.events().all();
    let unpause_event = events.iter().find(|e| {
        e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("pausable")
            && e.topics.get(1).unwrap() == &soroban_sdk::symbol_short!("unpaused")
    });

    assert!(unpause_event.is_some());
}

// ===== Admin Transfer Tests =====

#[test]
fn test_admin_can_transfer_admin_role() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_new_admin_can_pause() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);
    client.pause(&new_admin);

    assert_eq!(client.is_paused(), true);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_old_admin_cannot_pause_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);
    client.pause(&admin);
}

// ===== Multiple Pause/Unpause Cycles =====

#[test]
fn test_multiple_pause_unpause_cycles() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    for i in 0..5 {
        client.pause(&admin);
        assert_eq!(client.is_paused(), true, "Failed at cycle {}", i);

        client.unpause(&admin);
        assert_eq!(client.is_paused(), false, "Failed at cycle {}", i);
    }
}

// ===== Edge Cases =====

#[test]
fn test_is_paused_returns_false_for_new_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin) = setup_contract(&env);

    assert_eq!(client.is_paused(), false);
}

#[test]
fn test_pause_state_persists_across_calls() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);

    client.pause(&admin);

    // Multiple checks should all return true
    assert_eq!(client.is_paused(), true);
    assert_eq!(client.is_paused(), true);
    assert_eq!(client.is_paused(), true);
}

#[test]
fn test_admin_changed_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin) = setup_contract(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);

    let events = env.events().all();
    let admin_changed_event = events.iter().find(|e| {
        e.topics.get(0).unwrap() == &soroban_sdk::symbol_short!("pausable")
            && e.topics.get(1).unwrap() == &soroban_sdk::symbol_short!("admin_changed")
    });

    assert!(admin_changed_event.is_some());
}
