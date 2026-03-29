//! # Batch Budget Updates Tests

use soroban_sdk::{Address, Env, Symbol, Vec};
use stellar_contract_sdk::testutils::Address as TestAddress;

// Import the contracts we're testing
mod batch_budget {
    pub use crate::contracts::batch_budget::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as TestAddress};

    #[test]
    fn test_batch_update_budgets_success() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Create batch update requests
        let mut requests = Vec::new(&env);
        requests.push_back(TestBudgetUpdateRequest {
            user: user1.clone(),
            amount: 1000,
        });
        requests.push_back(TestBudgetUpdateRequest {
            user: user2.clone(),
            amount: 2000,
        });
        requests.push_back(TestBudgetUpdateRequest {
            user: user3.clone(),
            amount: 1500,
        });

        // Convert to contract type
        let contract_requests: Vec<contracts::batch::BudgetUpdateRequest> =
            requests.iter().map(|req| req.clone().into()).collect();

        // Execute batch update
        let result = contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            contract_requests,
        );

        // Verify results
        assert_eq!(result.total_requests, 3);
        assert_eq!(result.successful, 3);
        assert_eq!(result.failed, 0);
        assert_eq!(result.total_amount, 4500);

        // Verify individual budgets were set
        let budget1 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user1.clone());
        assert!(budget1.is_some());
        assert_eq!(budget1.unwrap().amount, 1000);

        let budget2 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user2.clone());
        assert!(budget2.is_some());
        assert_eq!(budget2.unwrap().amount, 2000);

        let budget3 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user3.clone());
        assert!(budget3.is_some());
        assert_eq!(budget3.unwrap().amount, 1500);

        // Verify total allocated
        let total_allocated =
            contracts::batch::BatchBudgetContract::get_total_allocated(env.clone());
        assert_eq!(total_allocated, 4500);

        // Verify batch statistics
        let total_batches = contracts::batch::BatchBudgetContract::get_total_batches(env.clone());
        assert_eq!(total_batches, 1);

        let total_updates =
            contracts::batch::BatchBudgetContract::get_total_updates_processed(env.clone());
        assert_eq!(total_updates, 3);
    }

    #[test]
    #[should_panic(expected = "EmptyBatch")]
    fn test_batch_update_empty_batch() {
        let env = Env::default();
        let admin = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Try to execute empty batch
        let empty_requests: Vec<contracts::batch::BudgetUpdateRequest> = Vec::new(&env);
        contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            empty_requests,
        );
    }

    #[test]
    #[should_panic(expected = "BatchTooLarge")]
    fn test_batch_update_too_large() {
        let env = Env::default();
        let admin = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Create batch that exceeds maximum size
        let mut requests = Vec::new(&env);
        for i in 0..101 {
            requests.push_back(contracts::batch::BudgetUpdateRequest {
                user: Address::generate(&env),
                amount: 100,
            });
        }

        contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            requests,
        );
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_batch_update_unauthorized() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let unauthorized_user = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Create batch update requests
        let mut requests = Vec::new(&env);
        requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: Address::generate(&env),
            amount: 1000,
        });

        // Try to execute batch update with unauthorized user
        contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            unauthorized_user.clone(),
            requests,
        );
    }

    #[test]
    fn test_batch_update_invalid_amounts() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Create batch update requests with invalid amounts
        let mut requests = Vec::new(&env);
        requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user1.clone(),
            amount: 1000, // Valid
        });
        requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user2.clone(),
            amount: 0, // Invalid (zero)
        });
        requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user3.clone(),
            amount: -500, // Invalid (negative)
        });

        // Execute batch update
        let result = contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            requests,
        );

        // Verify results
        assert_eq!(result.total_requests, 3);
        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 2);
        assert_eq!(result.total_amount, 1000); // Only valid amount counted

        // Verify only valid budget was set
        let budget1 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user1.clone());
        assert!(budget1.is_some());
        assert_eq!(budget1.unwrap().amount, 1000);

        let budget2 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user2.clone());
        assert!(budget2.is_none());

        let budget3 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user3.clone());
        assert!(budget3.is_none());
    }

    #[test]
    fn test_batch_update_duplicate_users() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Create batch update requests with duplicate users
        let mut requests = Vec::new(&env);
        requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user1.clone(),
            amount: 1000,
        });
        requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user2.clone(),
            amount: 2000,
        });
        requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user1.clone(), // Duplicate
            amount: 1500,
        });

        // Execute batch update
        let result = contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            requests,
        );

        // Verify results
        assert_eq!(result.total_requests, 3);
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 1);
        assert_eq!(result.total_amount, 3000); // Only valid amounts counted

        // Verify budgets were set correctly (first occurrence of user1 should succeed)
        let budget1 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user1.clone());
        assert!(budget1.is_some());
        assert_eq!(budget1.unwrap().amount, 1000);

        let budget2 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user2.clone());
        assert!(budget2.is_some());
        assert_eq!(budget2.unwrap().amount, 2000);
    }

    #[test]
    fn test_batch_update_existing_budgets() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Set initial budgets
        let mut initial_requests = Vec::new(&env);
        initial_requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user1.clone(),
            amount: 1000,
        });
        initial_requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user2.clone(),
            amount: 2000,
        });

        contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            initial_requests,
        );

        // Verify initial total allocated
        let total_allocated =
            contracts::batch::BatchBudgetContract::get_total_allocated(env.clone());
        assert_eq!(total_allocated, 3000);

        // Create update requests
        let mut update_requests = Vec::new(&env);
        update_requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user1.clone(),
            amount: 1500, // Increased
        });
        update_requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user2.clone(),
            amount: 1000, // Decreased
        });

        // Execute batch update
        let result = contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            update_requests,
        );

        // Verify results
        assert_eq!(result.total_requests, 2);
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 0);
        assert_eq!(result.total_amount, 2500);

        // Verify updated budgets
        let budget1 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user1.clone());
        assert!(budget1.is_some());
        assert_eq!(budget1.unwrap().amount, 1500);

        let budget2 = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user2.clone());
        assert!(budget2.is_some());
        assert_eq!(budget2.unwrap().amount, 1000);

        // Verify total allocated was updated correctly
        let total_allocated =
            contracts::batch::BatchBudgetContract::get_total_allocated(env.clone());
        assert_eq!(total_allocated, 2500);
    }

    #[test]
    fn test_get_admin() {
        let env = Env::default();
        let admin = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Get admin address
        let retrieved_admin = contracts::batch::BatchBudgetContract::get_admin(env.clone());
        assert_eq!(retrieved_admin, admin);
    }

    #[test]
    fn test_get_budget_nonexistent() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Try to get budget for non-existent user
        let budget = contracts::batch::BatchBudgetContract::get_budget(env.clone(), user.clone());
        assert!(budget.is_none());
    }

    #[test]
    fn test_atomic_execution() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        // Initialize the contract
        contracts::batch::BatchBudgetContract::initialize(env.clone(), admin.clone());

        // Set initial budget for user1
        let mut initial_requests = Vec::new(&env);
        initial_requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user1.clone(),
            amount: 1000,
        });

        contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            initial_requests,
        );

        // Verify initial state
        let initial_budget =
            contracts::batch::BatchBudgetContract::get_budget(env.clone(), user1.clone());
        assert_eq!(initial_budget.unwrap().amount, 1000);

        let initial_total = contracts::batch::BatchBudgetContract::get_total_allocated(env.clone());
        assert_eq!(initial_total, 1000);

        // Create batch with one valid and one invalid request
        let mut mixed_requests = Vec::new(&env);
        mixed_requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user1.clone(),
            amount: 2000, // Valid update
        });
        mixed_requests.push_back(contracts::batch::BudgetUpdateRequest {
            user: user2.clone(),
            amount: -500, // Invalid
        });

        // Execute batch update
        let result = contracts::batch::BatchBudgetContract::batch_update_budgets(
            env.clone(),
            admin.clone(),
            mixed_requests,
        );

        // Verify partial success
        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 1);

        // Verify valid update was applied
        let updated_budget =
            contracts::batch::BatchBudgetContract::get_budget(env.clone(), user1.clone());
        assert_eq!(updated_budget.unwrap().amount, 2000);

        // Verify total was updated correctly (only valid changes applied)
        let final_total = contracts::batch::BatchBudgetContract::get_total_allocated(env.clone());
        assert_eq!(final_total, 2000);
    }
}
