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

use crate::types::{BatchBudgetResult, BudgetRecord, BudgetRequest, CategoryBudgetRequest, UserBudgetCategories, DataKey};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec, Symbol, Map};

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

    /// Allocates budgets across multiple categories for a user.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - The admin address calling the function
    /// * `request` - Category budget allocation request
    pub fn allocate_budget_by_category(
        env: Env,
        admin: Address,
        request: CategoryBudgetRequest,
    ) -> bool {
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

        // Validate total amount matches sum of categories
        let mut calculated_total: i128 = 0;
        for category in request.categories.iter() {
            if category.amount < 0 {
                panic!("Negative category amount not allowed");
            }
            calculated_total = calculated_total.checked_add(category.amount)
                .expect("Overflow in category total calculation");
        }

        if calculated_total != request.total_amount {
            panic!("Total amount does not match sum of categories");
        }

        if request.total_amount < 0 {
            panic!("Negative total amount not allowed");
        }

        // Create category map
        let mut category_map = Map::<Symbol, i128>::new(&env);
        for category in request.categories.iter() {
            category_map.set(category.name, category.amount);
        }

        // Store user budget categories
        let user_categories = UserBudgetCategories {
            user: request.user.clone(),
            categories: category_map,
            total_amount: request.total_amount,
            last_updated: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::BudgetCategories(request.user.clone()), &user_categories);

        // Also update the legacy budget record for compatibility
        let budget_record = BudgetRecord {
            user: request.user.clone(),
            amount: request.total_amount,
            last_updated: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Budget(request.user.clone()), &budget_record);

        // Emit allocation events for each category
        for category in request.categories.iter() {
            env.events().publish(
                (symbol_short!("category"), symbol_short!("allocated")),
                (request.user.clone(), category.name, category.amount),
            );
        }

        // Emit total allocation event
        env.events().publish(
            (symbol_short!("budget"), symbol_short!("allocated")),
            (request.user, request.total_amount, request.categories.len()),
        );

        true
    }

    /// Retrieves budget categories for a specific user.
    pub fn get_budget_categories(env: Env, user: Address) -> Option<UserBudgetCategories> {
        env.storage().persistent().get(&DataKey::BudgetCategories(user))
    }

    /// Retrieves the budget for a specific category for a user.
    pub fn get_category_budget(env: Env, user: Address, category: Symbol) -> Option<i128> {
        let user_categories: Option<UserBudgetCategories> = env.storage().persistent().get(&DataKey::BudgetCategories(user));
        if let Some(categories) = user_categories {
            categories.categories.get(category)
        } else {
            None
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
