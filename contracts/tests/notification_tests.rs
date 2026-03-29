#![cfg(test)]

use soroban_sdk::{Env, Address, String, Vec};
use crate::{NotificationContract, NotificationContractClient};
use crate::types::Notification;

#[test]
fn test_batch_success() {
    let env = Env::default();
    let contract_id = env.register_contract(None, NotificationContract);
    let client = NotificationContractClient::new(&env, &contract_id);

    let user1 = Address::random(&env);
    let user2 = Address::random(&env);

    let mut batch = Vec::new(&env);

    batch.push_back(Notification {
        recipient: user1.clone(),
        language: String::from_str("en"),
        message: String::from_str("Hello"),
    });

    batch.push_back(Notification {
        recipient: user2.clone(),
        language: String::from_str("fr"),
        message: String::from_str("Bonjour"),
    });

    let result = client.send_batch_notifications(&batch);

    assert_eq!(result.len(), 2);
    assert!(result.get(0).unwrap().success);
    assert!(result.get(1).unwrap().success);
}

#[test]
fn test_partial_failure() {
    let env = Env::default();
    let contract_id = env.register_contract(None, NotificationContract);
    let client = NotificationContractClient::new(&env, &contract_id);

    let user = Address::random(&env);

    let mut batch = Vec::new(&env);

    batch.push_back(Notification {
        recipient: user.clone(),
        language: String::from_str("xx"), // invalid
        message: String::from_str("Test"),
    });

    let result = client.send_batch_notifications(&batch);

    assert_eq!(result.len(), 1);
    assert!(!result.get(0).unwrap().success);
}