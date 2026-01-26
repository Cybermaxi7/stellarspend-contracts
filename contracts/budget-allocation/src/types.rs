use soroban_sdk::{contracttype, Address};

/// Request structure for setting a user's budget
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BudgetRequest {
    /// The user address to set budget for
    pub user: Address,
    /// The monthly budget amount
    pub amount: i128,
}

/// Stored budget record for a user
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
    TotalAllocated, // Track global stats if needed
}

/// Result of a batch budget allocation operation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchBudgetResult {
    pub successful: u32,
    pub failed: u32,
    pub total_amount: i128,
}
