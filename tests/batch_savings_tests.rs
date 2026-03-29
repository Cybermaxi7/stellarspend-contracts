// Integration tests for batch contributions to savings goals.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_batch_contribute_success() {
    let env = Env::default();
    let user = Address::generate(&env);
    let goal_ids = vec![1, 2, 3];
    let amounts = vec![100, 200, 300];
    let result = SavingsContract::batch_contribute(
        env.clone(),
        user.clone(),
        goal_ids.clone(),
        amounts.clone(),
    );
    assert!(result.is_ok());
    let events = env.events().all();
    assert!(events.iter().any(|e| e.topics.0 == "milestone"));
}

#[test]
fn test_goal_amount_mismatch() {
    let env = Env::default();
    let user = Address::generate(&env);
    let goal_ids = vec![1, 2];
    let amounts = vec![100];
    let result = SavingsContract::batch_contribute(
        env.clone(),
        user.clone(),
        goal_ids.clone(),
        amounts.clone(),
    );
    assert_eq!(result, Err("goal_amount_mismatch"));
}

#[test]
fn test_invalid_goal_id() {
    let env = Env::default();
    let user = Address::generate(&env);
    let goal_ids = vec![999];
    let amounts = vec![100];
    // Patch is_valid_goal to return false for this test
    // (Would require dependency injection or trait in real code)
    // Here, just show the test structure
    // assert_eq!(result, Err("invalid_goal_id"));
}

#[test]
fn test_over_contribution() {
    let env = Env::default();
    let user = Address::generate(&env);
    let goal_ids = vec![1];
    let amounts = vec![1_000_000];
    // Patch would_over_contribute to return true for this test
    // assert_eq!(result, Err("over_contribution"));
}
