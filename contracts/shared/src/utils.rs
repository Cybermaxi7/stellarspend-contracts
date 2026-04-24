/// Shared validation errors for simple reusable helpers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    NegativeAmount,
}

/// Validates that an amount is not negative.
pub fn validate_amount(amount: i128) -> Result<(), ValidationError> {
    if amount < 0 {
        Err(ValidationError::NegativeAmount)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_amount, ValidationError};

    #[test]
    fn accepts_zero_and_positive_amounts() {
        assert_eq!(validate_amount(0), Ok(()));
        assert_eq!(validate_amount(1), Ok(()));
        assert_eq!(validate_amount(1_000_000), Ok(()));
    }

    #[test]
    fn rejects_negative_amounts() {
        assert_eq!(validate_amount(-1), Err(ValidationError::NegativeAmount));
        assert_eq!(validate_amount(-99), Err(ValidationError::NegativeAmount));
    }
}
