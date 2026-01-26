//! # Savings Goals Contract
//!
//! A Soroban smart contract for managing batch savings goal creation
//! for multiple users simultaneously.
//!
//! ## Features
//!
//! - **Batch Processing**: Efficiently create savings goals for multiple users in a single call
//! - **Comprehensive Validation**: Validates goal amounts, deadlines, and initial contributions
//! - **Event Emission**: Emits events for goal creation and batch processing
//! - **Error Handling**: Gracefully handles invalid inputs with detailed error codes
//! - **Optimized Storage**: Minimizes storage writes by batching operations
//!
//! ## Optimization Strategies
//!
//! - Single-pass processing for O(n) complexity
//! - Minimized storage operations (batch writes at the end)
//! - Efficient data structures
//! - Batched event emissions

#![no_std]

mod types;
mod validation;

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Vec};

pub use crate::types::{
    BatchGoalMetrics, BatchGoalResult, DataKey, ErrorCode, GoalEvents, GoalResult, SavingsGoal,
    SavingsGoalRequest, MAX_BATCH_SIZE,
};
use crate::validation::validate_goal_request;

/// Error codes for the savings goals contract.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SavingsGoalError {
    /// Contract not initialized
    NotInitialized = 1,
    /// Caller is not authorized
    Unauthorized = 2,
    /// Invalid batch data
    InvalidBatch = 3,
    /// Batch is empty
    EmptyBatch = 4,
    /// Batch exceeds maximum size
    BatchTooLarge = 5,
}

impl From<SavingsGoalError> for soroban_sdk::Error {
    fn from(e: SavingsGoalError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

#[contract]
pub struct SavingsGoalsContract;

#[contractimpl]
impl SavingsGoalsContract {
    /// Initializes the contract with an admin address.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address that can manage the contract
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::LastBatchId, &0u64);
        env.storage().instance().set(&DataKey::LastGoalId, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::TotalGoalsCreated, &0u64);
        env.storage()
            .instance()
            .set(&DataKey::TotalBatchesProcessed, &0u64);
    }

    /// Creates savings goals for multiple users in a batch.
    ///
    /// This is the main entry point for batch goal creation. It validates all requests,
    /// creates goals, emits events, and updates storage efficiently.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `caller` - The address calling this function (must be admin)
    /// * `requests` - Vector of savings goal requests
    ///
    /// # Returns
    /// * `BatchGoalResult` - Result containing created goals and metrics
    ///
    /// # Events Emitted
    /// * `batch_started` - When processing begins
    /// * `goal_created` - For each successful goal creation
    /// * `goal_creation_failed` - For each failed goal creation
    /// * `high_value_goal` - For goals with high target amounts
    /// * `batch_completed` - When processing completes
    ///
    /// # Errors
    /// * `EmptyBatch` - If no requests provided
    /// * `BatchTooLarge` - If batch exceeds maximum size
    /// * `Unauthorized` - If caller is not admin
    pub fn batch_set_savings_goals(
        env: Env,
        caller: Address,
        requests: Vec<SavingsGoalRequest>,
    ) -> BatchGoalResult {
        // Verify authorization
        caller.require_auth();
        Self::require_admin(&env, &caller);

        // Validate batch size
        let request_count = requests.len();
        if request_count == 0 {
            panic_with_error!(&env, SavingsGoalError::EmptyBatch);
        }
        if request_count > MAX_BATCH_SIZE {
            panic_with_error!(&env, SavingsGoalError::BatchTooLarge);
        }

        // Get batch ID and increment
        let batch_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LastBatchId)
            .unwrap_or(0)
            + 1;

        // Emit batch started event
        GoalEvents::batch_started(&env, batch_id, request_count);

        // Get current ledger timestamp
        let current_ledger = env.ledger().sequence() as u64;

