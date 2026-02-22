//! Validation for payment reminder requests: users and due dates.

use soroban_sdk::{Address, Env};

/// Validates a single reminder request (user and due date).
///
/// # Returns
/// * `Ok(())` if valid
/// * `Err(ValidationError)` if invalid
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
    InvalidUser,
    InvalidDueDate,
}

pub fn validate_reminder_request(
    env: &Env,
    user: &Address,
    due_date: u64,
) -> Result<(), ValidationError> {
    if !is_valid_user(user) {
        return Err(ValidationError::InvalidUser);
    }
    if !is_valid_due_date(env, due_date) {
        return Err(ValidationError::InvalidDueDate);
    }
    Ok(())
}

/// User address must be valid (Soroban addresses are valid by construction; stub for consistency).
fn is_valid_user(_user: &Address) -> bool {
    true
}

/// Due date must be in the future and not more than ~5 years ahead.
fn is_valid_due_date(env: &Env, due_date: u64) -> bool {
    let current = env.ledger().sequence() as u64;
    if due_date <= current {
        return false;
    }
    const MAX_FUTURE_LEDGERS: u64 = 31_536_000; // ~5 years at ~5s/ledger
    if due_date > current + MAX_FUTURE_LEDGERS {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_valid_due_date() {
        let env = Env::default();
        let current = env.ledger().sequence() as u64;
        assert!(is_valid_due_date(&env, current + 1));
        assert!(is_valid_due_date(&env, current + 1000));
        assert!(!is_valid_due_date(&env, current));
        assert!(!is_valid_due_date(&env, current.saturating_sub(1)));
    }

    #[test]
    fn test_validate_reminder_request_ok() {
        let env = Env::default();
        let user = Address::generate(&env);
        let due_date = env.ledger().sequence() as u64 + 100;
        assert_eq!(validate_reminder_request(&env, &user, due_date), Ok(()));
    }

    #[test]
    fn test_validate_reminder_request_invalid_due_date() {
        let env = Env::default();
        let user = Address::generate(&env);
        let due_date = env.ledger().sequence() as u64;
        assert_eq!(
            validate_reminder_request(&env, &user, due_date),
            Err(ValidationError::InvalidDueDate)
        );
    }
}
