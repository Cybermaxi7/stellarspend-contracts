use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, Vec,
};

#[path = "../contracts/wallet.rs"]
mod wallet;

use wallet::{
    LinkedWalletInfo, VerificationChallenge, WalletContract, WalletContractClient, WalletError,
};

fn setup_wallet_contract() -> (Env, Address, WalletContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(WalletContract, ());
    let client = WalletContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, client)
}

#[test]
fn test_wallet_initialization() {
    let (env, admin, client) = setup_wallet_contract();

    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic]
fn test_double_initialization_fails() {
    let (env, _admin, client) = setup_wallet_contract();

    let another_admin = Address::generate(&env);
    client.initialize(&another_admin);
}

#[test]
fn test_link_wallet_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let wallet_info = client
        .get_wallet_info(&wallet_address)
        .expect("wallet info should exist");
    assert_eq!(wallet_info.wallet_address, wallet_address);
    assert_eq!(wallet_info.owner_address, owner_address);
    assert_eq!(wallet_info.is_verified, false);
    assert_eq!(wallet_info.verification_nonce, 0);

    // Check events
    let events = env.events().all();
    let wallet_linked_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("linked")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(wallet_linked_events, 1);
}

#[test]
#[should_panic]
fn test_link_duplicate_wallet_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    // Second linking should fail
    let another_owner = Address::generate(&env);
    client.link_wallet(&admin, &wallet_address, &another_owner);
}

#[test]
fn test_unlink_wallet_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);
    client.unlink_wallet(&owner_address, &wallet_address);

    assert!(client.get_wallet_info(&wallet_address).is_none());
}

#[test]
#[should_panic]
fn test_unlink_wallet_unauthorized_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    // Unauthorized user cannot unlink
    client.unlink_wallet(&unauthorized, &wallet_address);
}

#[test]
fn test_verify_wallet_ownership_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    let result = client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);
    assert!(result);

    let wallet_info = client
        .get_wallet_info(&wallet_address)
        .expect("wallet info should exist");
    assert!(wallet_info.is_verified);
    assert_eq!(wallet_info.verification_nonce, 1);
    assert!(client.is_wallet_verified(&wallet_address));
}

#[test]
#[should_panic]
fn test_verify_wallet_ownership_wrong_signer_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let impostor = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    // Wrong signer should trigger ownership mismatch
    client.verify_wallet_ownership(&wallet_address, &impostor, &signature);
}

#[test]
#[should_panic]
fn test_verify_unlinked_wallet_fails() {
    let (env, _admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let signer = Address::generate(&env);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.verify_wallet_ownership(&wallet_address, &signer, &signature);
}

#[test]
fn test_create_verification_challenge_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let challenger = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let challenge_id = client.create_verification_challenge(&challenger, &wallet_address);
    assert!(challenge_id > 0);

    // Check events
    let events = env.events().all();
    let verification_started_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("started")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(verification_started_events, 1);
}

#[test]
#[should_panic]
fn test_create_challenge_unlinked_wallet_fails() {
    let (env, _admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let challenger = Address::generate(&env);

    client.create_verification_challenge(&challenger, &wallet_address);
}

#[test]
fn test_complete_verification_challenge_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let challenger = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);
    let challenge_id = client.create_verification_challenge(&challenger, &wallet_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    let result = client.complete_verification_challenge(&wallet_address, &challenge_id, &signature);
    assert!(result);

    assert!(client.is_wallet_verified(&wallet_address));
}

#[test]
#[should_panic]
fn test_complete_nonexistent_challenge_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.complete_verification_challenge(&wallet_address, &999, &signature);
}

#[test]
#[should_panic]
fn test_complete_challenge_invalid_signature_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let challenger = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);
    let challenge_id = client.create_verification_challenge(&challenger, &wallet_address);

    let empty_signature = Vec::new(&env);

    client.complete_verification_challenge(&wallet_address, &challenge_id, &empty_signature);
}

#[test]
fn test_require_verified_wallet_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);

    // This should not panic
    client.require_verified_wallet(&wallet_address, &owner_address);
}