        // Initialize result tracking
        let mut results: Vec<GoalResult> = Vec::new(&env);
        let mut successful_count: u32 = 0;
        let mut failed_count: u32 = 0;
        let mut total_target_amount: i128 = 0;
        let mut total_initial_contributions: i128 = 0;
        let mut goal_id_counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LastGoalId)
            .unwrap_or(0);

        // Process each request
        for request in requests.iter() {
            // Validate the request
            match validate_goal_request(&env, &request) {
                Ok(()) => {
                    // Validation succeeded - create the goal
                    goal_id_counter += 1;

                    let goal = SavingsGoal {
                        goal_id: goal_id_counter,
                        user: request.user.clone(),
                        goal_name: request.goal_name.clone(),
                        target_amount: request.target_amount,
                        current_amount: request.initial_contribution,
                        deadline: request.deadline,
                        created_at: current_ledger,
                        is_active: true,
                    };

                    // Accumulate metrics
                    total_target_amount = total_target_amount
                        .checked_add(request.target_amount)
                        .unwrap_or(i128::MAX);
                    total_initial_contributions = total_initial_contributions
                        .checked_add(request.initial_contribution)
                        .unwrap_or(i128::MAX);
                    successful_count += 1;

                    // Store the goal (optimized - one write per goal)
                    env.storage()
                        .persistent()
                        .set(&DataKey::Goal(goal_id_counter), &goal);

                    // Update user's goal list
                    let mut user_goals: Vec<u64> = env
                        .storage()
                        .persistent()
                        .get(&DataKey::UserGoals(request.user.clone()))
                        .unwrap_or(Vec::new(&env));
                    user_goals.push_back(goal_id_counter);
                    env.storage()
                        .persistent()
                        .set(&DataKey::UserGoals(request.user.clone()), &user_goals);

                    // Emit success event
                    GoalEvents::goal_created(&env, batch_id, &goal);

                    // Emit high-value goal event if applicable (>= 100,000 XLM)
                    if request.target_amount >= 1_000_000_000_000 {
                        GoalEvents::high_value_goal(
                            &env,
                            batch_id,
                            goal_id_counter,
                            request.target_amount,
                        );
                    }

                    results.push_back(GoalResult::Success(goal));
                }
                Err(error_code) => {
                    // Validation failed - record failure
                    failed_count += 1;

                    // Emit failure event
                    GoalEvents::goal_creation_failed(&env, batch_id, &request.user, error_code);

                    results.push_back(GoalResult::Failure(request.user.clone(), error_code));
                }
            }
        }

        // Calculate average goal amount
        let avg_goal_amount = if successful_count > 0 {
            total_target_amount / successful_count as i128
        } else {
            0
        };

        // Create metrics
        let metrics = BatchGoalMetrics {
            total_requests: request_count,
            successful_goals: successful_count,
            failed_goals: failed_count,
            total_target_amount,
            total_initial_contributions,
            avg_goal_amount,
            processed_at: current_ledger,
        };

        // Update storage (batched at the end for efficiency)
        let total_goals: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalGoalsCreated)
            .unwrap_or(0);
        let total_batches: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalBatchesProcessed)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&DataKey::LastBatchId, &batch_id);
        env.storage()
            .instance()
            .set(&DataKey::LastGoalId, &goal_id_counter);
        env.storage().instance().set(
            &DataKey::TotalGoalsCreated,
            &(total_goals + successful_count as u64),
        );
        env.storage()
            .instance()
            .set(&DataKey::TotalBatchesProcessed, &(total_batches + 1));

        // Emit batch completed event
        GoalEvents::batch_completed(
            &env,
            batch_id,
            successful_count,
            failed_count,
            total_target_amount,
        );

        BatchGoalResult {
            batch_id,
            total_requests: request_count,
            successful: successful_count,
            failed: failed_count,
            results,
            metrics,
        }
    }

    /// Retrieves a savings goal by ID.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `goal_id` - The ID of the goal to retrieve
    ///
    /// # Returns
    /// * `Option<SavingsGoal>` - The goal if found
    pub fn get_goal(env: Env, goal_id: u64) -> Option<SavingsGoal> {
        env.storage().persistent().get(&DataKey::Goal(goal_id))
    }

    /// Retrieves all goal IDs for a specific user.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `user` - The user's address
    ///
    /// # Returns
    /// * `Vec<u64>` - Vector of goal IDs for the user
    pub fn get_user_goals(env: Env, user: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::UserGoals(user))
            .unwrap_or(Vec::new(&env))
    }

    /// Returns the admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized")
    }

    /// Updates the admin address.
    pub fn set_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        Self::require_admin(&env, &current_admin);

        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Returns the last created batch ID.
    pub fn get_last_batch_id(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastBatchId)
            .unwrap_or(0)
    }

    /// Returns the last created goal ID.
    pub fn get_last_goal_id(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::LastGoalId)
            .unwrap_or(0)
    }

    /// Returns the total number of goals created.
    pub fn get_total_goals_created(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalGoalsCreated)
            .unwrap_or(0)
    }

    /// Returns the total number of batches processed.
    pub fn get_total_batches_processed(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalBatchesProcessed)
            .unwrap_or(0)
    }

    // Internal helper to verify admin
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");

        if *caller != admin {
            panic_with_error!(env, SavingsGoalError::Unauthorized);
        }
    }
}

#[cfg(test)]
mod test;
