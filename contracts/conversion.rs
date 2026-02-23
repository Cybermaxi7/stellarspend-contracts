//! Asset conversion contract for Stellar assets.

use soroban_sdk::{contract, contractimpl, Address, Env};

pub struct MockPriceOracle;
impl MockPriceOracle {
    pub fn get_rate(from: &Address, to: &Address) -> Option<(u32, u32)> {
        // Mock rates for tests
        Some((2, 1)) // 1 from = 2 to
    }
}

#[contract]
pub struct ConversionContract;

#[contractimpl]
impl ConversionContract {
    pub fn convert_assets(env: Env, user: Address, from_token: Address, to_token: Address, amount: i128) -> Result<i128, &'static str> {
        // Validate asset pair
        if from_token == to_token {
            return Err("same_token_conversion_not_allowed");
        }
        if amount <= 0 {
            return Err("invalid_amount");
        }
        // Get rate from mock oracle
        let (num, denom) = MockPriceOracle::get_rate(&from_token, &to_token).ok_or("rate_not_found")?;
        // Safe conversion with rounding
        let converted = amount.checked_mul(num as i128).ok_or("overflow")? / denom as i128;
        // Emit event
        env.events().publish(("conversion", user.clone()), (from_token.clone(), to_token.clone(), amount, converted));
        Ok(converted)
    }
}
