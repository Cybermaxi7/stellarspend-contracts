//! Integration tests for the Batch Wallet Creation Contract.

#![cfg(test)]

use crate::{
    BatchWalletContract, BatchWalletContractClient, WalletCreateRequest, WalletCreateResult,
};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    Address, Env, Vec,
};

/// Creates a test environment with the contract deployed and initialized.
fn setup_test_env() -> (Env, Address, BatchWalletContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.sequence_number = 12345;
    });

    // Deploy batch wallet contract
    let contract_id = env.register(BatchWalletContract, ());
    let client = BatchWalletContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, client)
}

/// Helper to create a wallet creation request.
fn create_wallet_request(_env: &Env, owner: Address) -> WalletCreateRequest {
    WalletCreateRequest { owner }
}

// Initialization Tests

#[test]
fn test_initialize_contract() {
    let (_env, admin, client) = setup_test_env();

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_total_batches(), 0);
    assert_eq!(client.get_total_wallets_created(), 0);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let (env, admin, client) = setup_test_env();

    let new_admin = Address::generate(&env);
    client.initialize(&new_admin);
}

// Batch Wallet Creation Tests

#[test]
fn test_batch_create_wallets_single() {
    let (env, admin, client) = setup_test_env();

    let owner = Address::generate(&env);

    let mut requests: Vec<WalletCreateRequest> = Vec::new(&env);
    requests.push_back(create_wallet_request(&env, owner.clone()));

    let result = client.batch_create_wallets(&admin, &requests);

    assert_eq!(result.total_requests, 1);
    assert_eq!(result.successful, 1);
    assert_eq!(result.failed, 0);
    assert_eq!(result.results.len(), 1);

    // Check wallet was created
    let wallet = client.get_wallet(&owner).unwrap();
    assert_eq!(wallet.owner, owner);
    assert_eq!(wallet.id, 1);
}

#[test]
fn test_batch_create_wallets_multiple() {
    let (env, admin, client) = setup_test_env();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let owner3 = Address::generate(&env);

    let mut requests: Vec<WalletCreateRequest> = Vec::new(&env);
    requests.push_back(create_wallet_request(&env, owner1.clone()));
    requests.push_back(create_wallet_request(&env, owner2.clone()));
    requests.push_back(create_wallet_request(&env, owner3.clone()));

    let result = client.batch_create_wallets(&admin, &requests);

    assert_eq!(result.total_requests, 3);
    assert_eq!(result.successful, 3);
    assert_eq!(result.failed, 0);

    // Check wallets were created with sequential IDs
    let wallet1 = client.get_wallet(&owner1).unwrap();
    assert_eq!(wallet1.id, 1);
    let wallet2 = client.get_wallet(&owner2).unwrap();
    assert_eq!(wallet2.id, 2);
    let wallet3 = client.get_wallet(&owner3).unwrap();
    assert_eq!(wallet3.id, 3);
}

#[test]
fn test_batch_create_wallets_partial_failures() {
    let (env, admin, client) = setup_test_env();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let owner3 = Address::generate(&env);

    // First batch: create wallets for owner1 and owner2
    let mut requests1: Vec<WalletCreateRequest> = Vec::new(&env);
    requests1.push_back(create_wallet_request(&env, owner1.clone()));
    requests1.push_back(create_wallet_request(&env, owner2.clone()));
    client.batch_create_wallets(&admin, &requests1);

    // Second batch: try to create for owner1 (duplicate), owner2 (duplicate), owner3 (new)
    let mut requests2: Vec<WalletCreateRequest> = Vec::new(&env);
    requests2.push_back(create_wallet_request(&env, owner1.clone())); // Duplicate
    requests2.push_back(create_wallet_request(&env, owner2.clone())); // Duplicate
    requests2.push_back(create_wallet_request(&env, owner3.clone())); // New

    let result = client.batch_create_wallets(&admin, &requests2);

    assert_eq!(result.total_requests, 3);
    assert_eq!(result.successful, 1);
    assert_eq!(result.failed, 2);

    // Check results
    match result.results.get(0).unwrap() {
        WalletCreateResult::Failure(addr, error_code) => {
            assert_eq!(addr, owner1);
            assert_eq!(error_code, 1); // Already exists
        }
        _ => panic!("Expected failure for duplicate"),
    }
    match result.results.get(1).unwrap() {
        WalletCreateResult::Failure(addr, error_code) => {
            assert_eq!(addr, owner2);
            assert_eq!(error_code, 1); // Already exists
        }
        _ => panic!("Expected failure for duplicate"),
    }
    match result.results.get(2).unwrap() {
        WalletCreateResult::Success(addr) => {
            assert_eq!(addr, owner3);
        }
        _ => panic!("Expected success for new wallet"),
    }

    // Check wallet3 was created
    let wallet3 = client.get_wallet(&owner3).unwrap();
    assert_eq!(wallet3.id, 3); // IDs continue from previous batch
}

