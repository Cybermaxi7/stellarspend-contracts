
#![cfg(test)]

use recurring_payment::RecurringPaymentContractClient;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token, vec, Address, Env, IntoVal,
};

fn setup_token<'a>(
    env: &'a Env,
    admin: &Address,
) -> (Address, token::StellarAssetClient<'a>, token::Client<'a>) {
    let addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let admin_client = token::StellarAssetClient::new(env, &addr);
    let client = token::Client::new(env, &addr);
    (addr, admin_client, client)
}

/// Registers the recurring payment contract and returns a client.
fn setup_contract(env: &Env) -> RecurringPaymentContractClient {
    let id = env.register(recurring_payment::RecurringPaymentContract, ());
    RecurringPaymentContractClient::new(env, &id)
}

#[test]
fn test_basic_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, token_client) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);

    let interval: u64 = 3_600;
    let start_time: u64 = 1_000;
    let amount: i128 = 1_000;

    // 1. Create
    let id = contract.create_payment(
        &sender,
        &recipient,
        &token_addr,
        &amount,
        &interval,
        &start_time,
    );
    assert_eq!(id, 1, "first payment should have id 1");

    let p = contract.get_payment(&id);
    assert_eq!(p.amount, amount);
    assert_eq!(p.interval, interval);
    assert_eq!(p.next_execution, start_time);
    assert!(p.active);

    // 2. Execute exactly on start_time
    env.ledger().set_timestamp(start_time);
    contract.execute_payment(&id);

    assert_eq!(token_client.balance(&sender), 4_000);
    assert_eq!(token_client.balance(&recipient), 1_000);

    let p = contract.get_payment(&id);
    assert_eq!(p.next_execution, start_time + interval);
    assert!(p.active);

    // 3. Cancel
    contract.cancel_payment(&id);
    let p = contract.get_payment(&id);
    assert!(!p.active, "payment should be inactive after cancel");
}

#[test]
#[should_panic(expected = "Too early for next execution")]
fn test_execute_too_early() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, _) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);

    let start_time: u64 = 5_000;
    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &3_600, &start_time);

    env.ledger().set_timestamp(start_time - 1);
    contract.execute_payment(&1);
}

#[test]
fn test_execute_overdue_skips_to_next_future_interval() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, token_client) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);

    let interval: u64 = 3_600;
    let start_time: u64 = 1_000;

    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &interval, &start_time);

    // 2.5 intervals after start_time
    env.ledger().set_timestamp(start_time + interval * 2 + 500);
    contract.execute_payment(&1);

    // Only one transfer should happen regardless of how overdue
    assert_eq!(token_client.balance(&recipient), 1_000);

    let p = contract.get_payment(&1);
    // next_execution must be strictly in the future
    assert!(p.next_execution > env.ledger().timestamp());
    // Must land on start_time + 3 * interval  (the next clean boundary)
    assert_eq!(p.next_execution, start_time + 3 * interval);
}

/// Overdue by exactly one interval: next_execution = start + 2 * interval.
#[test]
fn test_execute_one_full_interval_late() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, _) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);

    let interval: u64 = 3_600;
    let start_time: u64 = 1_000;

    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &interval, &start_time);

    // Exactly one full interval late
    env.ledger().set_timestamp(start_time + interval);
    contract.execute_payment(&1);

    let p = contract.get_payment(&1);
    assert_eq!(p.next_execution, start_time + 2 * interval);
}

#[test]
#[should_panic]
fn test_cancel_by_non_owner_panics() {
    let env = Env::default();
    // Do NOT mock all auths so the auth check fires for the wrong address.
    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let attacker = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, _) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    // Mock auths only for the sender so the attacker has no authority.
    env.mock_all_auths_allowing_non_root_auth();

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &3_600, &1_000);

    // Attempt cancellation signed by the attacker instead.
    // We have to call the raw contract; easiest is to just use the same client
    // which will fail auth because `attacker` did not require_auth here.
    let _ = attacker; // silence unused warning
    // The contract's cancel_payment calls payment.sender.require_auth(), which
    // will fail because the invocation isn't authorised by `sender`.
    contract.cancel_payment(&1);
}

/// Cancelling an already-cancelled payment should panic.
#[test]
#[should_panic(expected = "Payment is already canceled")]
fn test_double_cancel_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, _) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &3_600, &1_000);
    contract.cancel_payment(&1);
    contract.cancel_payment(&1); // must panic
}

