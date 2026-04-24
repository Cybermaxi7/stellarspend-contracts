use alloc::format;
use soroban_sdk::{Env, String};
use crate::storage::MAX_FEE_BPS;

/// Round up division for i128 values.
pub fn round_up_div(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator == 0 {
        return None;
    }
    numerator
        .checked_add(denominator - 1)?
        .checked_div(denominator)
}

/// Round down division for i128 values.
pub fn round_down_div(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator == 0 {
        return None;
    }
    numerator.checked_div(denominator)
}

/// Calculate fee with rounding up. Returns None on overflow or invalid bps.
pub fn calculate_fee_round_up(amount: i128, fee_bps: u32) -> Option<i128> {
    if amount <= 0 || fee_bps == 0 {
        return Some(0);
    }
    if fee_bps > MAX_FEE_BPS {
        return None;
    }
    let numerator = amount.checked_mul(fee_bps as i128)?;
    round_up_div(numerator, 10_000)
}

/// Calculate fee with rounding down. Returns None on overflow or invalid bps.
pub fn calculate_fee_round_down(amount: i128, fee_bps: u32) -> Option<i128> {
    if amount <= 0 || fee_bps == 0 {
        return Some(0);
    }
    if fee_bps > MAX_FEE_BPS {
        return None;
    }
    let numerator = amount.checked_mul(fee_bps as i128)?;
    round_down_div(numerator, 10_000)
}

/// Alias matching main's API for compatibility.
pub fn compute_fee(amount: i128, bps: u32) -> Option<i128> {
    calculate_fee_round_down(amount, bps)
}

/// Format an amount into a canonical decimal string.
pub fn format_amount(env: &Env, amount: i128) -> String {
    let formatted = format!("{amount}");
    String::from_str(env, formatted.as_str())
}