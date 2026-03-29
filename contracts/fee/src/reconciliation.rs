use soroban_sdk::{contracttype, Env};

use crate::storage::{read_escrow_balance, read_total_collected, read_total_released};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReconciliationResult {
    pub stored_balance: i128,
    pub calculated_balance: i128,
    pub discrepancy: i128,
    pub is_reconciled: bool,
}

/// Compare the stored escrow balance against the calculated balance
/// (total_collected - total_released). Returns a result describing any
/// discrepancy between the two values.
pub fn reconcile(env: &Env) -> ReconciliationResult {
    let stored_balance = read_escrow_balance(env);
    let total_collected = read_total_collected(env);
    let total_released = read_total_released(env);

    let calculated_balance = total_collected - total_released;
    let discrepancy = stored_balance - calculated_balance;

    ReconciliationResult {
        stored_balance,
        calculated_balance,
        discrepancy,
        is_reconciled: discrepancy == 0,
    }
}
