use soroban_sdk::{contracttype, symbol_short, Address, Env, Vec};

pub const MAX_BATCH_SIZE: u32 = 100;

#[derive(Clone, Debug)]
#[contracttype]
pub struct WalletCreateRequest {
    pub owner: Address,
}

#[derive(Clone, Debug)]
#[contracttype]
pub enum WalletCreateResult {
    Success(Address),
    Failure(Address, u32),
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct BatchCreateResult {
    pub total_requests: u32,
    pub successful: u32,
    pub failed: u32,
    pub results: Vec<WalletCreateResult>,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    TotalBatches,
    TotalWalletsCreated,
    Wallets(Address), // Map of address to wallet id or something
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Wallet {
    pub id: u64,
    pub owner: Address,
    pub created_at: u64,
}

pub struct WalletEvents;

impl WalletEvents {
    pub fn batch_started(env: &Env, batch_id: u64, request_count: u32) {
        let topics = (symbol_short!("batch"), symbol_short!("started"));
        env.events().publish(topics, (batch_id, request_count));
    }

    pub fn wallet_created(env: &Env, batch_id: u64, owner: &Address, wallet_id: u64) {
        let topics = (symbol_short!("wallet"), symbol_short!("created"), batch_id);
        env.events().publish(topics, (owner.clone(), wallet_id));
    }

    pub fn wallet_creation_failure(
        env: &Env,
        batch_id: u64,
        owner: &Address,
        error_code: u32,
    ) {
        let topics = (symbol_short!("wallet"), symbol_short!("failure"), batch_id);
        env.events().publish(topics, (owner.clone(), error_code));
    }

    pub fn batch_completed(
        env: &Env,
        batch_id: u64,
        successful: u32,
        failed: u32,
    ) {
        let topics = (symbol_short!("batch"), symbol_short!("completed"), batch_id);
        env.events().publish(topics, (successful, failed));
    }
}