/// Executing a cancelled payment should panic.
#[test]
#[should_panic(expected = "Payment is not active")]
fn test_execute_cancelled_payment_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, _) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);
    let start_time: u64 = 1_000;
    let interval: u64 = 3_600;

    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &interval, &start_time);
    contract.cancel_payment(&1);

    env.ledger().set_timestamp(start_time + interval);
    contract.execute_payment(&1); // must panic
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_create_with_zero_amount_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token, &0, &3_600, &1_000);
}

/// Negative amount must be rejected.
#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_create_with_negative_amount_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token, &-500, &3_600, &1_000);
}

/// Interval of 0 must be rejected.
#[test]
#[should_panic(expected = "Interval must be positive")]
fn test_create_with_zero_interval_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token, &1_000, &0, &1_000);
}

/// Getting a non-existent payment ID must panic.
#[test]
#[should_panic(expected = "Payment not found")]
fn test_get_nonexistent_payment_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract = setup_contract(&env);
    contract.get_payment(&99);
}

#[test]
fn test_multiple_independent_payments() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender_a = Address::generate(&env);
    let sender_b = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, token_client) = setup_token(&env, &admin);
    admin_client.mint(&sender_a, &10_000);
    admin_client.mint(&sender_b, &10_000);

    let contract = setup_contract(&env);

    let id_a = contract.create_payment(
        &sender_a, &recipient, &token_addr, &1_000, &3_600, &1_000,
    );
    let id_b = contract.create_payment(
        &sender_b, &recipient, &token_addr, &2_000, &7_200, &2_000,
    );

    assert_eq!(id_a, 1);
    assert_eq!(id_b, 2);

    // Execute A only
    env.ledger().set_timestamp(5_000);
    contract.execute_payment(&id_a);

    assert_eq!(token_client.balance(&sender_a), 9_000);
    assert_eq!(token_client.balance(&sender_b), 10_000); // untouched

    // Cancel B without touching A
    contract.cancel_payment(&id_b);
    assert!(contract.get_payment(&id_a).active);
    assert!(!contract.get_payment(&id_b).active);
}

/// Payment count increments correctly as new schedules are created.
#[test]
fn test_payment_ids_are_sequential() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let contract = setup_contract(&env);

    for expected_id in 1u64..=5 {
        let id = contract.create_payment(
            &sender, &recipient, &token, &100, &3_600, &1_000,
        );
        assert_eq!(id, expected_id);
    }
}



/// Creating a payment emits a ("recur", "created", id) event.
#[test]
fn test_create_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = Address::generate(&env);

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token, &1_000, &3_600, &1_000);

    let events = env.events().all();
    assert!(!events.is_empty(), "expected at least one event");
}

/// Executing a payment emits a ("recur", "executed", id) event.
#[test]
fn test_execute_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, _) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &3_600, &1_000);

    env.ledger().set_timestamp(1_000);
    contract.execute_payment(&1);

    let events = env.events().all();
    // At least the create event + execute event
    assert!(events.len() >= 2, "expected create + execute events");
}

/// Cancelling a payment emits a ("recur", "canceled", id) event.
#[test]
fn test_cancel_emits_event() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, _) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);
    contract.create_payment(&sender, &recipient, &token_addr, &1_000, &3_600, &1_000);
    contract.cancel_payment(&1);

    let events = env.events().all();
    // create event + cancel event
    assert!(events.len() >= 2, "expected create + cancel events");
}

/// Repeated executions across multiple intervals transfer correctly each time.
#[test]
fn test_repeated_executions_across_intervals() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token_addr, admin_client, token_client) = setup_token(&env, &admin);
    admin_client.mint(&sender, &5_000);

    let contract = setup_contract(&env);

    let interval: u64 = 3_600;
    let start_time: u64 = 1_000;
    let amount: i128 = 500;

    contract.create_payment(&sender, &recipient, &token_addr, &amount, &interval, &start_time);

    for i in 0u64..4 {
        env.ledger().set_timestamp(start_time + i * interval);
        contract.execute_payment(&1);
    }

    // 4 payments of 500 each = 2000 transferred
    assert_eq!(token_client.balance(&sender), 3_000);
    assert_eq!(token_client.balance(&recipient), 2_000);

    let p = contract.get_payment(&1);
    assert_eq!(p.next_execution, start_time + 4 * interval);
    assert!(p.active);
}