#[test]
fn test_batch_create_wallets_events_emitted() {
    let (env, admin, client) = setup_test_env();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);

    let mut requests: Vec<WalletCreateRequest> = Vec::new(&env);
    requests.push_back(create_wallet_request(&env, owner1.clone()));
    requests.push_back(create_wallet_request(&env, owner2.clone()));

    client.batch_create_wallets(&admin, &requests);

    let events = env.events().all();
    // Should have: batch_started, wallet_created (2), batch_completed
    assert!(events.len() >= 4);
}

#[test]
fn test_batch_create_wallets_accumulates_stats() {
    let (env, admin, client) = setup_test_env();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);

    let mut requests1: Vec<WalletCreateRequest> = Vec::new(&env);
    requests1.push_back(create_wallet_request(&env, owner1.clone()));

    let mut requests2: Vec<WalletCreateRequest> = Vec::new(&env);
    requests2.push_back(create_wallet_request(&env, owner2.clone()));

    assert_eq!(client.get_total_batches(), 0);
    assert_eq!(client.get_total_wallets_created(), 0);

    client.batch_create_wallets(&admin, &requests1);
    assert_eq!(client.get_total_batches(), 1);
    assert_eq!(client.get_total_wallets_created(), 1);

    client.batch_create_wallets(&admin, &requests2);
    assert_eq!(client.get_total_batches(), 2);
    assert_eq!(client.get_total_wallets_created(), 2);
}

#[test]
#[should_panic]
fn test_batch_create_wallets_empty_batch() {
    let (env, admin, client) = setup_test_env();

    let requests: Vec<WalletCreateRequest> = Vec::new(&env);
    client.batch_create_wallets(&admin, &requests);
}

#[test]
#[should_panic]
fn test_batch_create_wallets_unauthorized() {
    let (env, admin, client) = setup_test_env();

    let unauthorized = Address::generate(&env);
    let owner = Address::generate(&env);

    let mut requests: Vec<WalletCreateRequest> = Vec::new(&env);
    requests.push_back(create_wallet_request(&env, owner));

    // This should panic due to unauthorized access
    client.batch_create_wallets(&unauthorized, &requests);
}

#[test]
fn test_batch_create_wallets_large_batch() {
    let (env, admin, client) = setup_test_env();

    // Create a batch with 50 owners
    let mut requests: Vec<WalletCreateRequest> = Vec::new(&env);
    let mut owners: Vec<Address> = Vec::new(&env);

    for _i in 0..50 {
        let owner = Address::generate(&env);
        owners.push_back(owner.clone());
        requests.push_back(create_wallet_request(&env, owner));
    }

    let result = client.batch_create_wallets(&admin, &requests);

    assert_eq!(result.total_requests, 50);
    assert_eq!(result.successful, 50);
    assert_eq!(result.failed, 0);

    // Check some wallets
    let wallet1 = client.get_wallet(&owners.get(0).unwrap()).unwrap();
    assert_eq!(wallet1.id, 1);
    let wallet50 = client.get_wallet(&owners.get(49).unwrap()).unwrap();
    assert_eq!(wallet50.id, 50);
}

// Admin Tests

#[test]
fn test_set_admin() {
    let (env, admin, client) = setup_test_env();

    let new_admin = Address::generate(&env);
    client.set_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

// Multiple Simultaneous Batch Creations

#[test]
fn test_multiple_simultaneous_batch_creations() {
    let (env, admin, client) = setup_test_env();

    // First batch: 3 owners
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let owner3 = Address::generate(&env);

    let mut batch1: Vec<WalletCreateRequest> = Vec::new(&env);
    batch1.push_back(create_wallet_request(&env, owner1.clone()));
    batch1.push_back(create_wallet_request(&env, owner2.clone()));
    batch1.push_back(create_wallet_request(&env, owner3.clone()));

    let result1 = client.batch_create_wallets(&admin, &batch1);
    assert_eq!(result1.successful, 3);

    // Second batch: 2 owners (one new, one duplicate)
    let owner4 = Address::generate(&env);

    let mut batch2: Vec<WalletCreateRequest> = Vec::new(&env);
    batch2.push_back(create_wallet_request(&env, owner1.clone())); // Duplicate
    batch2.push_back(create_wallet_request(&env, owner4.clone())); // New

    let result2 = client.batch_create_wallets(&admin, &batch2);
    assert_eq!(result2.successful, 1);
    assert_eq!(result2.failed, 1);

    // Verify contract stats
    assert_eq!(client.get_total_batches(), 2);
    assert_eq!(client.get_total_wallets_created(), 4); // 3 + 1
}