use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentReminderRequest {
    pub user: Address,
    /// Due date as ledger sequence number (must be in the future).
    pub due_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchReminderResult {
    pub successful_count: u32,
    pub failed_addresses: Vec<Address>,
}
