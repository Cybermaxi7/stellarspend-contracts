use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events as _},
    Address, Env, String, Vec, U256,
};

#[path = "../contracts/history.rs"]
mod history;

use history::{
    HistoryContract, HistoryContractClient, HistoryError, PaginatedResult, SortOrder, TimeRange,
    TransactionRecord, TransactionStatus, TransactionType, UserTransactionSummary,
};

fn setup_history_contract() -> (Env, Address, HistoryContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(HistoryContract, ());
    let client = HistoryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    (env, admin, client)
}

fn create_test_transaction(
    env: &Env,
    client: &HistoryContractClient<'static>,
    from: &Address,
    to: &Address,
    amount: i128,
    description: &str,
    transaction_type: TransactionType,
) -> U256 {
    let desc = String::from_str(env, description);
    client.store_transaction(from, to, &amount, &desc, &transaction_type)
}

#[test]
fn test_history_initialization() {
    let (env, admin, client) = setup_history_contract();

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_transaction_count(), 0);
}

#[test]
#[should_panic]
fn test_double_initialization_fails() {
    let (env, _admin, client) = setup_history_contract();

    let another_admin = Address::generate(&env);
    client.initialize(&another_admin);
}

#[test]
fn test_store_transaction_success() {
    let (env, _admin, client) = setup_history_contract();

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 1000i128;

    let transaction_id = create_test_transaction(
        &env,
        &client,
        &from,
        &to,
        amount,
        "Test transaction",
        TransactionType::Payment,
    );

    assert!(client.get_transaction(&transaction_id).is_some());
    assert_eq!(client.get_transaction_count(), 1);

    // Check events
    let events = env.events().all();
    let stored_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("stored")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(stored_events, 1);
}

#[test]
fn test_get_transaction_success() {
    let (env, _admin, client) = setup_history_contract();

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 500i128;

    let transaction_id = create_test_transaction(
        &env,
        &client,
        &from,
        &to,
        amount,
        "Test transaction",
        TransactionType::Transfer,
    );

    let transaction = client
        .get_transaction(&transaction_id)
        .expect("transaction should exist");
    assert_eq!(transaction.id, transaction_id);
    assert_eq!(transaction.from, from);
    assert_eq!(transaction.to, to);
    assert_eq!(transaction.amount, amount);
    assert_eq!(transaction.transaction_type, TransactionType::Transfer);
    assert_eq!(transaction.status, TransactionStatus::Completed);
}

#[test]
fn test_get_nonexistent_transaction_returns_none() {
    let (env, _admin, client) = setup_history_contract();

    let fake_id = U256::from_u32(&env, 999);
    assert!(client.get_transaction(&fake_id).is_none());
}

#[test]
fn test_get_user_transactions_paginated_single_page() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create 5 transactions for the user
    for i in 0..5 {
        create_test_transaction(
            &env,
            &client,
            &user,
            &other,
            (i + 1) * 100,
            &format!("Transaction {}", i + 1).as_str(),
            TransactionType::Payment,
        );
    }

    let result = client.get_user_transactions_paginated(&user, 0, 10, SortOrder::Ascending);

    assert_eq!(result.total_count, 5);
    assert_eq!(result.page_number, 0);
    assert_eq!(result.page_size, 10);
    assert_eq!(result.transactions.len(), 5);
    assert!(!result.has_next);
    assert!(!result.has_previous);
}

#[test]
fn test_get_user_transactions_paginated_multiple_pages() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create 7 transactions for the user
    for i in 0..7 {
        create_test_transaction(
            &env,
            &client,
            &user,
            &other,
            (i + 1) * 100,
            &format!("Transaction {}", i + 1).as_str(),
            TransactionType::Payment,
        );
    }

    // First page (3 items)
    let page1 = client.get_user_transactions_paginated(&user, 0, 3, SortOrder::Ascending);
    assert_eq!(page1.total_count, 7);
    assert_eq!(page1.page_number, 0);
    assert_eq!(page1.page_size, 3);
    assert_eq!(page1.transactions.len(), 3);
    assert!(page1.has_next);
    assert!(!page1.has_previous);

    // Second page (3 items)
    let page2 = client.get_user_transactions_paginated(&user, 1, 3, SortOrder::Ascending);
    assert_eq!(page2.total_count, 7);
    assert_eq!(page2.page_number, 1);
    assert_eq!(page2.page_size, 3);
    assert_eq!(page2.transactions.len(), 3);
    assert!(page2.has_next);
    assert!(page2.has_previous);

    // Third page (1 item)
    let page3 = client.get_user_transactions_paginated(&user, 2, 3, SortOrder::Ascending);
    assert_eq!(page3.total_count, 7);
    assert_eq!(page3.page_number, 2);
    assert_eq!(page3.page_size, 3);
    assert_eq!(page3.transactions.len(), 1);
    assert!(!page3.has_next);
    assert!(page3.has_previous);
}

#[test]
#[should_panic]
fn test_invalid_page_size_fails() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    client.get_user_transactions_paginated(&user, 0, 0, SortOrder::Ascending);
}

