//! # Batch Budget Updates Contract
//!
//! A Soroban smart contract for updating multiple user budgets in a single transaction.
//!
//! ## Features
//!
//! - **Batch Processing**: Efficiently update budgets for multiple users in a single call
//! - **Atomic Execution**: Ensures all updates succeed or fail together
//! - **Validation**: Prevents negative or zero allocations per user
//! - **Total Allocation Validation**: Validates total allocation per user
//! - **Event Emission**: Emits events for each updated budget
//!
#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec, panic_with_error};

/// Error codes for the batch budget contract.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum BatchBudgetError {
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
    /// Invalid budget amount (negative or zero)
    InvalidAmount = 6,
    /// Duplicate user in batch
    DuplicateUser = 7,
    /// Arithmetic overflow detected
    Overflow = 8,
    /// Already initialized
    AlreadyInitialized = 9,
}

impl From<BatchBudgetError> for soroban_sdk::Error {
    fn from(e: BatchBudgetError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

/// Request structure for batch budget updates
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetUpdateRequest {
    /// The user address to update budget for
    pub user: Address,
    /// The new budget amount
    pub amount: i128,
}

/// Result of a single budget update
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BudgetUpdateResult {
    Success(Address, i128),
    Failure(Address, i128, u32), // user, amount, error_code
}

/// Result of a batch budget update operation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchBudgetUpdateResult {
    pub total_requests: u32,
    pub successful: u32,
    pub failed: u32,
    pub total_amount: i128,
    pub results: Vec<BudgetUpdateResult>,
}

/// Budget record for a user
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetRecord {
    pub user: Address,
    pub amount: i128,
    pub last_updated: u64,
}

/// Storage keys for the contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Budget(Address),
    TotalAllocated,
    TotalBatches,
    TotalUpdatesProcessed,
}

/// Maximum batch size to prevent gas limit issues
pub const MAX_BATCH_SIZE: u32 = 100;

#[contract]
pub struct BatchBudgetContract;

