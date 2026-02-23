#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env,
};

#[path = "../contracts/rewards.rs"]
mod rewards;

use rewards::{RewardsContract, RewardsContractClient};

fn setup_rewards_contract() -> (Env, Address, RewardsContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RewardsContract, ());
    let client = RewardsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    
    // Base reward = 100, multiplier = 50
    client.initialize(&admin, &100, &50);

    (env, admin, client)
}

#[test]
fn test_reward_initialization() {
    let (env, admin, client) = setup_rewards_contract();

    // Just verifying no panic during setup
    assert_eq!(client.calculate_reward(&1), 150);
}

#[test]
fn test_reward_calculation() {
    let (_env, _admin, client) = setup_rewards_contract();

    // formula = base + (multiplier * milestone_id)
    assert_eq!(client.calculate_reward(&1), 150); // 100 + (50 * 1)
    assert_eq!(client.calculate_reward(&5), 350); // 100 + (50 * 5)
    assert_eq!(client.calculate_reward(&10), 600); // 100 + (50 * 10)
}

#[test]
fn test_distribute_reward_success() {
    let (env, admin, client) = setup_rewards_contract();
    let user = Address::generate(&env);
    let milestone_id = 1;

    let reward_amount = client.distribute_reward(&admin, &user, &milestone_id);
    assert_eq!(reward_amount, 150);
    assert!(client.has_user_been_rewarded(&user, &milestone_id));

    // Check events
    let events = env.events().all();
    let distributed_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("issued") == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(distributed_events, 1);
}

#[test]
#[should_panic]
fn test_duplicate_reward_prevention() {
    let (env, admin, client) = setup_rewards_contract();
    let user = Address::generate(&env);
    let milestone_id = 1;

    // First time should succeed
    client.distribute_reward(&admin, &user, &milestone_id);

    // Second time for same user and milestone should panic
    client.distribute_reward(&admin, &user, &milestone_id);
}

#[test]
#[should_panic]
fn test_invalid_milestone() {
    let (env, admin, client) = setup_rewards_contract();
    let user = Address::generate(&env);
    
    // milestone_id = 0 is invalid
    let milestone_id = 0;

    client.distribute_reward(&admin, &user, &milestone_id);
}

#[test]
#[should_panic]
fn test_unauthorized_distribution() {
    let (env, _admin, client) = setup_rewards_contract();
    let unauthorized_user = Address::generate(&env);
    let target_user = Address::generate(&env);
    let milestone_id = 1;

    client.distribute_reward(&unauthorized_user, &target_user, &milestone_id);
}
