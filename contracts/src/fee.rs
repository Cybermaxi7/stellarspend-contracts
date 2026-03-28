mod storage;

use soroban_sdk::{contractimpl, contracttype, Address, Env, Vec};
pub use storage::{FeeLog, FeeLogKind};

use self::storage::{
    append_fee_log, get_fee_log as read_fee_log, get_fee_log_count as read_fee_log_count,
    get_fee_logs as read_fee_logs, FeeLogKind as StorageFeeLogKind,
};

#[derive(Clone)]
#[contracttype]
pub struct FeeWindow {
    pub start: u64,   // ledger timestamp start
    pub end: u64,     // ledger timestamp end
    pub fee_rate: u32 // basis points (e.g., 100 = 1%)
}

#[derive(Clone)]
#[contracttype]
pub struct FeeConfig {
    pub default_fee_rate: u32,
    pub windows: Vec<FeeWindow>,
}

pub fn calculate_fee(env: &Env, amount: i128, config: &FeeConfig) -> i128 {
    let now = env.ledger().timestamp();

    let mut fee_rate = config.default_fee_rate;
    for window in config.windows.iter() {
        if now >= window.start && now <= window.end {
            fee_rate = window.fee_rate;
            break;
        }
    }

    (amount * fee_rate as i128) / 10_000 // basis points calculation
}

pub fn validate_windows(windows: &Vec<FeeWindow>) -> bool {
    for w in windows.iter() {
        if w.start >= w.end {
            return false;
        }
    }
    true
}

pub struct FeeContract;

#[contractimpl]
impl FeeContract {
    pub fn simulate_fee(env: Env, amount: i128, _user: Address) -> i128 {
        // Read-only: fetch config, calculate fee, return estimate
        let config: FeeConfig = env.storage().persistent().get(&"fee_config").unwrap();
        calculate_fee(&env, amount, &config)
    }

    pub fn get_fee(env: Env, amount: i128) -> i128 {
        let config: FeeConfig = env.storage().persistent().get(&"fee_config").unwrap();
        let fee = calculate_fee(&env, amount, &config);
        append_fee_log(&env, None, amount, fee, StorageFeeLogKind::Charge);
        fee
    }

    pub fn charge_fee(env: Env, payer: Address, amount: i128) -> i128 {
        let config: FeeConfig = env.storage().persistent().get(&"fee_config").unwrap();
        let fee = calculate_fee(&env, amount, &config);
        append_fee_log(
            &env,
            Some(payer),
            amount,
            fee,
            StorageFeeLogKind::Charge,
        );
        fee
    }

    pub fn record_fee_refund(env: Env, payer: Address, amount: i128, refunded_fee: i128) -> FeeLog {
        append_fee_log(
            &env,
            Some(payer),
            amount,
            refunded_fee,
            StorageFeeLogKind::Refund,
        )
    }

    pub fn get_fee_log(env: Env, id: u64) -> Option<FeeLog> {
        read_fee_log(&env, id)
    }

    pub fn get_fee_log_count(env: Env) -> u64 {
        read_fee_log_count(&env)
    }

    pub fn get_fee_logs(env: Env, start: u64, end: u64) -> Vec<FeeLog> {
        read_fee_logs(&env, start, end)
    }
}

pub fn safe_multiply(amount: i128, rate: u32) -> Option<i128> {
    amount.checked_mul(rate as i128)
}

pub fn safe_divide(value: i128, divisor: i128) -> Option<i128> {
    value.checked_div(divisor)
}
