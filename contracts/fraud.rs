//! Fraud detection logic for flagging suspicious transactions.

use soroban_sdk::{contract, contractimpl, Address, Env};

const DEFAULT_FRAUD_THRESHOLD: i128 = 10_000; // Default threshold

#[derive(Clone, Debug)]
pub struct FraudConfig {
    pub threshold: i128,
    pub max_daily: i128,
}

impl Default for FraudConfig {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_FRAUD_THRESHOLD,
            max_daily: 100_000,
        }
    }
}

#[contract]
pub struct FraudContract;

#[contractimpl]
impl FraudContract {
    /// Checks and flags suspicious transactions based on size and user history.
    pub fn check_transaction(env: Env, user: Address, amount: i128) -> bool {
        let config = FraudConfig::default();
        let mut flagged = false;
        let mut reasons = Vec::new();

        // Rule 1: Abnormal size
        if amount >= config.threshold {
            flagged = true;
            reasons.push("abnormal_size");
        }

        // Rule 2: Daily total
        let today = env.ledger().timestamp() / 86400;
        let user_key = ("user_daily", user.clone(), today);
        let prev_total: i128 = env.storage().persistent().get(&user_key).unwrap_or(0);
        let new_total = prev_total + amount;
        env.storage().persistent().set(&user_key, &new_total);
        if new_total > config.max_daily {
            flagged = true;
            reasons.push("daily_limit");
        }

        // Emit detailed fraud alert event
        if flagged {
            env.events().publish(("fraud_alert", user.clone()), (amount, reasons.clone()));
        }
        flagged
    }

    /// Allows updating fraud config (admin only, mock auth)
    pub fn set_config(_env: Env, _admin: Address, _threshold: i128, _max_daily: i128) {
        // For extensibility: not implemented, mock only
    }
}
