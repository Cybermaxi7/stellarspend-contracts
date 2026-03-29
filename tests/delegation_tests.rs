#![cfg(test)]

#[path = "../contracts/delegation.rs"]
mod delegation;

use delegation::{DelegationContract, DelegationContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_successful_delegation() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let delegate = Address::generate(&env);
    let limit = 1000;

    let client =
        DelegationContractClient::new(&env, &env.register_contract(None, DelegationContract));
    client.set_delegation(&owner, &delegate, &limit);

    let delegation = client.get_delegation(&owner, &delegate).unwrap();
    assert_eq!(delegation.limit, limit);
    assert_eq!(delegation.spent, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_delegate_to_self_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let limit = 1000;

    let client =
        DelegationContractClient::new(&env, &env.register_contract(None, DelegationContract));
    client.set_delegation(&owner, &owner, &limit);
}

#[test]
fn test_revoke_delegation() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let delegate = Address::generate(&env);
    let limit = 1000;

    let client =
        DelegationContractClient::new(&env, &env.register_contract(None, DelegationContract));
    client.set_delegation(&owner, &delegate, &limit);

    // Revoke
    client.revoke_delegation(&owner, &delegate);

    let delegation = client.get_delegation(&owner, &delegate);
    assert!(delegation.is_none());
}

#[test]
fn test_spend_within_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let delegate = Address::generate(&env);
    let limit = 1000;

    let client =
        DelegationContractClient::new(&env, &env.register_contract(None, DelegationContract));
    client.set_delegation(&owner, &delegate, &limit);

    let spend_amount = 500;
    client.consume_allowance(&owner, &delegate, &spend_amount);

    let delegation = client.get_delegation(&owner, &delegate).unwrap();
    assert_eq!(delegation.limit, limit);
    assert_eq!(delegation.spent, spend_amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_overspend_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let delegate = Address::generate(&env);
    let limit = 1000;

    let client =
        DelegationContractClient::new(&env, &env.register_contract(None, DelegationContract));
    client.set_delegation(&owner, &delegate, &limit);

    let spend_amount = 1500;
    client.consume_allowance(&owner, &delegate, &spend_amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_unauthorized_delegate_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let delegate = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let limit = 1000;

    let client =
        DelegationContractClient::new(&env, &env.register_contract(None, DelegationContract));
    client.set_delegation(&owner, &delegate, &limit);

    let spend_amount = 500;
    client.consume_allowance(&owner, &unauthorized, &spend_amount);
}
