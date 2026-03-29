// =====================
// FEE SYSTEM EXTENSIONS
// =====================
// This module implements:
// - Fee cap enforcement
// - Per-operation fee configuration
// - Fee distribution splitting
// - Fee pause mechanism
//
// See each function for details and usage.
/// Returns true if fees are currently paused.
pub fn is_fee_paused(env: &Env) -> bool {
    env.storage().instance().get(&crate::types::DataKey::FeePaused).unwrap_or(false)
}

/// Sets the fee pause flag (admin only).
pub fn set_fee_paused(env: &Env, admin: &Address, paused: bool) -> Result<(), ValidationError> {
    // Verify caller is admin
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&crate::types::DataKey::Admin)
        .expect("Contract not initialized");
    if *admin != stored_admin {
        return Err(ValidationError::InvalidAddress);
    }
    env.storage().instance().set(&crate::types::DataKey::FeePaused, &paused);
    crate::types::AnalyticsEvents::fee_pause_toggled(env, admin, paused);
    Ok(())
}
use crate::types::FeeRecipientShare;
/// Validates that the sum of recipient shares is exactly 10000 (100%).
pub fn validate_recipient_shares(shares: &Vec<FeeRecipientShare>) -> Result<(), ValidationError> {
    let mut total: u32 = 0;
    for s in shares.iter() {
        total = total.checked_add(s.share_bps).ok_or(ValidationError::InvalidAmount)?;
    }
    if total != 10000 {
        return Err(ValidationError::InvalidAmount);
    }
    Ok(())
}

/// Splits a fee amount among recipients and emits events.
pub fn distribute_fee(env: &Env, fee_amount: i128, shares: &Vec<FeeRecipientShare>) {
    // Validate shares sum to 100%
    validate_recipient_shares(shares).expect("Invalid recipient shares");
    let mut distributed = 0i128;
    for (i, s) in shares.iter().enumerate() {
        let is_last = i == (shares.len() - 1);
        let share_amt = if is_last {
            // Assign remainder to last recipient
            fee_amount - distributed
        } else {
            let amt = (fee_amount * (s.share_bps as i128)) / 10000;
            distributed += amt;
            amt
        };
        crate::types::AnalyticsEvents::fee_distributed(env, &s.recipient, share_amt, s.share_bps);
        // In a real contract, here you would transfer the share_amt to s.recipient
    }
}
//! # Fee Calculation Engine
//!
//! Implements dynamic fee calculation for transactions with configurable fee structures.
//! Supports percentage-based fees, tiered pricing, and automatic fee deductions.

use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::types::{FeeConfig, FeeTier, FeeCalculationResult, DataKey, FeeDeductionEvent, ValidationError, AnalyticsEvents};
use soroban_sdk::Symbol;
use crate::types::{
    AnalyticsEvents, DataKey, FeeCalculationResult, FeeConfig, FeeDeductionEvent, FeeTier,
    ValidationError,
};

