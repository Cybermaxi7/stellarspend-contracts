#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

use stellarspend_contract::{StellarSpendContract, StellarSpendContractClient};

fn setup() -> (Env, StellarSpendContractClient<'static>) {
    let env = Env::default();
    let contract_id = env.register(StellarSpendContract, ());
    let client = StellarSpendContractClient::new(&env, &contract_id);

    (env, client)
}

#[test]
fn test_full_stellarspend_workflow() {
    let (env, client) = setup();

    // Simulate wallet creation
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Fund users (mock initial balance)
    client.create_wallet(&user1, &1000);
    client.create_wallet(&user2, &500);

    // Validate wallet creation
    assert_eq!(client.get_balance(&user1), 1000);
    assert_eq!(client.get_balance(&user2), 500);

    // Execute transfer
    client.transfer(&user1, &user2, &200);

    assert_eq!(client.get_balance(&user1), 800);
    assert_eq!(client.get_balance(&user2), 700);

    // Create budget
    client.create_budget(&user1, &"food".into(), &300);

    let remaining_budget = client.get_budget(&user1, &"food".into());
    assert_eq!(remaining_budget, 300);

    // Spend from budget
    client.spend_from_budget(&user1, &"food".into(), &100);
    assert_eq!(client.get_budget(&user1, &"food".into()), 200);

    // Create savings
    client.create_savings(&user1, &"vacation".into(), &200);

    assert_eq!(client.get_savings(&user1, &"vacation".into()), 200);
}
