//! Integration tests for the Budget Recommendations Contract.

#![cfg(test)]

use crate::{
    BudgetRecommendationsContract, BudgetRecommendationsContractClient, RecommendationResult,
    UserProfile,
};
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, Symbol, Vec,
};

/// Creates a test environment with the contract deployed and initialized.
fn setup_test_env() -> (Env, Address, BudgetRecommendationsContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BudgetRecommendationsContract, ());
    let client = BudgetRecommendationsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, client)
}

/// Helper to create a test user profile.
fn create_user_profile(
    env: &Env,
    user_id: u64,
    income: i128,
    expenses: i128,
    savings: i128,
    risk_tolerance: u8,
) -> UserProfile {
    UserProfile {
        user_id,
        address: Address::generate(env),
        monthly_income: income,
        monthly_expenses: expenses,
        savings_balance: savings,
        spending_categories: Symbol::new(env, "food,transport,utilities"),
        risk_tolerance,
    }
}

// ============================================================================
// Initialization Tests
// ============================================================================

#[test]
fn test_initialize_contract() {
    let (_env, admin, client) = setup_test_env();

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_last_batch_id(), 0);
    assert_eq!(client.get_total_users_processed(), 0);
    assert_eq!(client.get_total_recommendations_generated(), 0);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let (env, _admin, client) = setup_test_env();

    let new_admin = Address::generate(&env);
    client.initialize(&new_admin);
}

// ============================================================================
// Batch Recommendation Tests
// ============================================================================

#[test]
fn test_generate_batch_recommendations_single_user() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 3));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    assert_eq!(result.batch_id, 1);
    assert_eq!(result.total_users, 1);
    assert_eq!(result.successful, 1);
    assert_eq!(result.failed, 0);
    assert_eq!(result.metrics.user_count, 1);
    assert_eq!(result.metrics.successful_recommendations, 1);
    assert_eq!(result.metrics.failed_recommendations, 0);
}

#[test]
fn test_generate_batch_recommendations_multiple_users() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 3));
    profiles.push_back(create_user_profile(&env, 2, 200000, 100000, 50000, 2));
    profiles.push_back(create_user_profile(&env, 3, 150000, 80000, 20000, 4));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    assert_eq!(result.total_users, 3);
    assert_eq!(result.successful, 3);
    assert_eq!(result.failed, 0);
    assert_eq!(result.metrics.user_count, 3);
    assert!(result.metrics.total_recommended_budget > 0);
    assert!(result.metrics.total_recommended_savings > 0);
}

#[test]
fn test_generate_batch_recommendations_different_risk_tolerances() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    // Conservative user
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 1));
    // Moderate user
    profiles.push_back(create_user_profile(&env, 2, 100000, 50000, 10000, 3));
    // Aggressive user
    profiles.push_back(create_user_profile(&env, 3, 100000, 50000, 10000, 5));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    assert_eq!(result.successful, 3);
    
    // Check that recommendations have different types based on risk tolerance
    let rec1 = match result.results.get(0).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };
    let rec2 = match result.results.get(1).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };
    let rec3 = match result.results.get(2).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };

    assert_eq!(rec1.recommendation_type, Symbol::new(&env, "conservative"));
    assert_eq!(rec2.recommendation_type, Symbol::new(&env, "moderate"));
    assert_eq!(rec3.recommendation_type, Symbol::new(&env, "aggressive"));

    // Conservative should recommend higher savings
    assert!(rec1.recommended_savings >= rec2.recommended_savings);
    assert!(rec2.recommended_savings >= rec3.recommended_savings);
}

#[test]
fn test_generate_batch_recommendations_events_emitted() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 3));
    profiles.push_back(create_user_profile(&env, 2, 200000, 100000, 50000, 2));

    client.generate_batch_recommendations(&admin, &profiles);

    let events = env.events().all();
    // Should have: batch_started, recommendation_generated (2), batch_completed
    assert!(events.len() >= 4);
}

#[test]
fn test_generate_batch_recommendations_accumulates_stats() {
    let (env, admin, client) = setup_test_env();

    let mut profiles1: Vec<UserProfile> = Vec::new(&env);
    profiles1.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 3));

    let mut profiles2: Vec<UserProfile> = Vec::new(&env);
    profiles2.push_back(create_user_profile(&env, 2, 200000, 100000, 50000, 2));

    assert_eq!(client.get_last_batch_id(), 0);
    assert_eq!(client.get_total_users_processed(), 0);
    assert_eq!(client.get_total_recommendations_generated(), 0);

    client.generate_batch_recommendations(&admin, &profiles1);
    assert_eq!(client.get_last_batch_id(), 1);
    assert_eq!(client.get_total_users_processed(), 1);
    assert_eq!(client.get_total_recommendations_generated(), 1);

    client.generate_batch_recommendations(&admin, &profiles2);
    assert_eq!(client.get_last_batch_id(), 2);
    assert_eq!(client.get_total_users_processed(), 2);
    assert_eq!(client.get_total_recommendations_generated(), 2);
}

#[test]
fn test_generate_batch_recommendations_stores_results() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 3));

    let result = client.generate_batch_recommendations(&admin, &profiles);
    let batch_id = result.batch_id;

    let stored = client.get_batch_recommendations(&batch_id);
    assert!(stored.is_some());
    let stored_results = stored.unwrap();
    assert_eq!(stored_results.len(), 1);
}

