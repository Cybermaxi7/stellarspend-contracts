use soroban_sdk::{
    contract, contractimpl, contracterror, contracttype, panic_with_error, symbol_short, 
    Address, Env, String, U256, Vec,
};

#[derive(Clone)]
#[contracttype]
pub enum ComplianceDataKey {
    Admin,
    Limit(String), // e.g., "max_amount"
    FlaggedTransactions, // Vec<U256>
    TransactionStatus(U256), // Transaction ID -> bool (true if flagged)
}

#[derive(Clone)]
#[contracttype]
pub struct FlaggedTransaction {
    pub transaction_id: U256,
    pub user: Address,
    pub amount: i128,
    pub limit_violated: String,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ComplianceError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidLimit = 4,
}

pub struct ComplianceEvents;

impl ComplianceEvents {
    pub fn transaction_flagged(env: &Env, transaction_id: &U256, user: &Address, limit_name: &String) {
        let topics = (symbol_short!("complian"), symbol_short!("flagged"));
        env.events()
            .publish(topics, (transaction_id.clone(), user.clone(), limit_name.clone(), env.ledger().timestamp()));
    }

    pub fn limit_updated(env: &Env, limit_name: &String, new_value: i128) {
        let topics = (symbol_short!("complian"), symbol_short!("limit_up"));
        env.events()
            .publish(topics, (limit_name.clone(), new_value, env.ledger().timestamp()));
    }
}

pub fn initialize_compliance(env: &Env, admin: Address) {
    if env.storage().instance().has(&ComplianceDataKey::Admin) {
        panic_with_error!(env, ComplianceError::AlreadyInitialized);
    }
    env.storage().instance().set(&ComplianceDataKey::Admin, &admin);
    env.storage().instance().set(&ComplianceDataKey::FlaggedTransactions, &Vec::<U256>::new(env));
}

pub fn require_admin(env: &Env, caller: &Address) {
    caller.require_auth();
    let admin: Address = env.storage().instance()
        .get(&ComplianceDataKey::Admin)
        .unwrap_or_else(|| panic_with_error!(env, ComplianceError::NotInitialized));
    if admin != *caller {
        panic_with_error!(env, ComplianceError::Unauthorized);
    }
}

pub fn set_limit(env: &Env, caller: Address, limit_name: String, value: i128) {
    require_admin(env, &caller);
    if value < 0 {
        panic_with_error!(env, ComplianceError::InvalidLimit);
    }
    env.storage().instance().set(&ComplianceDataKey::Limit(limit_name.clone()), &value);
    ComplianceEvents::limit_updated(env, &limit_name, value);
}

pub fn check_and_flag_transaction(
    env: &Env,
    transaction_id: U256,
    user: Address,
    amount: i128,
) -> bool {
    // This function would be called by other contracts during payment/transfer
    
    let limit_name = String::from_str(env, "max_transfer_amount");
    let max_amount: i128 = env.storage().instance()
        .get(&ComplianceDataKey::Limit(limit_name.clone()))
        .unwrap_or(i128::MAX);

    if amount > max_amount {
        // Flag the transaction
        let mut flagged: Vec<U256> = env.storage().instance()
            .get(&ComplianceDataKey::FlaggedTransactions)
            .unwrap_or_else(|| Vec::new(env));
        
        flagged.push_back(transaction_id.clone());
        env.storage().instance().set(&ComplianceDataKey::FlaggedTransactions, &flagged);
        env.storage().persistent().set(&ComplianceDataKey::TransactionStatus(transaction_id.clone()), &true);
        
        ComplianceEvents::transaction_flagged(env, &transaction_id, &user, &limit_name);
        return true;
    }

    false
}

pub fn is_transaction_flagged(env: &Env, transaction_id: U256) -> bool {
    env.storage().persistent().get(&ComplianceDataKey::TransactionStatus(transaction_id)).unwrap_or(false)
}

#[contract]
pub struct ComplianceContract;

#[contractimpl]
impl ComplianceContract {
    pub fn initialize(env: Env, admin: Address) {
        initialize_compliance(&env, admin);
    }

    pub fn set_limit(env: Env, admin: Address, limit_name: String, value: i128) {
        set_limit(&env, admin, limit_name, value);
    }

    pub fn check_transaction(env: Env, transaction_id: U256, user: Address, amount: i128) -> bool {
        check_and_flag_transaction(&env, transaction_id, user, amount)
    }

    pub fn is_flagged(env: Env, transaction_id: U256) -> bool {
        is_transaction_flagged(&env, transaction_id)
    }

    pub fn get_flagged_count(env: Env) -> u32 {
        let flagged: Vec<U256> = env.storage().instance()
            .get(&ComplianceDataKey::FlaggedTransactions)
            .unwrap_or_else(|| Vec::new(&env));
        flagged.len()
    }
}
