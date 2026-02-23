// Edge case tests for wallet rate limiting.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_within_limit() {
    let env = Env::default();
    let wallet = Address::generate(&env);
    for _ in 0..5 {
        let result = RateLimitContract::check_and_record(env.clone(), wallet.clone());
        assert!(result.is_ok());
    }
}

#[test]
fn test_exceed_limit() {
    let env = Env::default();
    let wallet = Address::generate(&env);
    for _ in 0..5 {
        let result = RateLimitContract::check_and_record(env.clone(), wallet.clone());
        assert!(result.is_ok());
    }
    let result = RateLimitContract::check_and_record(env.clone(), wallet.clone());
    assert_eq!(result, Err("rate_limit_exceeded"));
    let events = env.events().all();
    assert!(events.iter().any(|e| e.topics.0 == "rate_limit"));
}

#[test]
fn test_new_window_resets_limit() {
    let env = Env::default();
    let wallet = Address::generate(&env);
    for _ in 0..5 {
        let result = RateLimitContract::check_and_record(env.clone(), wallet.clone());
        assert!(result.is_ok());
    }
    // Simulate new window
    env.ledger().with_mut(|li| li.timestamp += 3600);
    let result = RateLimitContract::check_and_record(env.clone(), wallet.clone());
    assert!(result.is_ok());
}

#[test]
fn test_multiple_wallets_independent_limits() {
    let env = Env::default();
    let wallet1 = Address::generate(&env);
    let wallet2 = Address::generate(&env);
    for _ in 0..5 {
        assert!(RateLimitContract::check_and_record(env.clone(), wallet1.clone()).is_ok());
        assert!(RateLimitContract::check_and_record(env.clone(), wallet2.clone()).is_ok());
    }
    assert_eq!(RateLimitContract::check_and_record(env.clone(), wallet1.clone()), Err("rate_limit_exceeded"));
    assert_eq!(RateLimitContract::check_and_record(env.clone(), wallet2.clone()), Err("rate_limit_exceeded"));
}