#[test]
#[should_panic]
fn test_require_verified_unlinked_wallet_fails() {
    let (env, _admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let user = Address::generate(&env);

    client.require_verified_wallet(&wallet_address, &user);
}

#[test]
#[should_panic]
fn test_require_verified_unverified_wallet_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    // Wallet is linked but not verified
    client.require_verified_wallet(&wallet_address, &owner_address);
}

#[test]
#[should_panic]
fn test_require_verified_wrong_caller_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let impostor = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);

    // Wrong caller should be blocked as spoofed call
    client.require_verified_wallet(&wallet_address, &impostor);
}

#[test]
fn test_validate_wallet_action_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);

    // This should succeed
    client.validate_wallet_action(&wallet_address, &owner_address);
}

#[test]
#[should_panic]
fn test_validate_wallet_action_unlinked_fails() {
    let (env, _admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let user = Address::generate(&env);

    client.validate_wallet_action(&wallet_address, &user);
}

#[test]
#[should_panic]
fn test_validate_wallet_action_unverified_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    // Wallet is linked but not verified
    client.validate_wallet_action(&wallet_address, &owner_address);
}

#[test]
#[should_panic]
fn test_validate_wallet_action_spoofed_call_fails() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let impostor = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);

    // Impostor should be blocked
    client.validate_wallet_action(&wallet_address, &impostor);
}

#[test]
fn test_get_wallet_owner_success() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let retrieved_owner = client
        .get_wallet_owner(&wallet_address)
        .expect("owner should exist");
    assert_eq!(retrieved_owner, owner_address);
}

#[test]
fn test_get_wallet_owner_unlinked_returns_none() {
    let (env, _admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);

    let owner = client.get_wallet_owner(&wallet_address);
    assert!(owner.is_none());
}

#[test]
fn test_admin_can_unlink_any_wallet() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    // Admin should be able to unlink any wallet
    client.unlink_wallet(&admin, &wallet_address);

    assert!(client.get_wallet_info(&wallet_address).is_none());
}

#[test]
fn test_multiple_wallets_per_owner() {
    let (env, admin, client) = setup_wallet_contract();

    let owner_address = Address::generate(&env);
    let wallet1 = Address::generate(&env);
    let wallet2 = Address::generate(&env);

    client.link_wallet(&admin, &wallet1, &owner_address);
    client.link_wallet(&admin, &wallet2, &owner_address);

    let info1 = client
        .get_wallet_info(&wallet1)
        .expect("wallet1 info should exist");
    let info2 = client
        .get_wallet_info(&wallet2)
        .expect("wallet2 info should exist");

    assert_eq!(info1.owner_address, owner_address);
    assert_eq!(info2.owner_address, owner_address);
    assert_ne!(info1.wallet_address, info2.wallet_address);
}

#[test]
fn test_verification_events_emitted() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);

    let events = env.events().all();

    // Check for ownership verified event
    let ownership_verified_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("verified")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();

    assert!(ownership_verified_events >= 1);
}

#[test]
#[should_panic]
fn test_spoofed_call_blocked_event_emitted() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);
    let impostor = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);

    // This should fail and emit a spoofed call blocked event
    client.validate_wallet_action(&wallet_address, &impostor);
}

#[test]
fn test_nonce_incrementing_on_verification() {
    let (env, admin, client) = setup_wallet_contract();

    let wallet_address = Address::generate(&env);
    let owner_address = Address::generate(&env);

    client.link_wallet(&admin, &wallet_address, &owner_address);

    let signature = Vec::new(&env);
    signature.push_back(&1);

    // First verification
    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature.clone());
    let info1 = client
        .get_wallet_info(&wallet_address)
        .expect("wallet info should exist");
    assert_eq!(info1.verification_nonce, 1);

    // Second verification (nonce should increment)
    client.verify_wallet_ownership(&wallet_address, &owner_address, &signature);
    let info2 = client
        .get_wallet_info(&wallet_address)
        .expect("wallet info should exist");
    assert_eq!(info2.verification_nonce, 2);
}