#[test]
fn test_generate_batch_recommendations_high_confidence() {
    let (env, admin, client) = setup_test_env();

    // User with good financial profile (high income, positive savings)
    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 200000, 80000, 50000, 3));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    let rec = match result.results.get(0).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };

    // Should have high confidence due to good financial profile
    assert!(rec.confidence_score >= 80);
}

#[test]
fn test_generate_batch_recommendations_low_income_scenario() {
    let (env, admin, client) = setup_test_env();

    // User with expenses exceeding income
    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 50000, 80000, 0, 3));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    assert_eq!(result.successful, 1);
    let rec = match result.results.get(0).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };

    // Should have notes about expenses exceeding income
    assert_eq!(
        rec.notes,
        Symbol::new(&env, "expenses_exceed_income_review_needed")
    );
}

#[test]
fn test_generate_batch_recommendations_emergency_fund_target() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    // Conservative user should have higher emergency fund target
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 0, 1));
    // Aggressive user should have lower emergency fund target
    profiles.push_back(create_user_profile(&env, 2, 100000, 50000, 0, 5));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    let rec1 = match result.results.get(0).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };
    let rec2 = match result.results.get(1).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };

    // Conservative should have higher emergency fund (6 months vs 3 months)
    assert!(rec1.emergency_fund_target > rec2.emergency_fund_target);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
#[should_panic]
fn test_generate_batch_recommendations_empty_batch() {
    let (env, admin, client) = setup_test_env();

    let profiles: Vec<UserProfile> = Vec::new(&env);
    client.generate_batch_recommendations(&admin, &profiles);
}

#[test]
#[should_panic]
fn test_generate_batch_recommendations_unauthorized() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 3));

    let unauthorized = Address::generate(&env);
    client.generate_batch_recommendations(&unauthorized, &profiles);
}

#[test]
fn test_generate_batch_recommendations_large_batch() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    // Create batch of 50 users (within limit)
    for i in 1..=50 {
        profiles.push_back(create_user_profile(&env, i, 100000, 50000, 10000, 3));
    }

    let result = client.generate_batch_recommendations(&admin, &profiles);

    assert_eq!(result.total_users, 50);
    assert_eq!(result.successful, 50);
}

// ============================================================================
// Simulation Tests
// ============================================================================

#[test]
fn test_simulate_recommendation() {
    let (env, _admin, client) = setup_test_env();

    let profile = create_user_profile(&env, 1, 100000, 50000, 10000, 3);

    let recommendation = client.simulate_recommendation(&profile).unwrap();

    assert_eq!(recommendation.user_id, 1);
    assert!(recommendation.recommended_budget > 0);
    assert!(recommendation.recommended_savings >= 0);
    assert!(recommendation.confidence_score > 0);
}

#[test]
fn test_simulate_recommendation_no_storage() {
    let (env, admin, client) = setup_test_env();

    let profile = create_user_profile(&env, 1, 100000, 50000, 10000, 3);

    // Simulate should not increment batch ID
    let batch_id_before = client.get_last_batch_id();
    client.simulate_recommendation(&profile).unwrap();
    let batch_id_after = client.get_last_batch_id();

    assert_eq!(batch_id_before, batch_id_after);
}

// ============================================================================
// Admin Tests
// ============================================================================

#[test]
fn test_set_admin() {
    let (env, admin, client) = setup_test_env();

    let new_admin = Address::generate(&env);
    client.set_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic]
fn test_set_admin_unauthorized() {
    let (env, admin, client) = setup_test_env();

    let unauthorized = Address::generate(&env);
    let new_admin = Address::generate(&env);
    client.set_admin(&unauthorized, &new_admin);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_generate_batch_recommendations_zero_expenses() {
    let (env, admin, client) = setup_test_env();

    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 100000, 0, 10000, 3));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    assert_eq!(result.successful, 1);
    let rec = match result.results.get(0).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };

    // Should still generate valid recommendations
    assert!(rec.recommended_budget >= 0);
}

#[test]
fn test_generate_batch_recommendations_high_savings() {
    let (env, admin, client) = setup_test_env();

    // User with high savings balance
    let mut profiles: Vec<UserProfile> = Vec::new(&env);
    profiles.push_back(create_user_profile(&env, 1, 100000, 50000, 500000, 3));

    let result = client.generate_batch_recommendations(&admin, &profiles);

    assert_eq!(result.successful, 1);
    let rec = match result.results.get(0).unwrap() {
        RecommendationResult::Success(rec) => rec,
        _ => panic!("Expected success"),
    };

    // Should have higher confidence due to existing savings
    assert!(rec.confidence_score >= 85);
}

#[test]
fn test_generate_batch_recommendations_multiple_simultaneous_batches() {
    let (env, admin, client) = setup_test_env();

    let mut profiles1: Vec<UserProfile> = Vec::new(&env);
    profiles1.push_back(create_user_profile(&env, 1, 100000, 50000, 10000, 3));

    let mut profiles2: Vec<UserProfile> = Vec::new(&env);
    profiles2.push_back(create_user_profile(&env, 2, 200000, 100000, 50000, 2));

    let result1 = client.generate_batch_recommendations(&admin, &profiles1);
    let result2 = client.generate_batch_recommendations(&admin, &profiles2);

    assert_eq!(result1.batch_id, 1);
    assert_eq!(result2.batch_id, 2);
    assert_eq!(client.get_total_users_processed(), 2);
}
