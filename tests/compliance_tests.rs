#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, String, U256,
};

#[path = "../contracts/compliance.rs"]
mod compliance;

use compliance::{ComplianceContract, ComplianceContractClient};

fn setup_compliance_contract() -> (Env, Address, ComplianceContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ComplianceContract, ());
    let client = ComplianceContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, client)
}

#[test]
fn test_compliance_initialization() {
    let (_env, _admin, client) = setup_compliance_contract();
    assert_eq!(client.get_flagged_count(), 0);
}

#[test]
fn test_set_limit_success() {
    let (env, admin, client) = setup_compliance_contract();
    let limit_name = String::from_str(&env, "max_transfer_amount");
    
    client.set_limit(&admin, &limit_name, &1000);
    
    // Check events
    let events = env.events().all();
    let limit_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("limit_up") == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(limit_events, 1);
}

#[test]
fn test_check_and_flag_transaction() {
    let (env, admin, client) = setup_compliance_contract();
    let user = Address::generate(&env);
    let limit_name = String::from_str(&env, "max_transfer_amount");
    
    // Set limit to 500
    client.set_limit(&admin, &limit_name, &500);
    
    let tx_id_1 = U256::from_u32(&env, 123);
    let amount_1 = 400; // Not flagged
    
    let flagged_1 = client.check_transaction(&tx_id_1, &user, &amount_1);
    assert!(!flagged_1);
    assert!(!client.is_flagged(&tx_id_1));
    assert_eq!(client.get_flagged_count(), 0);
    
    let tx_id_2 = U256::from_u32(&env, 456);
    let amount_2 = 600; // Flagged
    
    let flagged_2 = client.check_transaction(&tx_id_2, &user, &amount_2);
    assert!(flagged_2);
    assert!(client.is_flagged(&tx_id_2));
    assert_eq!(client.get_flagged_count(), 1);
    
    // Check for flagged event
    let events = env.events().all();
    let flagged_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("flagged") == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(flagged_events, 1);
}

#[test]
#[should_panic]
fn test_unauthorized_limit_update() {
    let (env, _admin, client) = setup_compliance_contract();
    let unauthorized = Address::generate(&env);
    let limit_name = String::from_str(&env, "max_transfer_amount");
    
    client.set_limit(&unauthorized, &limit_name, &1000);
}

#[test]
#[should_panic]
fn test_invalid_negative_limit() {
    let (env, admin, client) = setup_compliance_contract();
    let limit_name = String::from_str(&env, "max_transfer_amount");
    
    client.set_limit(&admin, &limit_name, &-1);
}
