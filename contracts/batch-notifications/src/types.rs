use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationPayload {
    pub user: Address,
    pub message: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchResult {
    pub successful_count: u32,
    pub failed_addresses: Vec<Address>,
}
