//! Simulation module for read-only transaction validation and outcome projection.

use crate::transactions::{Transaction, TransactionOutcome};

/// Simulate a transaction without mutating state or writing to the ledger.
pub fn simulate_transaction(tx: &Transaction) -> Result<TransactionOutcome, SimulationError> {
    // Validate parameters
    if !tx.is_valid() {
        return Err(SimulationError::InvalidParameters);
    }
    // Project outcome (read-only)
    let outcome = tx.project_outcome();
    Ok(outcome)
}

#[derive(Debug, PartialEq)]
pub enum SimulationError {
    InvalidParameters,
    // Add more error types as needed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transactions::{Transaction, TransactionOutcome};

    #[test]
    fn test_simulate_valid_transaction() {
        let tx = Transaction::mock_valid();
        let result = simulate_transaction(&tx);
        assert!(result.is_ok());
        let outcome = result.unwrap();
        assert_eq!(outcome, tx.project_outcome());
    }

    #[test]
    fn test_simulate_invalid_transaction() {
        let tx = Transaction::mock_invalid();
        let result = simulate_transaction(&tx);
        assert_eq!(result, Err(SimulationError::InvalidParameters));
    }
}
