//! Validation logic for batch token minting.

use soroban_sdk::Address;

use crate::types::{ErrorCode, TokenMintRequest, MAX_MINT_AMOUNT, MIN_MINT_AMOUNT};

/// Validates a token mint request.
///
/// # Arguments
/// * `request` - The mint request to validate
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err(error_code)` if invalid
pub fn validate_mint_request(request: &TokenMintRequest) -> Result<(), u32> {
    // Validate recipient address is valid (always true by construction in Soroban)
    if !is_valid_recipient(&request.recipient) {
        return Err(ErrorCode::INVALID_RECIPIENT);
    }

    // Validate mint amount
    if !is_valid_amount(request.amount) {
        return Err(ErrorCode::INVALID_AMOUNT);
    }

    Ok(())
}

/// Validates that a recipient address is valid.
///
/// In Soroban, all Address instances are valid by construction.
/// This function exists for consistency with validation patterns.
fn is_valid_recipient(_recipient: &Address) -> bool {
    // Address is always valid in Soroban SDK by construction
    true
}

/// Validates that a mint amount is within acceptable bounds.
///
/// # Arguments
/// * `amount` - The amount to validate
///
/// # Returns
/// * `true` if amount is >= MIN_MINT_AMOUNT and <= MAX_MINT_AMOUNT
pub fn is_valid_amount(amount: i128) -> bool {
    amount >= MIN_MINT_AMOUNT && amount <= MAX_MINT_AMOUNT
}

/// Validates a batch of mint requests has valid structure.
///
/// # Arguments
/// * `requests` - The batch of mint requests
///
/// # Returns
/// * `Ok(())` if batch is valid
/// * `Err(())` if batch is malformed
pub fn validate_batch(requests: &soroban_sdk::Vec<TokenMintRequest>) -> Result<(), ()> {
    // Check that batch is not empty
    if requests.is_empty() {
        return Err(());
    }

    // Basic structural validation
    // Individual validation happens during processing
    Ok(())
}

/// Validates a token address.
///
/// # Arguments
/// * `token` - The token address to validate
///
/// # Returns
/// * `true` if token address is valid
pub fn is_valid_token_address(_token: &Address) -> bool {
    // Address is always valid by construction
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn create_valid_request(env: &Env) -> TokenMintRequest {
        TokenMintRequest {
            recipient: Address::generate(env),
            amount: 100_000_000, // 0.1 XLM in stroops
        }
    }

    #[test]
    fn test_valid_mint_request() {
        let env = Env::default();
        let request = create_valid_request(&env);
        assert!(validate_mint_request(&request).is_ok());
    }

    #[test]
    fn test_invalid_amount_zero() {
        let env = Env::default();
        let mut request = create_valid_request(&env);
        request.amount = 0;
        assert_eq!(validate_mint_request(&request), Err(ErrorCode::INVALID_AMOUNT));
    }

    #[test]
    fn test_invalid_amount_negative() {
        let env = Env::default();
        let mut request = create_valid_request(&env);
        request.amount = -1000;
        assert_eq!(validate_mint_request(&request), Err(ErrorCode::INVALID_AMOUNT));
    }

    #[test]
    fn test_invalid_amount_too_large() {
        let env = Env::default();
        let mut request = create_valid_request(&env);
        request.amount = MAX_MINT_AMOUNT + 1;
        assert_eq!(validate_mint_request(&request), Err(ErrorCode::INVALID_AMOUNT));
    }

    #[test]
    fn test_is_valid_amount() {
        assert!(is_valid_amount(MIN_MINT_AMOUNT));
        assert!(is_valid_amount(MAX_MINT_AMOUNT));
        assert!(is_valid_amount(100_000_000));
        assert!(!is_valid_amount(0));
        assert!(!is_valid_amount(-1000));
        assert!(!is_valid_amount(MAX_MINT_AMOUNT + 1));
    }

    #[test]
    fn test_validate_batch() {
        let env = Env::default();
        let mut requests: soroban_sdk::Vec<TokenMintRequest> = soroban_sdk::Vec::new(&env);
        
        assert!(validate_batch(&requests).is_err());
        
        requests.push_back(create_valid_request(&env));
        assert!(validate_batch(&requests).is_ok());
    }
}
