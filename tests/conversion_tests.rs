// Conversion tests for Stellar asset conversion.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_valid_conversion() {
    let env = Env::default();
    let user = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 100;
    let result = ConversionContract::convert_assets(env.clone(), user.clone(), from.clone(), to.clone(), amount);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 200); // 1:2 mock rate
}

#[test]
fn test_invalid_pair() {
    let env = Env::default();
    let user = Address::generate(&env);
    let token = Address::generate(&env);
    let result = ConversionContract::convert_assets(env.clone(), user.clone(), token.clone(), token.clone(), 100);
    assert_eq!(result, Err("same_token_conversion_not_allowed"));
}

#[test]
fn test_invalid_amount() {
    let env = Env::default();
    let user = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let result = ConversionContract::convert_assets(env.clone(), user.clone(), from.clone(), to.clone(), 0);
    assert_eq!(result, Err("invalid_amount"));
}

#[test]
fn test_event_emission() {
    let env = Env::default();
    let user = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 50;
    let _ = ConversionContract::convert_assets(env.clone(), user.clone(), from.clone(), to.clone(), amount);
    let events = env.events().all();
    assert!(events.iter().any(|e| e.topics.0 == "conversion"));
}
