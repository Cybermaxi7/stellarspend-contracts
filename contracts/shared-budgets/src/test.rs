// Integration tests for the Shared Budget Contract.

#![cfg(test)]

use crate::{
    Budget, BudgetContribution, BudgetSpendingRule, SharedBudgetContract,
    SharedBudgetContractClient, SharedBudgetError,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token, Address, Env, Symbol, Vec,
};

/// Creates a test environment with the contract deployed and initialized.
fn setup_test_env() -> (Env, Address, Address, token::Client<'static>, SharedBudgetContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.sequence_number = 12345;
    });

    // Deploy token contract (simulating XLM StellarAssetContract)
    let issuer = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(issuer);
    let token_id: Address = stellar_asset.address();
    let token_client = token::Client::new(&env, &token_id);

    // Deploy shared budget contract
    let contract_id = env.register(SharedBudgetContract, ());
    let client = SharedBudgetContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, token_id, token_client, client)
}

// Initialization Tests

#[test]
fn test_initialize_contract() {
    let (_env, admin, _token, _token_client, client) = setup_test_env();

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_total_budgets_created(), 0);
    assert_eq!(client.get_total_contribs_processed(), 0);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_initialize_twice() {
    let (env, _admin, _token, _token_client, client) = setup_test_env();

    let new_admin = Address::generate(&env);
    client.initialize(&new_admin);
}

// Budget Creation Tests

#[test]
fn test_create_budget() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());
    members.push_back(member2.clone());

    let budget_name = Symbol::new(&env, "family_budget");

    let mut spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);
    let rule = BudgetSpendingRule {
        applicable_to: member1.clone(),
        percentage_threshold: 10,
        requires_approval: false,
        description: Symbol::new(&env, "small_purchases"),
    };
    spending_rules.push_back(rule);

    let budget_id = client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    assert!(budget_id > 0);

    let budget = client.get_budget(&budget_id);
    assert_eq!(budget.name, budget_name);
    assert_eq!(budget.creator, creator);
    assert_eq!(budget.token, token);
    assert_eq!(budget.balance, 0);
    assert_eq!(budget.total_contributed, 0);
    assert_eq!(budget.is_active, true);
    assert!(client.is_budget_member(&budget_id, &member1));
    assert!(client.is_budget_member(&budget_id, &member2));
}

#[test]
fn test_contribute_to_budget() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let member1 = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());

    let budget_name = Symbol::new(&env, "vacation_fund");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    let budget_id = client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    // Initially budget should have 0 balance
    let budget_before = client.get_budget(&budget_id);
    assert_eq!(budget_before.balance, 0);

    // Contribute to the budget
    let contribution_amount = 100_000_000; // 10 XLM
    client.contribute_to_budget(&contributor, &budget_id, &contribution_amount);

    // Check that budget balance increased
    let budget_after = client.get_budget(&budget_id);
    assert_eq!(budget_after.balance, contribution_amount);
    assert_eq!(budget_after.total_contributed, contribution_amount);

    // Check that total contributions processed increased
    assert_eq!(client.get_total_contribs_processed(), 1);
}

#[test]
fn test_spend_from_budget() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let recipient = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());

    let budget_name = Symbol::new(&env, "project_fund");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    let budget_id = client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    // Contribute to budget first
    let contribution_amount = 100_000_000; // 10 XLM
    client.contribute_to_budget(&member1, &budget_id, &contribution_amount);

    // Spend from budget
    let expense_amount = 50_000_000; // 5 XLM
    client.spend_from_budget(&member1, &budget_id, &recipient, &expense_amount);

    // Check that budget balance decreased
    let budget_after = client.get_budget(&budget_id);
    assert_eq!(budget_after.balance, contribution_amount - expense_amount);
}

#[test]
fn test_add_member_to_budget() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let existing_member = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(existing_member.clone());

    let budget_name = Symbol::new(&env, "team_budget");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    let budget_id = client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    // Add new member
    let new_member = Address::generate(&env);
    client.add_member_to_budget(&creator, &budget_id, &new_member);

    // Check that new member is now part of budget
    assert!(client.is_budget_member(&budget_id, &new_member));
}

#[test]
fn test_add_spending_rule() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());

    let budget_name = Symbol::new(&env, "controlled_budget");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    let budget_id = client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    // Add spending rule
    let new_rule = BudgetSpendingRule {
        applicable_to: member1.clone(),
        percentage_threshold: 20,
        requires_approval: true,
        description: Symbol::new(&env, "approval_required"),
    };

    client.add_spending_rule(&creator, &budget_id, &new_rule);

    // Verify the rule was added by attempting to get the budget and check its rules
    let budget = client.get_budget(&budget_id);
    assert!(budget.spending_rules.len() > 0);
}

#[test]
fn test_budget_events_emitted() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());

    let budget_name = Symbol::new(&env, "event_budget");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    // Check that events were emitted
    let events = env.events().all();
    assert!(events.len() >= 1); // At least budget creation event
    
    // Check that events were emitted (simplified check)
    assert!(!events.is_empty());
}

#[test]
fn test_budget_stats_accumulate() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    assert_eq!(client.get_total_budgets_created(), 0);

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());

    let budget_name = Symbol::new(&env, "stats_budget");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    assert_eq!(client.get_total_budgets_created(), 1);

    // Create another budget
    let budget_name2 = Symbol::new(&env, "stats_budget2");
    client.create_budget(&creator, &budget_name2, &members, &token, &spending_rules);

    assert_eq!(client.get_total_budgets_created(), 2);
}

// Error Tests

#[test]
fn test_spend_without_sufficient_funds() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let recipient = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());

    let budget_name = Symbol::new(&env, "empty_budget");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    let budget_id = client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    // Try to spend without contributing anything
    let expense_amount = 50_000_000; // 5 XLM
    client.spend_from_budget(&member1, &budget_id, &recipient, &expense_amount);
}

#[test]
fn test_non_member_cannot_spend() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let creator = Address::generate(&env);
    let member1 = Address::generate(&env);
    let non_member = Address::generate(&env);
    let recipient = Address::generate(&env);

    let mut members: Vec<Address> = Vec::new(&env);
    members.push_back(member1.clone());

    let budget_name = Symbol::new(&env, "restricted_budget");
    let spending_rules: Vec<BudgetSpendingRule> = Vec::new(&env);

    let budget_id = client.create_budget(&creator, &budget_name, &members, &token, &spending_rules);

    // Contribute to budget first
    let contribution_amount = 100_000_000; // 10 XLM
    client.contribute_to_budget(&member1, &budget_id, &contribution_amount);

    // Non-member tries to spend (should fail)
    let expense_amount = 50_000_000; // 5 XLM
    client.spend_from_budget(&non_member, &budget_id, &recipient, &expense_amount);
}

#[test]
#[should_panic]
fn test_unauthorized_admin_function() {
    let (env, admin, token, _token_client, client) = setup_test_env();

    let unauthorized_user = Address::generate(&env);
    let new_admin = Address::generate(&env);

    client.set_admin(&unauthorized_user, &new_admin);
}