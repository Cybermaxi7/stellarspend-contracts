/// Utility functions for fee calculations with consistent rounding behavior.

/// Round up division for i128 values.
/// Returns (numerator + denominator - 1) / denominator
/// This ensures we always round up to avoid precision loss in fee calculations.
pub fn round_up_div(numerator: i128, denominator: i128) -> i128 {
    if denominator == 0 {
        return 0;
    }
    (numerator + denominator - 1) / denominator
}

/// Round down division for i128 values.
/// Returns numerator / denominator (standard integer division)
pub fn round_down_div(numerator: i128, denominator: i128) -> i128 {
    if denominator == 0 {
        return 0;
    }
    numerator / denominator
}

/// Calculate fee amount with rounding up.
/// fee = (amount * fee_bps + 10000 - 1) / 10000
/// This ensures fees are always rounded up to avoid precision loss.
pub fn calculate_fee_round_up(amount: i128, fee_bps: u32) -> i128 {
    if amount <= 0 || fee_bps == 0 {
        return 0;
    }
    
    let numerator = amount * fee_bps as i128;
    round_up_div(numerator, 10_000)
}

/// Calculate fee amount with rounding down.
/// fee = (amount * fee_bps) / 10000
pub fn calculate_fee_round_down(amount: i128, fee_bps: u32) -> i128 {
    if amount <= 0 || fee_bps == 0 {
        return 0;
    }
    
    let numerator = amount * fee_bps as i128;
    round_down_div(numerator, 10_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_up_div_basic() {
        assert_eq!(round_up_div(100, 10), 10);
        assert_eq!(round_up_div(101, 10), 11);
        assert_eq!(round_up_div(99, 10), 10);
        assert_eq!(round_up_div(1, 10), 1);
    }

    #[test]
    fn test_round_up_div_zero_denominator() {
        assert_eq!(round_up_div(100, 0), 0);
    }

    #[test]
    fn test_round_down_div_basic() {
        assert_eq!(round_down_div(100, 10), 10);
        assert_eq!(round_down_div(101, 10), 10);
        assert_eq!(round_down_div(99, 10), 9);
        assert_eq!(round_down_div(1, 10), 0);
    }

    #[test]
    fn test_round_down_div_zero_denominator() {
        assert_eq!(round_down_div(100, 0), 0);
    }

    #[test]
    fn test_calculate_fee_round_up() {
        // 1000 * 500 bps / 10000 = 50
        assert_eq!(calculate_fee_round_up(1000, 500), 50);
        
        // 100 * 500 bps / 10000 = 5
        assert_eq!(calculate_fee_round_up(100, 500), 5);
        
        // 1 * 500 bps / 10000 = 0.05, rounds up to 1
        assert_eq!(calculate_fee_round_up(1, 500), 1);
        
        // 99 * 500 bps / 10000 = 4.95, rounds up to 5
        assert_eq!(calculate_fee_round_up(99, 500), 5);
    }

    #[test]
    fn test_calculate_fee_round_down() {
        // 1000 * 500 bps / 10000 = 50
        assert_eq!(calculate_fee_round_down(1000, 500), 50);
        
        // 100 * 500 bps / 10000 = 5
        assert_eq!(calculate_fee_round_down(100, 500), 5);
        
        // 1 * 500 bps / 10000 = 0.05, rounds down to 0
        assert_eq!(calculate_fee_round_down(1, 500), 0);
        
        // 99 * 500 bps / 10000 = 4.95, rounds down to 4
        assert_eq!(calculate_fee_round_down(99, 500), 4);
    }

    #[test]
    fn test_calculate_fee_zero_cases() {
        assert_eq!(calculate_fee_round_up(0, 500), 0);
        assert_eq!(calculate_fee_round_up(1000, 0), 0);
        assert_eq!(calculate_fee_round_down(0, 500), 0);
        assert_eq!(calculate_fee_round_down(1000, 0), 0);
    }

    #[test]
    fn test_rounding_consistency() {
        // Round up should always be >= round down
        for amount in [1, 10, 50, 99, 100, 1000] {
            for fee_bps in [100, 250, 500, 1000] {
                let rounded_up = calculate_fee_round_up(amount, fee_bps);
                let rounded_down = calculate_fee_round_down(amount, fee_bps);
                assert!(
                    rounded_up >= rounded_down,
                    "Round up should be >= round down for amount={}, fee_bps={}",
                    amount,
                    fee_bps
                );
            }
        }
    }
}
