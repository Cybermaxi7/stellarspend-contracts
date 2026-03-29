#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, String, Vec, U256,
};

#[path = "../contracts/archive.rs"]
mod archive;

use archive::history::{TransactionRecord, TransactionStatus, TransactionType};
use archive::{ArchiveContract, ArchiveContractClient, ArchiveError, CompressedArchive};

fn setup_archive_contract() -> (Env, Address, ArchiveContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ArchiveContract, ());
    let client = ArchiveContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &86400); // 1 day threshold

    (env, admin, client)
}

#[test]
fn test_archive_initialization() {
    let (env, admin, client) = setup_archive_contract();

    client.set_threshold(&admin, &10000);
}

#[test]
fn test_archive_records_success() {
    let (env, admin, client) = setup_archive_contract();

    // Set threshold to 1 so cutoff is 0 (since default ledger timestamp is 0)
    // 0 saturating_sub 1 = 0
    client.set_threshold(&admin, &1);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let mut records = Vec::new(&env);
    for i in 1..=5 {
        records.push_back(TransactionRecord {
            id: U256::from_u32(&env, i),
            from: from.clone(),
            to: to.clone(),
            amount: 100,
            timestamp: 0,
            description: String::from_str(&env, "Test"),
            transaction_type: TransactionType::Payment,
            block_number: 1,
            status: TransactionStatus::Completed,
        });
    }

    let batch_id = client.archive_records(&admin, &records);
    assert_eq!(batch_id, 1);

    let batch = client
        .get_archive_batch(&batch_id)
        .expect("batch should exist");
    assert_eq!(batch.batch_id, 1);
    assert_eq!(batch.records.len(), 5);

    // Check events
    let events = env.events().all();
    let archived_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("batched")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(archived_events, 1);
}

#[test]
#[should_panic]
fn test_archive_records_too_new() {
    let (env, admin, client) = setup_archive_contract();

    // the max timestamp possible
    client.set_threshold(&admin, &100);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    let mut records = Vec::new(&env);
    records.push_back(TransactionRecord {
        id: U256::from_u32(&env, 1),
        from: from.clone(),
        to: to.clone(),
        amount: 100,
        // Since ledger timestamp is 0 by default, cutoff is 0
        // record has timestamp 100 which is > 0, so it will be filtered out.
        timestamp: 100,
        description: String::from_str(&env, "Test"),
        transaction_type: TransactionType::Payment,
        block_number: 1,
        status: TransactionStatus::Completed,
    });

    client.archive_records(&admin, &records);
}
