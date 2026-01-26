//! # Budget Allocation Contract
//!
//! A Soroban smart contract for assigning monthly budgets to multiple users
//! in a single batch operation.
//!
//! ## Features
//!
//! - **Batch Processing**: Efficiently allocate budgets for multiple users in a single call
//! - **Atomic Updates**: Ensures reliable state changes for each user
//! - **Validation**: Prevents invalid budget amounts
//! - **Event Emission**: Tracks budget updates and failures
//!
#![no_std]

mod test;
mod types;

use crate::types::{BatchBudgetResult, BudgetRecord, BudgetRequest, DataKey};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};

#[contract]
pub struct BudgetAllocationContract;

#[contractimpl]
impl BudgetAllocationContract {
    /// Initializes the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Assigns monthly budgets to multiple users in a single operation.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address calling the function
    /// * `requests` - List of user-budget pairs
    pub fn batch_allocate_budget(
        env: Env,
        admin: Address,
        requests: Vec<BudgetRequest>,
    ) -> BatchBudgetResult {
        // Verify admin authority
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Not initialized");
        if admin != stored_admin {
            panic!("Unauthorized");
        }

        let mut successful = 0;
        let mut failed = 0;
        let mut total_amount: i128 = 0;
        let current_time = env.ledger().timestamp();

        for req in requests.iter() {
            // Validate input amount
            if req.amount < 0 {
                failed += 1;
                // Emit failure event?
                env.events().publish(
                    (symbol_short!("budget"), symbol_short!("failed")),
                    (req.user, req.amount), // Amount is negative here
                );
                continue;
            }

            // Atomic update for user: overwrite existing
            let record = BudgetRecord {
                user: req.user.clone(),
                amount: req.amount,
                last_updated: current_time,
            };

            env.storage()
                .persistent()
                .set(&DataKey::Budget(req.user.clone()), &record);

            // Emit update event
            env.events().publish(
                (symbol_short!("budget"), symbol_short!("set")),
                (req.user, req.amount),
            );

            successful += 1;
            total_amount = total_amount.checked_add(req.amount).unwrap_or(i128::MAX);
            // Prevent overflow panic
        }

        BatchBudgetResult {
            successful,
            failed,
            total_amount,
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
            .expect("Not initialized")
    }
}
