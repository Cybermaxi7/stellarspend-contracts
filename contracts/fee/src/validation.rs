use soroban_sdk::Env;

use crate::{storage::MAX_FEE_BPS, FeeContractError};
use shared::utils::validate_amount as validate_non_negative_amount;
use soroban_sdk::panic_with_error;

/// Validate fee basis points are within [0, MAX_FEE_BPS].
/// Panics with InvalidConfig on failure. Returns true on success to enable
/// chaining in callers when desired.
pub fn validate_fee_bps_or_panic(env: &Env, fee_bps: u32) -> bool {
    if fee_bps > MAX_FEE_BPS {
        panic_with_error!(env, FeeContractError::InvalidConfig);
    }
    true
}

/// Validate minimum fee is non-negative.
/// Panics with InvalidConfig on failure. Returns true on success.
pub fn validate_min_fee_or_panic(env: &Env, min_fee: i128) -> bool {
    if validate_non_negative_amount(min_fee).is_err() {
        panic_with_error!(env, FeeContractError::InvalidConfig);
    }
    true
}

/// Validate that a discount (in bps) is not greater than the base fee bps,
/// and both are within allowed ranges. Not currently invoked by the contract,
/// but provided for reuse by future config methods.
pub fn validate_discount_vs_base_or_panic(env: &Env, base_bps: u32, discount_bps: u32) -> bool {
    validate_fee_bps_or_panic(env, base_bps);
    validate_fee_bps_or_panic(env, discount_bps);
    if discount_bps > base_bps {
        panic_with_error!(env, FeeContractError::InvalidConfig);
    }
    true
}

/// Validate fee basis points without panicking.
/// Returns Ok(()) if valid, or Err(FeeContractError::InvalidConfig) if invalid.
/// This is a safer alternative to validate_fee_bps_or_panic that allows
/// callers to handle errors gracefully.
pub fn validate_fee_bps(fee_bps: u32) -> Result<(), FeeContractError> {
    if fee_bps > MAX_FEE_BPS {
        Err(FeeContractError::InvalidConfig)
    } else {
        Ok(())
    }
}

/// Validate minimum fee without panicking.
/// Returns Ok(()) if valid, or Err(FeeContractError::InvalidConfig) if invalid.
pub fn validate_min_fee(min_fee: i128) -> Result<(), FeeContractError> {
    if validate_non_negative_amount(min_fee).is_err() {
        Err(FeeContractError::InvalidConfig)
    } else {
        Ok(())
    }
}