#[test]
#[should_panic]
fn test_page_size_too_large_fails() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    client.get_user_transactions_paginated(&user, 0, 101, SortOrder::Ascending);
}

#[test]
#[should_panic]
fn test_invalid_page_number_fails() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    client.get_user_transactions_paginated(&user, 1, 10, SortOrder::Ascending);
}

#[test]
fn test_sort_order_ascending() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create transactions with different amounts
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        100,
        "First",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        200,
        "Second",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        50,
        "Third",
        TransactionType::Payment,
    );

    let result = client.get_user_transactions_paginated(&user, 0, 10, SortOrder::Ascending);

    assert_eq!(result.transactions.len(), 3);
    assert_eq!(result.transactions.get(0).unwrap().amount, 100);
    assert_eq!(result.transactions.get(1).unwrap().amount, 200);
    assert_eq!(result.transactions.get(2).unwrap().amount, 50);
}

#[test]
fn test_sort_order_descending() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create transactions with different amounts
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        100,
        "First",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        200,
        "Second",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        50,
        "Third",
        TransactionType::Payment,
    );

    let result = client.get_user_transactions_paginated(&user, 0, 10, SortOrder::Descending);

    assert_eq!(result.transactions.len(), 3);
    assert_eq!(result.transactions.get(0).unwrap().amount, 50);
    assert_eq!(result.transactions.get(1).unwrap().amount, 200);
    assert_eq!(result.transactions.get(2).unwrap().amount, 100);
}

#[test]
fn test_get_latest_transactions() {
    let (env, _admin, client) = setup_history_contract();

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Create 10 transactions
    for i in 0..10 {
        create_test_transaction(
            &env,
            &client,
            &user1,
            &user2,
            (i + 1) * 100,
            &format!("Transaction {}", i + 1).as_str(),
            TransactionType::Payment,
        );
    }

    let latest = client.get_latest_transactions(5);
    assert_eq!(latest.len(), 5);

    // Should be in descending order (newest first)
    for i in 0..4 {
        assert!(latest.get(i).unwrap().timestamp >= latest.get(i + 1).unwrap().timestamp);
    }
}

#[test]
fn test_get_transactions_by_time_range_all() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create transactions
    for i in 0..5 {
        create_test_transaction(
            &env,
            &client,
            &user,
            &other,
            (i + 1) * 100,
            &format!("Transaction {}", i + 1).as_str(),
            TransactionType::Payment,
        );
    }

    let transactions = client.get_transactions_by_time_range(TimeRange::All, 10);
    assert_eq!(transactions.len(), 5);
}

#[test]
fn test_get_transactions_by_time_range_custom() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create transactions
    for i in 0..5 {
        create_test_transaction(
            &env,
            &client,
            &user,
            &other,
            (i + 1) * 100,
            &format!("Transaction {}", i + 1).as_str(),
            TransactionType::Payment,
        );
    }

    let current_time = env.ledger().timestamp();
    let start_time = current_time.saturating_sub(100);
    let end_time = current_time.saturating_add(100);

    let transactions =
        client.get_transactions_by_time_range(TimeRange::Custom(start_time, end_time), 10);
    assert_eq!(transactions.len(), 5);
}

#[test]
#[should_panic]
fn test_invalid_time_range_fails() {
    let (env, _admin, client) = setup_history_contract();

    client.get_transactions_by_time_range(
        TimeRange::Custom(1000, 500), // start > end
        10,
    );
}

#[test]
fn test_get_user_transaction_summary() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create sent transactions
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        1000,
        "Sent 1",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        500,
        "Sent 2",
        TransactionType::Transfer,
    );

    // Create received transactions
    create_test_transaction(
        &env,
        &client,
        &other,
        &user,
        300,
        "Received 1",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &other,
        &user,
        200,
        "Received 2",
        TransactionType::Transfer,
    );

    let summary = client.get_user_transaction_summary(&user);

    assert_eq!(summary.user, user);
    assert_eq!(summary.total_transactions, 4);
    assert_eq!(summary.total_sent, 1500);
    assert_eq!(summary.total_received, 500);
    assert!(summary.first_transaction_timestamp.is_some());
    assert!(summary.last_transaction_timestamp.is_some());
}

#[test]
fn test_search_transactions() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create transactions with different descriptions
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        100,
        "Payment for coffee",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        200,
        "Payment for lunch",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        300,
        "Transfer to savings",
        TransactionType::Transfer,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        400,
        "Payment for dinner",
        TransactionType::Payment,
    );

    // Search for "Payment"
    let query = String::from_str(&env, "Payment");
    let result = client.search_transactions(&query, 0, 10);

    assert_eq!(result.total_count, 3);
    assert_eq!(result.transactions.len(), 3);

    // Verify all results contain "Payment" in description
    for transaction in result.transactions.iter() {
        assert!(transaction.description.contains(&query));
    }
}

