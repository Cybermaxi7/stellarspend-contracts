// Simulation tests for read-only transaction validation and outcome projection.

use contracts::simulation::{simulate_transaction, SimulationError};
use contracts::transactions::{Transaction, TransactionOutcome};

#[test]
fn test_simulation_valid_transaction() {
    let tx = Transaction::mock_valid();
    let result = simulate_transaction(&tx);
    assert!(result.is_ok());
    let outcome = result.unwrap();
    assert_eq!(outcome, tx.project_outcome());
}

#[test]
fn test_simulation_invalid_transaction() {
    let tx = Transaction::mock_invalid();
    let result = simulate_transaction(&tx);
    assert_eq!(result, Err(SimulationError::InvalidParameters));
}
