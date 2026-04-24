use soroban_sdk::{contractimpl, Env, Address, Symbol};

pub struct TransactionContract;

#[contractimpl]
impl TransactionContract {
    /// Create a transaction and emit an event
    pub fn create_transaction(env: Env, from: Address, to: Address, amount: i128) {
        // Store transaction data (simplified example)
        let tx_key = format!("tx:{}:{}", from, to);
        env.storage().set(&tx_key, &amount);

        // Emit event for off-chain listeners
        env.events().publish(
            (Symbol::short("transaction_created"),),
            (from, to, amount),
        );
    }
}
