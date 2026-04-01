#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

use crate::{FeeContract, FeeContractClient};

fn setup() -> (Env, Address, FeeContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let treasury = Address::generate(&env);
    let contract_id = env.register(FeeContract, ());
    let client = FeeContractClient::new(&env, &contract_id);
    client.initialize(&admin, &token, &treasury, &500u32, &1u64);
    (env, admin, client)
}

#[test]
fn test_set_user_tier_valid() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);
    let tier = Symbol::new(&env, "gold");

    client.set_user_tier(&admin, &user, &tier);

    let stored = client.get_user_tier(&user).unwrap();
    assert_eq!(stored, tier);
}

#[test]
fn test_set_user_tier_all_valid_tiers() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    for name in ["bronze", "silver", "gold", "platinum"] {
        let tier = Symbol::new(&env, name);
        client.set_user_tier(&admin, &user, &tier);
        assert_eq!(client.get_user_tier(&user).unwrap(), tier);
    }
}

#[test]
#[should_panic]
fn test_set_user_tier_invalid_tier_panics() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);
    let bad_tier = Symbol::new(&env, "diamond");
    client.set_user_tier(&admin, &user, &bad_tier);
}

#[test]
#[should_panic]
fn test_set_user_tier_unauthorized_panics() {
    let (env, _admin, client) = setup();
    let non_admin = Address::generate(&env);
    let user = Address::generate(&env);
    let tier = Symbol::new(&env, "silver");
    client.set_user_tier(&non_admin, &user, &tier);
}

#[test]
fn test_remove_user_tier() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);
    let tier = Symbol::new(&env, "platinum");

    client.set_user_tier(&admin, &user, &tier);
    assert!(client.get_user_tier(&user).is_some());

    client.remove_user_tier(&admin, &user);
    assert!(client.get_user_tier(&user).is_none());
}

#[test]
fn test_remove_user_tier_no_tier_is_noop() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);
    // Should not panic even if user has no tier
    client.remove_user_tier(&admin, &user);
    assert!(client.get_user_tier(&user).is_none());
}

#[test]
#[should_panic]
fn test_remove_user_tier_unauthorized_panics() {
    let (env, admin, client) = setup();
    let non_admin = Address::generate(&env);
    let user = Address::generate(&env);
    let tier = Symbol::new(&env, "bronze");
    client.set_user_tier(&admin, &user, &tier);
    client.remove_user_tier(&non_admin, &user);
}

#[test]
fn test_get_user_tier_returns_none_when_unset() {
    let (env, _admin, client) = setup();
    let user = Address::generate(&env);
    assert!(client.get_user_tier(&user).is_none());
}

#[test]
fn test_tier_can_be_overwritten() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);

    client.set_user_tier(&admin, &user, &Symbol::new(&env, "bronze"));
    client.set_user_tier(&admin, &user, &Symbol::new(&env, "gold"));

    assert_eq!(
        client.get_user_tier(&user).unwrap(),
        Symbol::new(&env, "gold")
    );
}
