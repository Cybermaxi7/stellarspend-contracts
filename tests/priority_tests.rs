use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, Symbol,
};

#[path = "../contracts/priority.rs"]
mod priority;

use priority::{PendingItem, PriorityContract, PriorityContractClient};

fn setup() -> (Env, Address, PriorityContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PriorityContract, ());
    let client = PriorityContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, client)
}

#[test]
fn test_priority_order() {
    let (env, _admin, client) = setup();

    let caller = Address::generate(&env);

    client.enqueue(&caller, &symbol_short!("low1"), &0u32);
    client.enqueue(&caller, &symbol_short!("med1"), &1u32);
    client.enqueue(&caller, &symbol_short!("high1"), &2u32);

    let first: PendingItem = client.dequeue().expect("expected item");
    assert_eq!(first.priority, 2);

    let second: PendingItem = client.dequeue().expect("expected item");
    assert_eq!(second.priority, 1);

    let third: PendingItem = client.dequeue().expect("expected item");
    assert_eq!(third.priority, 0);
}

#[test]
fn test_starvation_prevention() {
    let (env, _admin, client) = setup();

    let caller = Address::generate(&env);

    // Enqueue a low-priority item first (it will age)
    client.enqueue(&caller, &symbol_short!("low_old"), &0u32);

    // Advance time so low item becomes older
    env.ledger().set_timestamp(env.ledger().timestamp() + 120);

    // Enqueue several high-priority items that are fresh
    client.enqueue(&caller, &symbol_short!("high_fresh1"), &2u32);
    client.enqueue(&caller, &symbol_short!("high_fresh2"), &2u32);

    // Now dequeue: starvation prevention should return the old low-priority item first
    let popped: PendingItem = client.dequeue().expect("expected item");
    assert_eq!(popped.priority, 0);
    assert_eq!(popped.payload, symbol_short!("low_old"));
}
