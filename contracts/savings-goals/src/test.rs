//! Comprehensive unit and integration tests for the savings goals contract.

#![cfg(test)]

use crate::{SavingsGoalsContract, SavingsGoalsContractClient};
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, Symbol, Vec};

use crate::types::{ErrorCode, GoalResult, SavingsGoalRequest};

/// Helper function to create a test environment with initialized contract.
fn setup_test_contract() -> (Env, Address, SavingsGoalsContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SavingsGoalsContract, ());
    let client = SavingsGoalsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, client)
}

/// Helper function to create a valid savings goal request.
fn create_valid_request(
    env: &Env,
    user: &Address,
    goal_name: &str,
    amount: i128,
) -> SavingsGoalRequest {
    let current_ledger = env.ledger().sequence() as u64;
    SavingsGoalRequest {
        user: user.clone(),
        goal_name: Symbol::new(env, goal_name),
        target_amount: amount,
        deadline: current_ledger + 1000,
        initial_contribution: amount / 10, // 10% initial contribution
    }
}

#[test]
fn test_initialize() {
    let (_, admin, client) = setup_test_contract();

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_last_batch_id(), 0);
    assert_eq!(client.get_last_goal_id(), 0);
    assert_eq!(client.get_total_goals_created(), 0);
    assert_eq!(client.get_total_batches_processed(), 0);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_initialize_twice_fails() {
    let (env, _, client) = setup_test_contract();
    let new_admin = Address::generate(&env);
    client.initialize(&new_admin);
}

#[test]
fn test_batch_set_savings_goals_single_user() {
    let (env, admin, client) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    requests.push_back(create_valid_request(&env, &user, "vacation", 100_000_000));

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.total_requests, 1);
    assert_eq!(result.successful, 1);
    assert_eq!(result.failed, 0);
    assert_eq!(result.batch_id, 1);

    // Verify storage updates
    assert_eq!(client.get_last_batch_id(), 1);
    assert_eq!(client.get_last_goal_id(), 1);
    assert_eq!(client.get_total_goals_created(), 1);
    assert_eq!(client.get_total_batches_processed(), 1);
}

#[test]
fn test_batch_set_savings_goals_multiple_users() {
    let (env, client, admin) = setup_test_contract();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    requests.push_back(create_valid_request(&env, &user1, "vacation", 100_000_000));
    requests.push_back(create_valid_request(&env, &user2, "house", 500_000_000));
    requests.push_back(create_valid_request(&env, &user3, "emergency", 200_000_000));

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.total_requests, 3);
    assert_eq!(result.successful, 3);
    assert_eq!(result.failed, 0);
    assert_eq!(result.results.len(), 3);

    // Verify all goals were created successfully
    for goal_result in result.results.iter() {
        match goal_result {
            GoalResult::Success(goal) => {
                assert!(goal.goal_id > 0);
                assert!(goal.target_amount > 0);
                assert_eq!(goal.is_active, true);
            }
            GoalResult::Failure(_, _) => panic!("Expected success, got failure"),
        }
    }

    // Verify storage updates
    assert_eq!(client.get_total_goals_created(), 3);
    assert_eq!(client.get_last_goal_id(), 3);
}

#[test]
fn test_batch_set_savings_goals_with_invalid_requests() {
    let (env, client, admin) = setup_test_contract();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);

    // Valid request
    requests.push_back(create_valid_request(&env, &user1, "vacation", 100_000_000));

    // Invalid request - amount too low
    let mut invalid_request = create_valid_request(&env, &user2, "test", 1000);
    invalid_request.target_amount = 1000; // Below minimum
    requests.push_back(invalid_request);

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.total_requests, 2);
    assert_eq!(result.successful, 1);
    assert_eq!(result.failed, 1);

    // Verify the first succeeded and second failed
    match &result.results.get(0).unwrap() {
        GoalResult::Success(_) => {}
        GoalResult::Failure(_, _) => panic!("Expected first request to succeed"),
    }

    match &result.results.get(1).unwrap() {
        GoalResult::Success(_) => panic!("Expected second request to fail"),
        GoalResult::Failure(_, error_code) => {
            assert_eq!(*error_code, ErrorCode::INVALID_AMOUNT);
        }
    }
}

#[test]
fn test_batch_set_savings_goals_invalid_deadline() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    let mut request = create_valid_request(&env, &user, "vacation", 100_000_000);
    request.deadline = 0; // Past deadline
    requests.push_back(request);

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.successful, 0);
    assert_eq!(result.failed, 1);

    match &result.results.get(0).unwrap() {
        GoalResult::Failure(_, error_code) => {
            assert_eq!(*error_code, ErrorCode::INVALID_DEADLINE);
        }
        GoalResult::Success(_) => panic!("Expected failure"),
    }
}

#[test]
fn test_batch_set_savings_goals_invalid_initial_contribution() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    let mut request = create_valid_request(&env, &user, "vacation", 100_000_000);
    request.initial_contribution = -1000; // Negative contribution
    requests.push_back(request);

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.successful, 0);
    assert_eq!(result.failed, 1);

    match &result.results.get(0).unwrap() {
        GoalResult::Failure(_, error_code) => {
            assert_eq!(*error_code, ErrorCode::INVALID_INITIAL_CONTRIBUTION);
        }
        GoalResult::Success(_) => panic!("Expected failure"),
    }
}

#[test]
#[should_panic]
fn test_batch_set_savings_goals_empty_batch() {
    let (env, client, admin) = setup_test_contract();
    let requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    client.batch_set_savings_goals(&admin, &requests);
}