/// Calculates fees for a single transaction based on the current fee configuration.
///
/// # Arguments
/// * `env` - The contract environment
/// * `amount` - The transaction amount
/// * `fee_config` - The fee configuration to use
///
/// # Returns
/// * `FeeCalculationResult` containing the calculated fee and net amount
pub fn calculate_transaction_fee(env: &Env, amount: i128, fee_config: &FeeConfig) -> FeeCalculationResult {
    if is_fee_paused(env) || amount <= 0 {
pub fn calculate_transaction_fee(
    env: &Env,
    amount: i128,
    fee_config: &FeeConfig,
) -> FeeCalculationResult {
    if amount <= 0 {
        return FeeCalculationResult {
            gross_amount: amount,
            fee_amount: 0,
            net_amount: amount,
            fee_percentage_bps: 0,
        };
    }

    let fee_amount = match &fee_config.fee_model {
        crate::types::FeeModel::Flat(flat_fee) => *flat_fee,
        crate::types::FeeModel::Percentage(percentage_bps) => {
            (amount * (*percentage_bps as i128)) / 10000
        }
        crate::types::FeeModel::Tiered(tiers) => calculate_tiered_fee(amount, tiers),
    };

    // Apply min and max fee constraints
    let constrained_fee = constrain_fee_amount(fee_amount, &fee_config);

    // Ensure fee doesn't exceed the transaction amount
    let final_fee = if constrained_fee > amount {
        amount // Cap fee at the transaction amount
    } else {
        constrained_fee
    };

    FeeCalculationResult {
        gross_amount: amount,
        fee_amount: final_fee,
        net_amount: amount - final_fee,
        fee_percentage_bps: calculate_effective_rate(amount, final_fee),
    }
}

/// Calculates fees based on tiered fee structure
fn calculate_tiered_fee(amount: i128, tiers: &Vec<FeeTier>) -> i128 {
    if tiers.is_empty() {
        return 0;
    }

    // Sort tiers by threshold in ascending order
    let mut sorted_tiers = Vec::new(&Env::default());
    for tier in tiers.iter() {
        sorted_tiers.push_back(tier.clone());
    }

    // Find the appropriate tier based on amount
    let mut applicable_tier = &tiers.get(0).unwrap(); // Default to first tier

    for tier in tiers.iter() {
        if amount >= tier.threshold {
            applicable_tier = tier;
        } else {
            break;
        }
    }

    // Calculate fee based on the applicable tier
    match &applicable_tier.fee_model {
        crate::types::FeeModel::Flat(flat_fee) => *flat_fee,
        crate::types::FeeModel::Percentage(percentage_bps) => {
            (amount * (*percentage_bps as i128)) / 10000
        }
        crate::types::FeeModel::Tiered(_) => {
            // Nested tiered fees not supported, fall back to percentage
            (amount * (applicable_tier.default_percentage_bps as i128)) / 10000
        }
    }
}

/// Applies min/max constraints to the calculated fee
fn constrain_fee_amount(calculated_fee: i128, config: &FeeConfig) -> i128 {
    let mut constrained_fee = calculated_fee;

    if let Some(min_fee) = config.min_fee {
        if constrained_fee < min_fee as i128 {
            constrained_fee = min_fee as i128;
        }
    }

    if let Some(max_fee) = config.max_fee {
        if constrained_fee > max_fee as i128 {
            constrained_fee = max_fee as i128;
        }
    }

    constrained_fee
}

/// Calculates the effective fee rate in basis points
fn calculate_effective_rate(gross_amount: i128, fee_amount: i128) -> u32 {
    if gross_amount == 0 {
        return 0;
    }

    // Calculate as (fee_amount * 10000) / gross_amount to get basis points
    ((fee_amount * 10000) / gross_amount) as u32
}

/// Calculates fees for multiple transactions
pub fn calculate_batch_fees(
    env: &Env,
    amounts: &[i128],
    fee_config: &FeeConfig,
) -> Vec<FeeCalculationResult> {
    let mut results = Vec::new(env);

    for &amount in amounts {
        let result = calculate_transaction_fee(env, amount, fee_config);
        results.push_back(result);
    }

    results
}

/// Validates fee configuration
pub fn validate_fee_config(config: &FeeConfig) -> Result<(), ValidationError> {
    // Validate percentage is not over 100% (10000 basis points)
    match &config.fee_model {
        crate::types::FeeModel::Percentage(percentage_bps) => {
            if *percentage_bps > 10000 {
                // More than 100%
                return Err(ValidationError::InvalidPercentage);
            }
        }
        crate::types::FeeModel::Tiered(tiers) => {
            // Validate each tier
            for tier in tiers.iter() {
                if tier.threshold < 0 {
                    return Err(ValidationError::InvalidAmount);
                }

                match &tier.fee_model {
                    crate::types::FeeModel::Percentage(percentage_bps) => {
                        if *percentage_bps > 10000 {
                            // More than 100%
                            return Err(ValidationError::InvalidPercentage);
                        }
                    }
                    _ => {} // Other models have different validation
                }
            }

            // Validate tiers are in ascending order
            let mut prev_threshold = 0i128;
            for tier in tiers.iter() {
                if tier.threshold < prev_threshold {
                    return Err(ValidationError::InvalidAmount); // Reusing error type
                }
                prev_threshold = tier.threshold;
            }
        }
        _ => {}
    }

    // Validate min/max fees
    if let Some(min_fee) = config.min_fee {
        if min_fee > i64::MAX as u64 {
            // Prevent overflow
            return Err(ValidationError::InvalidAmount);
        }
    }

    if let Some(max_fee) = config.max_fee {
        if max_fee > i64::MAX as u64 {
            // Prevent overflow
            return Err(ValidationError::InvalidAmount);
        }
    }

    // Validate min fee is not greater than max fee
    if let (Some(min_fee), Some(max_fee)) = (config.min_fee, config.max_fee) {
        if min_fee > max_fee {
            return Err(ValidationError::InvalidAmount);
        }
    }

    Ok(())
}

/// Stores the fee configuration in persistent storage
pub fn store_fee_config(env: &Env, config: &FeeConfig) -> Result<(), ValidationError> {
    validate_fee_config(config)?;

    env.storage()
        .instance()
        .set(&DataKey::CurrentFeeConfig, config);
    Ok(())
}

/// Retrieves the current fee configuration from storage
pub fn get_current_fee_config(env: &Env) -> Option<FeeConfig> {
    env.storage().instance().get(&DataKey::CurrentFeeConfig)
}

/// Retrieves per-operation fee configuration.
pub fn get_operation_fee_config(env: &Env, operation: &Symbol) -> Option<FeeConfig> {
    env.storage().instance().get(&DataKey::OperationFeeConfig(operation.clone()))
}

/// Stores per-operation fee configuration.
pub fn store_operation_fee_config(env: &Env, operation: &Symbol, config: &FeeConfig) -> Result<(), ValidationError> {
    validate_fee_config(config)?;
    env.storage().instance().set(&DataKey::OperationFeeConfig(operation.clone()), config);
    Ok(())
}

/// Deducts fees from a transaction amount and returns the net amount
pub fn deduct_fees(env: &Env, gross_amount: i128) -> FeeCalculationResult {
    let config = get_current_fee_config(env).unwrap_or(default_fee_config()); // Use default if none configured

    let result = calculate_transaction_fee(env, gross_amount, &config);

    // Emit fee deduction event
    AnalyticsEvents::fee_deducted(
        env,
        result.gross_amount,
        result.fee_amount,
        result.net_amount,
        result.fee_percentage_bps,
    );

    result
}

/// Creates a default fee configuration (0.1% flat fee)
fn default_fee_config() -> FeeConfig {
    use crate::types::FeeModel;

    FeeConfig {
        fee_model: FeeModel::Percentage(10), // 0.1% = 10 basis points
        min_fee: Some(1),                    // Minimum 1 stroop
        max_fee: None,                       // No maximum
        enabled: true,
        description: Some(Symbol::new(&Env::default(), "Default 0.1% fee")),
    }
}

/// Updates fee configuration with validation
pub fn update_fee_config(
    env: &Env,
    admin: &Address,
    new_config: FeeConfig,
) -> Result<(), ValidationError> {
    // Verify caller is admin
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Contract not initialized");

    if *admin != stored_admin {
        return Err(ValidationError::InvalidAddress); // Reusing error type
    }

    validate_fee_config(&new_config)?;
    store_fee_config(env, &new_config)?;

    Ok(())
}

/// Updates per-operation fee configuration (admin only)
pub fn update_operation_fee_config(env: &Env, admin: &Address, operation: &Symbol, new_config: FeeConfig) -> Result<(), ValidationError> {
    // Verify caller is admin
    let stored_admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Contract not initialized");
    if *admin != stored_admin {
        return Err(ValidationError::InvalidAddress);
    }

    // Capture previous for event
    let previous = get_operation_fee_config(env, operation);

    validate_fee_config(&new_config)?;
    store_operation_fee_config(env, operation, &new_config)?;

    // Emit operation fee updated event
    AnalyticsEvents::operation_fee_updated(env, admin, operation, previous, new_config);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FeeModel, FeeTier};
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_flat_fee_calculation() {
        let env = Env::default();
        let config = FeeConfig {
            fee_model: FeeModel::Flat(100), // Fixed fee of 100
            min_fee: None,
            max_fee: None,
            enabled: true,
            description: None,
        };

        let result = calculate_transaction_fee(&env, 1000, &config);
        assert_eq!(result.gross_amount, 1000);
        assert_eq!(result.fee_amount, 100);
        assert_eq!(result.net_amount, 900);
    }

    #[test]
    fn test_percentage_fee_calculation() {
        let env = Env::default();
        let config = FeeConfig {
            fee_model: FeeModel::Percentage(50), // 0.5% = 50 basis points
            min_fee: None,
            max_fee: None,
            enabled: true,
            description: None,
        };

        let result = calculate_transaction_fee(&env, 1000, &config);
        assert_eq!(result.gross_amount, 1000);
        assert_eq!(result.fee_amount, 5); // 0.5% of 1000 = 5
        assert_eq!(result.net_amount, 995);
    }

    #[test]
    fn test_min_max_constraints() {
        let env = Env::default();
        let mut config = FeeConfig {
            fee_model: FeeModel::Percentage(1), // 0.01%
            min_fee: Some(10),
            max_fee: Some(100),
            enabled: true,
            description: None,
        };

        // Test minimum constraint: 50 * 0.01% = 0.005, but min is 10
        let result = calculate_transaction_fee(&env, 50, &config);
        assert_eq!(result.fee_amount, 10); // Minimum applies

        // Test maximum constraint: 1000000 * 0.01% = 100, which equals max
        let result = calculate_transaction_fee(&env, 1000000, &config);
        assert_eq!(result.fee_amount, 100); // Maximum applies
    }

    #[test]
    fn test_zero_negative_amount() {
        let env = Env::default();
        let config = FeeConfig {
            fee_model: FeeModel::Percentage(100), // 1%
            min_fee: None,
            max_fee: None,
            enabled: true,
            description: None,
        };

        let result = calculate_transaction_fee(&env, 0, &config);
        assert_eq!(result.fee_amount, 0);
        assert_eq!(result.net_amount, 0);

        let result = calculate_transaction_fee(&env, -100, &config);
        assert_eq!(result.fee_amount, 0);
        assert_eq!(result.net_amount, -100);
    }

    #[test]
    fn test_tiered_fee_calculation() {
        let env = Env::default();

        // Create a tiered fee structure: 0-100: 1%, 101+: 0.5%
        let mut tiers = Vec::new(&env);
        tiers.push_back(FeeTier {
            threshold: 0,
            fee_model: FeeModel::Percentage(100), // 1%
            default_percentage_bps: 100,
        });
        tiers.push_back(FeeTier {
            threshold: 101,
            fee_model: FeeModel::Percentage(50), // 0.5%
            default_percentage_bps: 50,
        });

        let config = FeeConfig {
            fee_model: FeeModel::Tiered(tiers),
            min_fee: None,
            max_fee: None,
            enabled: true,
            description: None,
        };

        // Amount 50 should use 1% fee = 0.5, rounded down = 0
        let result = calculate_transaction_fee(&env, 50, &config);
        assert_eq!(result.fee_amount, 0); // 50 * 1% = 0.5, floor to 0

        // Amount 200 should use 0.5% fee = 1
        let result = calculate_transaction_fee(&env, 200, &config);
        assert_eq!(result.fee_amount, 1); // 200 * 0.5% = 1
    }

    #[test]
    fn test_fee_constraint_validation() {
        let mut config = FeeConfig {
            fee_model: FeeModel::Percentage(10), // 0.1%
            min_fee: Some(100),
            max_fee: Some(50), // Invalid: min > max
            enabled: true,
            description: None,
        };

        assert!(validate_fee_config(&config).is_err());

        // Valid config
        config.max_fee = Some(150); // Now min (100) < max (150)
        assert!(validate_fee_config(&config).is_ok());
    }

    #[test]
    fn test_percentage_limit_validation() {
        let config = FeeConfig {
            fee_model: FeeModel::Percentage(10001), // Over 100% (10000 bps)
            min_fee: None,
            max_fee: None,
            enabled: true,
            description: None,
        };

        assert!(validate_fee_config(&config).is_err());
    }

    #[test]
    fn test_fee_distribution_splits_correctly() {
        let env = Env::default();
        let r1 = Address::generate(&env);
        let r2 = Address::generate(&env);
        let shares = vec![
            FeeRecipientShare { recipient: r1.clone(), share_bps: 6000 },
            FeeRecipientShare { recipient: r2.clone(), share_bps: 4000 },
        ];
        let shares_vec = Vec::from_array(&env, &shares);
        let fee = 1000i128;
        crate::fees::distribute_fee(&env, fee, &shares_vec);
        let events = env.events().all();
        // Should emit two events
        assert!(events.iter().any(|e| e.topics().contains(&r1)));
        assert!(events.iter().any(|e| e.topics().contains(&r2)));
    }

    #[test]
    fn test_fee_pausing_mechanism() {
        let env = Env::default();
        let admin = Address::generate(&env);
        env.storage().instance().set(&crate::types::DataKey::Admin, &admin);
        // Set a fee config
        let config = FeeConfig {
            fee_model: FeeModel::Percentage(100), // 1%
            min_fee: None,
            max_fee: None,
            enabled: true,
            description: None,
        };
        crate::fees::store_fee_config(&env, &config).unwrap();
        // Not paused: fee should be nonzero
        let result = crate::fees::calculate_transaction_fee(&env, 10000, &config);
        assert_eq!(result.fee_amount, 100);
        // Pause fees
        crate::fees::set_fee_paused(&env, &admin, true).unwrap();
        let result_paused = crate::fees::calculate_transaction_fee(&env, 10000, &config);
        assert_eq!(result_paused.fee_amount, 0);
        // Resume fees
        crate::fees::set_fee_paused(&env, &admin, false).unwrap();
        let result_resumed = crate::fees::calculate_transaction_fee(&env, 10000, &config);
        assert_eq!(result_resumed.fee_amount, 100);
    }
}
}
