//! Data types and events for batch savings goal operations.

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

/// Maximum number of user-goal pairs in a single batch for optimization.
pub const MAX_BATCH_SIZE: u32 = 100;

/// Minimum goal amount (1 XLM in stroops)
pub const MIN_GOAL_AMOUNT: i128 = 10_000_000;

/// Maximum goal amount (1 billion XLM in stroops)
pub const MAX_GOAL_AMOUNT: i128 = 1_000_000_000_000_000_000;

/// Represents a savings goal request for a user.
#[derive(Clone, Debug)]
#[contracttype]
pub struct SavingsGoalRequest {
    /// User's address
    pub user: Address,
    /// Goal name/description (e.g., "vacation", "emergency_fund", "house")
    pub goal_name: Symbol,
    /// Target amount to save (in stroops)
    pub target_amount: i128,
    /// Deadline timestamp (ledger sequence number)
    pub deadline: u64,
    /// Initial contribution amount (optional, can be 0)
    pub initial_contribution: i128,
}

/// Represents a created savings goal.
#[derive(Clone, Debug)]
#[contracttype]
pub struct SavingsGoal {
    /// Unique goal ID
    pub goal_id: u64,
    /// User's address
    pub user: Address,
    /// Goal name/description
    pub goal_name: Symbol,
    /// Target amount to save (in stroops)
    pub target_amount: i128,
    /// Current saved amount (in stroops)
    pub current_amount: i128,
    /// Deadline timestamp (ledger sequence number)
    pub deadline: u64,
    /// Goal creation timestamp
    pub created_at: u64,
    /// Whether the goal is active
    pub is_active: bool,
}

/// Result of processing a single goal creation.
#[derive(Clone, Debug)]
#[contracttype]
pub enum GoalResult {
    Success(SavingsGoal),
    Failure(Address, u32), // user address, error code
}

/// Aggregated metrics for a batch of goal creations.
#[derive(Clone, Debug)]
#[contracttype]
pub struct BatchGoalMetrics {
    /// Total number of goal requests
    pub total_requests: u32,
    /// Number of successful goal creations
    pub successful_goals: u32,
    /// Number of failed goal creations
    pub failed_goals: u32,
    /// Total target amount across all goals
    pub total_target_amount: i128,
    /// Total initial contributions
    pub total_initial_contributions: i128,
    /// Average goal amount
    pub avg_goal_amount: i128,
    /// Batch processing timestamp
    pub processed_at: u64,
}

/// Result of batch goal creation.
#[derive(Clone, Debug)]
#[contracttype]
pub struct BatchGoalResult {
    /// Batch ID
    pub batch_id: u64,
    /// Total number of requests
    pub total_requests: u32,
    /// Number of successful creations
    pub successful: u32,
    /// Number of failed creations
    pub failed: u32,
    /// Individual goal results
    pub results: Vec<GoalResult>,
    /// Aggregated metrics
    pub metrics: BatchGoalMetrics,
}

/// Storage keys for contract state.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Last created batch ID
    LastBatchId,
    /// Last created goal ID
    LastGoalId,
    /// Stored goal by goal_id
    Goal(u64),
    /// User's goals (user address -> Vec<goal_id>)
    UserGoals(Address),
    /// Total goals created lifetime
    TotalGoalsCreated,
    /// Total batches processed lifetime
    TotalBatchesProcessed,
}

/// Error codes for goal validation and creation.
pub mod ErrorCode {
    /// Invalid goal amount (too low or negative)
    pub const INVALID_AMOUNT: u32 = 0;
    /// Invalid deadline (in the past or too far in future)
    pub const INVALID_DEADLINE: u32 = 1;
    /// Invalid initial contribution (negative or exceeds target)
    pub const INVALID_INITIAL_CONTRIBUTION: u32 = 2;
    /// Goal name is empty or invalid
    pub const INVALID_GOAL_NAME: u32 = 3;
    /// User address is invalid
    pub const INVALID_USER_ADDRESS: u32 = 4;
}

/// Events emitted by the savings goals contract.
pub struct GoalEvents;

impl GoalEvents {
    /// Event emitted when batch goal creation starts.
    pub fn batch_started(env: &Env, batch_id: u64, request_count: u32) {
        let topics = (symbol_short!("batch"), symbol_short!("started"));
        env.events().publish(topics, (batch_id, request_count));
    }

    /// Event emitted when a goal is successfully created.
    pub fn goal_created(env: &Env, batch_id: u64, goal: &SavingsGoal) {
        let topics = (symbol_short!("goal"), symbol_short!("created"), batch_id);
        env.events().publish(
            topics,
            (goal.goal_id, goal.user.clone(), goal.target_amount),
        );
    }

    /// Event emitted when goal creation fails.
    pub fn goal_creation_failed(env: &Env, batch_id: u64, user: &Address, error_code: u32) {
        let topics = (symbol_short!("goal"), symbol_short!("failed"), batch_id);
        env.events().publish(topics, (user.clone(), error_code));
    }

    /// Event emitted when batch goal creation completes.
    pub fn batch_completed(
        env: &Env,
        batch_id: u64,
        successful: u32,
        failed: u32,
        total_amount: i128,
    ) {
        let topics = (symbol_short!("batch"), symbol_short!("completed"), batch_id);
        env.events()
            .publish(topics, (successful, failed, total_amount));
    }

    /// Event emitted for high-value goals (>= 10,000 XLM).
    pub fn high_value_goal(env: &Env, batch_id: u64, goal_id: u64, amount: i128) {
        let topics = (symbol_short!("goal"), symbol_short!("highval"), batch_id);
        env.events().publish(topics, (goal_id, amount));
    }
}