#[test]
fn test_search_transactions_pagination() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create transactions with "Payment" in description
    for i in 0..5 {
        create_test_transaction(
            &env,
            &client,
            &user,
            &other,
            (i + 1) * 100,
            &format!("Payment {}", i + 1).as_str(),
            TransactionType::Payment,
        );
    }

    let query = String::from_str(&env, "Payment");

    // First page
    let page1 = client.search_transactions(&query, 0, 2);
    assert_eq!(page1.total_count, 5);
    assert_eq!(page1.transactions.len(), 2);
    assert!(page1.has_next);
    assert!(!page1.has_previous);

    // Second page
    let page2 = client.search_transactions(&query, 1, 2);
    assert_eq!(page2.total_count, 5);
    assert_eq!(page2.transactions.len(), 2);
    assert!(page2.has_next);
    assert!(page2.has_previous);

    // Third page
    let page3 = client.search_transactions(&query, 2, 2);
    assert_eq!(page3.total_count, 5);
    assert_eq!(page3.transactions.len(), 1);
    assert!(!page3.has_next);
    assert!(page3.has_previous);
}

#[test]
fn test_search_no_results() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        100,
        "Random transaction",
        TransactionType::Payment,
    );

    let query = String::from_str(&env, "Nonexistent");
    let result = client.search_transactions(&query, 0, 10);

    assert_eq!(result.total_count, 0);
    assert_eq!(result.transactions.len(), 0);
    assert!(!result.has_next);
    assert!(!result.has_previous);
}

#[test]
fn test_rebuild_index_admin_only() {
    let (env, admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create some transactions
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        100,
        "Test",
        TransactionType::Payment,
    );

    // Admin can rebuild index
    client.rebuild_index(&admin);

    // Check events
    let events = env.events().all();
    let rebuild_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("index_rebuilt")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert!(rebuild_events >= 1);
}

#[test]
#[should_panic]
fn test_rebuild_index_unauthorized_fails() {
    let (env, _admin, client) = setup_history_contract();

    let unauthorized = Address::generate(&env);
    client.rebuild_index(&unauthorized);
}

#[test]
fn test_user_transactions_with_same_sender_receiver() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);

    // Create transaction where user is both sender and receiver
    create_test_transaction(
        &env,
        &client,
        &user,
        &user,
        100,
        "Self transaction",
        TransactionType::Payment,
    );

    let result = client.get_user_transactions_paginated(&user, 0, 10, SortOrder::Ascending);
    assert_eq!(result.total_count, 1);
    assert_eq!(result.transactions.len(), 1);

    let transaction = result.transactions.get(0).unwrap();
    assert_eq!(transaction.from, user);
    assert_eq!(transaction.to, user);
}

#[test]
fn test_transaction_count_increments() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    assert_eq!(client.get_transaction_count(), 0);

    // Add transactions
    for i in 1..=5 {
        create_test_transaction(
            &env,
            &client,
            &user,
            &other,
            i * 100,
            "Test",
            TransactionType::Payment,
        );
        assert_eq!(client.get_transaction_count(), i);
    }
}

#[test]
fn test_different_transaction_types() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create different types of transactions
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        100,
        "Payment",
        TransactionType::Payment,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        200,
        "Transfer",
        TransactionType::Transfer,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        300,
        "Deposit",
        TransactionType::Deposit,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        400,
        "Withdrawal",
        TransactionType::Withdrawal,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        500,
        "Reward",
        TransactionType::Reward,
    );
    create_test_transaction(
        &env,
        &client,
        &user,
        &other,
        600,
        "Refund",
        TransactionType::Refund,
    );

    let result = client.get_user_transactions_paginated(&user, 0, 10, SortOrder::Ascending);
    assert_eq!(result.transactions.len(), 6);

    // Verify all transaction types are present
    let mut types_found = Vec::new(&env);
    for transaction in result.transactions.iter() {
        types_found.push_back(transaction.transaction_type);
    }
    assert_eq!(types_found.len(), 6);
}

#[test]
fn test_pagination_events_emitted() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);
    let other = Address::generate(&env);

    // Create some transactions
    for i in 0..3 {
        create_test_transaction(
            &env,
            &client,
            &user,
            &other,
            100,
            "Test",
            TransactionType::Payment,
        );
    }

    // Retrieve paginated results
    client.get_user_transactions_paginated(&user, 0, 2, SortOrder::Ascending);

    // Check events
    let events = env.events().all();
    let page_events = events
        .iter()
        .filter(|event| {
            event.1.iter().any(|topic| {
                symbol_short!("page_retrieved")
                    == soroban_sdk::Symbol::try_from_val(&env, &topic).unwrap_or(symbol_short!(""))
            })
        })
        .count();
    assert_eq!(page_events, 1);
}

#[test]
fn test_empty_user_history() {
    let (env, _admin, client) = setup_history_contract();

    let user = Address::generate(&env);

    let result = client.get_user_transactions_paginated(&user, 0, 10, SortOrder::Ascending);

    assert_eq!(result.total_count, 0);
    assert_eq!(result.transactions.len(), 0);
    assert!(!result.has_next);
    assert!(!result.has_previous);

    let summary = client.get_user_transaction_summary(&user);
    assert_eq!(summary.total_transactions, 0);
    assert_eq!(summary.total_sent, 0);
    assert_eq!(summary.total_received, 0);
    assert!(summary.first_transaction_timestamp.is_none());
    assert!(summary.last_transaction_timestamp.is_none());
}
