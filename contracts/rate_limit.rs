//! Rate limit logic for wallet transaction frequency.

use soroban_sdk::{contract, contractimpl, Address, Env};

const DEFAULT_LIMIT: u32 = 5; // Default max transactions per window
const WINDOW_SECONDS: u64 = 3600; // 1 hour window

#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    pub max_tx: u32,
    pub window: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_tx: DEFAULT_LIMIT,
            window: WINDOW_SECONDS,
        }
    }
}

#[contract]
pub struct RateLimitContract;

#[contractimpl]
impl RateLimitContract {
    /// Checks and enforces rate limit for a wallet address.
    pub fn check_and_record(env: Env, wallet: Address) -> Result<(), &'static str> {
        let config = RateLimitConfig::default();
        let now = env.ledger().timestamp();
        let window_start = now - (now % config.window);
        let key = ("rate_limit", wallet.clone(), window_start);
        let count: u32 = env.storage().persistent().get(&key).unwrap_or(0);
        if count >= config.max_tx {
            env.events().publish(("rate_limit", wallet.clone()), count);
            return Err("rate_limit_exceeded");
        }
        env.storage().persistent().set(&key, &(count + 1));
        Ok(())
    }
    /// Allows updating rate limit config (admin only, mock auth)
    pub fn set_config(_env: Env, _admin: Address, _max_tx: u32, _window: u64) {
        // For extensibility: not implemented, mock only
    }
}
