#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, Vec,
};

// Use the escrow contract from the local crate
mod escrow_contract {
    soroban_sdk::contractimport!(file = "target/wasm32-unknown-unknown/release/escrow.wasm");
}
// Actually, it's easier to just use the crate if possible, or register the contract directly if we have the source.
// Since we are in the same workspace, we can just use the crate.

use escrow::{EscrowContract, EscrowContractClient, EscrowStatus, ReversalRequest};

fn setup_test_env() -> (
    Env,
    Address, // admin
    Address, // token_id
    token::StellarAssetClient<'static>,
    EscrowContractClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    // Set initial ledger time
    env.ledger().set_timestamp(1000);

    // Deploy token contract
    let issuer = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(issuer.clone())
        .address();
    let token_admin = token::StellarAssetClient::new(&env, &token_id);

    // Deploy escrow contract
    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &token_id);

    (env, admin, token_id, token_admin, client)
}

#[test]
fn test_escrow_with_arbiter_flow() {
    let (env, admin, _token_id, token_admin, client) = setup_test_env();

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 2000; // deadline at t=2000

    // Mint tokens to depositor
    token_admin.mint(&depositor, &amount);

    // 1. Create escrow with arbiter
    let escrow_id = client.create_escrow(
        &depositor,
        &recipient,
        &Some(arbiter.clone()),
        &amount,
        &deadline,
    );

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.arbiter, Some(arbiter.clone()));
    assert_eq!(escrow.status, EscrowStatus::Active);

    // 2. Arbiter releases funds
    client.release_escrow(&arbiter, &escrow_id);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.status, EscrowStatus::Released);
}

#[test]
fn test_depositor_cannot_reverse_before_deadline() {
    let (env, _admin, _token_id, token_admin, client) = setup_test_env();

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 2000;

    token_admin.mint(&depositor, &amount);
    let escrow_id = client.create_escrow(&depositor, &recipient, &None, &amount, &deadline);

    // Try to reverse at t=1500 (before deadline)
    env.ledger().set_timestamp(1500);

    let mut requests = Vec::new(&env);
    requests.push_back(ReversalRequest { escrow_id });

    // The call should return a failure in result results, not panic, because it's a batch operation
    let result = client.batch_reverse_escrows(&depositor, &requests);
    assert_eq!(result.failed, 1);
    assert_eq!(result.successful, 0);
}

#[test]
fn test_depositor_can_reverse_after_deadline() {
    let (env, _admin, _token_id, token_admin, client) = setup_test_env();

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 2000;

    token_admin.mint(&depositor, &amount);
    let escrow_id = client.create_escrow(&depositor, &recipient, &None, &amount, &deadline);

    // Move to t=2500 (after deadline)
    env.ledger().set_timestamp(2500);

    let mut requests = Vec::new(&env);
    requests.push_back(ReversalRequest { escrow_id });

    let result = client.batch_reverse_escrows(&depositor, &requests);
    assert_eq!(result.successful, 1);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.status, EscrowStatus::Reversed);
}

#[test]
fn test_arbiter_can_reverse_anytime() {
    let (env, _admin, _token_id, token_admin, client) = setup_test_env();

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let arbiter = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 2000;

    token_admin.mint(&depositor, &amount);
    let escrow_id = client.create_escrow(
        &depositor,
        &recipient,
        &Some(arbiter.clone()),
        &amount,
        &deadline,
    );

    // Arbiter reverses at t=1500 (before deadline)
    env.ledger().set_timestamp(1500);

    let mut requests = Vec::new(&env);
    requests.push_back(ReversalRequest { escrow_id });

    let result = client.batch_reverse_escrows(&arbiter, &requests);
    assert_eq!(result.successful, 1);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.status, EscrowStatus::Reversed);
}

#[test]
fn test_admin_can_reverse_anytime() {
    let (env, admin, _token_id, token_admin, client) = setup_test_env();

    let depositor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 2000;

    token_admin.mint(&depositor, &amount);
    let escrow_id = client.create_escrow(&depositor, &recipient, &None, &amount, &deadline);

    // Admin reverses at t=1500 (before deadline)
    env.ledger().set_timestamp(1500);

    let mut requests = Vec::new(&env);
    requests.push_back(ReversalRequest { escrow_id });

    let result = client.batch_reverse_escrows(&admin, &requests);
    assert_eq!(result.successful, 1);

    let escrow = client.get_escrow(&escrow_id).unwrap();
    assert_eq!(escrow.status, EscrowStatus::Reversed);
}