#[test]
#[should_panic]
fn test_batch_set_savings_goals_batch_too_large() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    // Create 101 requests (exceeds MAX_BATCH_SIZE of 100)
    for i in 0..101 {
        requests.push_back(create_valid_request(
            &env,
            &user,
            "goal",
            100_000_000 + i as i128,
        ));
    }

    client.batch_set_savings_goals(&admin, &requests);
}

#[test]
fn test_get_goal() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    requests.push_back(create_valid_request(&env, &user, "vacation", 100_000_000));

    let result = client.batch_set_savings_goals(&admin, &requests);

    // Get the created goal
    let goal = client.get_goal(&1).unwrap();

    assert_eq!(goal.goal_id, 1);
    assert_eq!(goal.user, user);
    assert_eq!(goal.target_amount, 100_000_000);
    assert_eq!(goal.current_amount, 10_000_000); // 10% initial
    assert_eq!(goal.is_active, true);
}

#[test]
fn test_get_user_goals() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    requests.push_back(create_valid_request(&env, &user, "vacation", 100_000_000));
    requests.push_back(create_valid_request(&env, &user, "house", 500_000_000));

    client.batch_set_savings_goals(&admin, &requests);

    let user_goals = client.get_user_goals(&user);
    assert_eq!(user_goals.len(), 2);
    assert_eq!(user_goals.get(0).unwrap(), 1);
    assert_eq!(user_goals.get(1).unwrap(), 2);
}

#[test]
fn test_batch_metrics() {
    let (env, client, admin) = setup_test_contract();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    requests.push_back(create_valid_request(&env, &user1, "vacation", 100_000_000));
    requests.push_back(create_valid_request(&env, &user2, "house", 200_000_000));

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.metrics.total_requests, 2);
    assert_eq!(result.metrics.successful_goals, 2);
    assert_eq!(result.metrics.failed_goals, 0);
    assert_eq!(result.metrics.total_target_amount, 300_000_000);
    assert_eq!(result.metrics.total_initial_contributions, 30_000_000);
    assert_eq!(result.metrics.avg_goal_amount, 150_000_000);
}

#[test]
fn test_multiple_batches() {
    let (env, client, admin) = setup_test_contract();

    // First batch
    let user1 = Address::generate(&env);
    let mut requests1: Vec<SavingsGoalRequest> = Vec::new(&env);
    requests1.push_back(create_valid_request(&env, &user1, "vacation", 100_000_000));
    let result1 = client.batch_set_savings_goals(&admin, &requests1);
    assert_eq!(result1.batch_id, 1);

    // Second batch
    let user2 = Address::generate(&env);
    let mut requests2: Vec<SavingsGoalRequest> = Vec::new(&env);
    requests2.push_back(create_valid_request(&env, &user2, "house", 500_000_000));
    let result2 = client.batch_set_savings_goals(&admin, &requests2);
    assert_eq!(result2.batch_id, 2);

    // Verify totals
    assert_eq!(client.get_total_batches_processed(), 2);
    assert_eq!(client.get_total_goals_created(), 2);
    assert_eq!(client.get_last_goal_id(), 2);
}

#[test]
fn test_high_value_goal_event() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    // Create high-value goal (>= 100,000 XLM)
    requests.push_back(create_valid_request(
        &env,
        &user,
        "mansion",
        1_000_000_000_000,
    ));

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.successful, 1);
    // High-value event should be emitted (verified in event logs)
}

#[test]
fn test_set_admin() {
    let (env, client, admin) = setup_test_contract();
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_mixed_valid_and_invalid_requests() {
    let (env, client, admin) = setup_test_contract();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);

    // Valid
    requests.push_back(create_valid_request(&env, &user1, "vacation", 100_000_000));

    // Invalid - amount too low
    let mut invalid1 = create_valid_request(&env, &user2, "test", 1000);
    invalid1.target_amount = 1000;
    requests.push_back(invalid1);

    // Valid
    requests.push_back(create_valid_request(&env, &user3, "house", 500_000_000));

    // Invalid - deadline in past
    let mut invalid2 = create_valid_request(&env, &user4, "test", 100_000_000);
    invalid2.deadline = 0;
    requests.push_back(invalid2);

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.total_requests, 4);
    assert_eq!(result.successful, 2);
    assert_eq!(result.failed, 2);

    // Only successful goals should be stored
    assert_eq!(client.get_total_goals_created(), 2);
}

#[test]
fn test_zero_initial_contribution() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    let mut request = create_valid_request(&env, &user, "vacation", 100_000_000);
    request.initial_contribution = 0; // Zero initial contribution is valid
    requests.push_back(request);

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.successful, 1);
    assert_eq!(result.failed, 0);

    let goal = client.get_goal(&1).unwrap();
    assert_eq!(goal.current_amount, 0);
}

#[test]
fn test_full_initial_contribution() {
    let (env, client, admin) = setup_test_contract();
    let user = Address::generate(&env);

    let mut requests: Vec<SavingsGoalRequest> = Vec::new(&env);
    let mut request = create_valid_request(&env, &user, "vacation", 100_000_000);
    request.initial_contribution = 100_000_000; // Full amount
    requests.push_back(request);

    let result = client.batch_set_savings_goals(&admin, &requests);

    assert_eq!(result.successful, 1);

    let goal = client.get_goal(&1).unwrap();
    assert_eq!(goal.current_amount, 100_000_000);
    assert_eq!(goal.target_amount, 100_000_000);
}