#[contractimpl]
impl BatchBudgetContract {
    /// Initializes the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, BatchBudgetError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TotalAllocated, &0i128);
        env.storage().instance().set(&DataKey::TotalBatches, &0u64);
        env.storage().instance().set(&DataKey::TotalUpdatesProcessed, &0u64);
    }

    /// Updates multiple user budgets in a single atomic transaction.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address calling the function
    /// * `requests` - Vector of user-budget update requests
    pub fn batch_update_budgets(
        env: Env,
        admin: Address,
        requests: Vec<BudgetUpdateRequest>,
    ) -> BatchBudgetUpdateResult {
        // Verify admin authority
        admin.require_auth();
        Self::require_admin(&env, &admin);

        // Validate batch size
        let request_count = requests.len();
        if request_count == 0 {
            panic_with_error!(&env, BatchBudgetError::EmptyBatch);
        }
        if request_count > MAX_BATCH_SIZE {
            panic_with_error!(&env, BatchBudgetError::BatchTooLarge);
        }

        // Get batch ID with overflow protection
        let batch_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalBatches)
            .unwrap_or(0)
            .checked_add(1)
            .unwrap_or_else(|| panic_with_error!(&env, BatchBudgetError::Overflow));

        // Emit batch started event
        env.events().publish(
            (symbol_short!("batch"), symbol_short!("started")),
            (batch_id, request_count),
        );

        // Initialize result tracking
        let mut results: Vec<BudgetUpdateResult> = Vec::new(&env);
        let mut successful_count: u32 = 0;
        let mut failed_count: u32 = 0;
        let mut total_amount: i128 = 0;
        let current_time = env.ledger().timestamp();

        // Get current total allocated
        let mut total_allocated: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalAllocated)
            .unwrap_or(0);

        // First pass: Validate all requests and check for duplicates
        let mut validated_requests: Vec<(BudgetUpdateRequest, bool, u32)> = Vec::new(&env);
        let mut seen_users: Vec<Address> = Vec::new(&env);

        for request in requests.iter() {
            let mut is_valid = true;
            let mut error_code = 0u32;

            // Check for duplicate users
            for seen_user in seen_users.iter() {
                if *seen_user == request.user {
                    is_valid = false;
                    error_code = 7; // Duplicate user
                    break;
                }
            }

            if is_valid {
                seen_users.push_back(request.user.clone());
                
                // Validate amount
                if request.amount <= 0 {
                    is_valid = false;
                    error_code = 6; // Invalid amount
                }
            }

            validated_requests.push_back((request.clone(), is_valid, error_code));
        }

        // Second pass: Process each validated request
        for (request, is_valid, error_code) in validated_requests.iter() {
            if !is_valid {
                // Validation failed - record and continue
                results.push_back(BudgetUpdateResult::Failure(
                    request.user.clone(),
                    request.amount,
                    error_code.clone(),
                ));
                failed_count += 1;

                // Emit failure event
                env.events().publish(
                    (symbol_short!("budget"), symbol_short!("update_failed")),
                    (request.user.clone(), request.amount, error_code.clone()),
                );
                continue;
            }

            // Process successful update
            let old_amount: i128 = if let Some(old_record) = env
                .storage()
                .persistent()
                .get(&DataKey::Budget(request.user.clone()))
            {
                old_record.amount
            } else {
                0
            };

            // Update total allocated with overflow protection
            total_allocated = total_allocated
                .checked_sub(old_amount)
                .unwrap_or_else(|| panic_with_error!(&env, BatchBudgetError::Overflow))
                .checked_add(request.amount)
                .unwrap_or_else(|| panic_with_error!(&env, BatchBudgetError::Overflow));

            // Create new budget record
            let record = BudgetRecord {
                user: request.user.clone(),
                amount: request.amount,
                last_updated: current_time,
            };

            // Store the updated budget
            env.storage()
                .persistent()
                .set(&DataKey::Budget(request.user.clone()), &record);

            // Record success
            results.push_back(BudgetUpdateResult::Success(
                request.user.clone(),
                request.amount,
            ));
            successful_count += 1;
            total_amount = total_amount
                .checked_add(request.amount)
                .unwrap_or_else(|| panic_with_error!(&env, BatchBudgetError::Overflow));

            // Emit success event
            env.events().publish(
                (symbol_short!("budget"), symbol_short!("updated")),
                (request.user.clone(), request.amount, current_time),
            );
        }

        // Update storage statistics
        env.storage()
            .instance()
            .set(&DataKey::TotalAllocated, &total_allocated);
        env.storage()
            .instance()
            .set(&DataKey::TotalBatches, &batch_id);
        
        let total_processed: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalUpdatesProcessed)
            .unwrap_or(0);
        let new_total_processed = total_processed
            .checked_add(request_count as u64)
            .unwrap_or_else(|| panic_with_error!(&env, BatchBudgetError::Overflow));
        env.storage()
            .instance()
            .set(&DataKey::TotalUpdatesProcessed, &new_total_processed);

        // Emit batch completed event
        env.events().publish(
            (symbol_short!("batch"), symbol_short!("completed")),
            (batch_id, successful_count, failed_count, total_amount),
        );

        BatchBudgetUpdateResult {
            total_requests: request_count,
            successful: successful_count,
            failed: failed_count,
            total_amount,
            results,
        }
    }

    /// Retrieves the budget for a specific user.
    pub fn get_budget(env: Env, user: Address) -> Option<BudgetRecord> {
        env.storage().persistent().get(&DataKey::Budget(user))
    }

    /// Returns the admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, BatchBudgetError::NotInitialized))
    }

    /// Returns the total allocated budget amount
    pub fn get_total_allocated(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalAllocated)
            .unwrap_or(0)
    }

    /// Returns the total number of batches processed
    pub fn get_total_batches(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalBatches)
            .unwrap_or(0)
    }

    /// Returns the total number of updates processed
    pub fn get_total_updates_processed(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalUpdatesProcessed)
            .unwrap_or(0)
    }

    /// Internal helper to verify admin authority
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, BatchBudgetError::NotInitialized));

        if *caller != admin {
            panic_with_error!(env, BatchBudgetError::Unauthorized);
        }
    }
}
