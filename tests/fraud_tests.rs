// Tests for fraud detection logic.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_flag_abnormal_transaction() {
    let env = Env::default();
    let user = Address::generate(&env);
    let amount = 20_000; // Above threshold
    let flagged = FraudContract::check_transaction(env.clone(), user.clone(), amount);
    assert!(flagged);
    let events = env.events().all();
    assert!(events.iter().any(|e| e.topics.0 == "fraud_alert"));
    // Check event details
    let found = events.iter().any(|e| {
        e.topics.0 == "fraud_alert" && e.data.0 == amount && e.data.1.contains(&"abnormal_size")
    });
    assert!(found);
}

#[test]
fn test_normal_transaction() {
    let env = Env::default();
    let user = Address::generate(&env);
    let amount = 5_000; // Below threshold
    let flagged = FraudContract::check_transaction(env.clone(), user.clone(), amount);
    assert!(!flagged);
    let events = env.events().all();
    assert!(events.iter().all(|e| e.topics.0 != "fraud_alert"));
}

#[test]
fn test_daily_limit_detection() {
    let env = Env::default();
    let user = Address::generate(&env);
    // Simulate multiple transactions to exceed daily limit
    let amounts = vec![50_000, 60_000];
    let mut flagged = false;
    for amt in amounts {
        flagged = FraudContract::check_transaction(env.clone(), user.clone(), amt);
    }
    assert!(flagged);
    let events = env.events().all();
    let found = events.iter().any(|e| {
        e.topics.0 == "fraud_alert" && e.data.1.contains(&"daily_limit")
    });
    assert!(found);
}
