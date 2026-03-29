#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env,
};

#[path = "../contracts/snapshots.rs"]
mod snapshots;

use snapshots::{SnapshotsContract, SnapshotsContractClient};

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        timestamp: 1_700_000_000,
        protocol_version: 22,
        sequence_number: 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 6_312_000,
    });
    env
}

fn deploy(env: &Env) -> (SnapshotsContractClient<'_>, Address) {
    let admin = Address::generate(env);
    let contract_id = env.register(SnapshotsContract, ());
    let client = SnapshotsContractClient::new(env, &contract_id);
    (client, admin)
}

#[test]
fn test_initialize() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    assert_eq!(client.get_admin(), Some(admin.clone()));
    assert_eq!(client.get_period_duration(), Some(86400));
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_double_initialize_fails() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    client.initialize(&admin, &3600_u64);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_initialize_zero_period_fails() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &0_u64);
}

#[test]
fn test_create_snapshot_and_retrieve_by_timestamp() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    let user = Address::generate(&env);
    let period_start = 0u64;
    client.create_snapshot(&admin, &user, &1000_i128, &period_start);
    let at_ts = 1000u64;
    let snap = client.get_snapshot(&user, &at_ts).unwrap();
    assert_eq!(snap.user, user);
    assert_eq!(snap.amount, 1000);
    assert_eq!(snap.period_start, period_start);
    assert_eq!(snap.created_at, 1_700_000_000);
}

#[test]
fn test_get_snapshot_at_period() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &3600_u64);
    let user = Address::generate(&env);
    let period_start = 0u64;
    client.create_snapshot(&admin, &user, &500_i128, &period_start);
    let snap = client.get_snapshot_at_period(&user, &period_start).unwrap();
    assert_eq!(snap.amount, 500);
    assert_eq!(snap.period_start, period_start);
}

#[test]
fn test_get_snapshot_none_for_other_period() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    let user = Address::generate(&env);
    let period_start = 0u64;
    client.create_snapshot(&admin, &user, &1000_i128, &period_start);
    let other_ts = 100_000_000u64;
    let snap = client.get_snapshot(&user, &other_ts);
    assert!(snap.is_none());
}

#[test]
fn test_snapshot_event_emitted() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    let user = Address::generate(&env);
    let period_start = 0u64;
    client.create_snapshot(&admin, &user, &2000_i128, &period_start);
    let snap = client.get_snapshot_at_period(&user, &period_start).unwrap();
    assert_eq!(snap.amount, 2000);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_create_snapshot_unauthorized() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    let user = Address::generate(&env);
    let other = Address::generate(&env);
    client.create_snapshot(&other, &user, &1000_i128, &1_699_977_600_u64);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_create_snapshot_unaligned_period_fails() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    let user = Address::generate(&env);
    let unaligned = 1_699_977_600u64 + 100;
    client.create_snapshot(&admin, &user, &1000_i128, &unaligned);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_create_snapshot_future_period_fails() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    let user = Address::generate(&env);
    let future = 2_000_000_000u64;
    client.create_snapshot(&admin, &user, &1000_i128, &future);
}

#[test]
fn test_overwrite_snapshot_same_period() {
    let env = setup_env();
    let (client, admin) = deploy(&env);
    client.initialize(&admin, &86400_u64);
    let user = Address::generate(&env);
    let period_start = 0u64;
    client.create_snapshot(&admin, &user, &1000_i128, &period_start);
    client.create_snapshot(&admin, &user, &2000_i128, &period_start);
    let snap = client.get_snapshot_at_period(&user, &period_start).unwrap();
    assert_eq!(snap.amount, 2000);
}
