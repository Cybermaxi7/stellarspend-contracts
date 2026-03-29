#![no_std]

use soroban_sdk::{
    contracttype, symbol_short, Bytes, Env, Map, String, Symbol, Vec,
};

/// Maximum metadata size in bytes (1KB)
const MAX_METADATA_SIZE: u32 = 1024;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TransactionMetadata(Bytes),
}

#[contracttype]
#[derive(Clone)]
pub struct TransactionMetadata {
    pub data: Map<Symbol, String>,
}

pub struct TransactionMetadataContract;

impl TransactionMetadataContract {
    /// Store structured metadata for a transaction
    pub fn set_metadata(
        env: Env,
        transaction_id: Bytes,
        metadata: Map<Symbol, String>,
    ) {
        Self::validate_metadata(&env, &metadata);

        let key = DataKey::TransactionMetadata(transaction_id.clone());

        let value = TransactionMetadata {
            data: metadata.clone(),
        };

        env.storage().persistent().set(&key, &value);

        // Emit metadata event
        env.events().publish(
            (symbol_short!("metadata_set"), transaction_id),
            metadata,
        );
    }

    /// Retrieve stored metadata
    pub fn get_metadata(
        env: Env,
        transaction_id: Bytes,
    ) -> Option<TransactionMetadata> {
        let key = DataKey::TransactionMetadata(transaction_id);
        env.storage().persistent().get(&key)
    }

    /// Validate metadata size and structure
    fn validate_metadata(env: &Env, metadata: &Map<Symbol, String>) {
        let mut total_size: u32 = 0;

        let keys: Vec<Symbol> = metadata.keys();

        for key in keys.iter() {
            let value = metadata.get(key).unwrap();

            total_size += key.to_string().len() as u32;
            total_size += value.len() as u32;
        }

        if total_size > MAX_METADATA_SIZE {
            panic!("Metadata exceeds maximum allowed size");
        }
    }
}