use soroban_sdk::{Address, Env, String};
use stellar_spend_contracts::contracts::errors::StellarSpendError;
use stellar_spend_contracts::contracts::refunds::{
    DataKey, RefundConfig, RefundEligibility, RefundEvents, RefundRequest, RefundStatus,
    RefundsContract,
};

#[test]
fn test_refund_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Test successful initialization
    RefundsContract::initialize(env.clone(), admin.clone());

    assert_eq!(RefundsContract::get_admin(env.clone()), admin);

    // Test default config
    let config = RefundsContract::get_config(env.clone());
    assert_eq!(config.refund_window_seconds, 86400);
    assert_eq!(config.auto_approve_threshold, 1000);
    assert_eq!(config.admin_required_threshold, 10000);
    assert_eq!(config.max_refund_reason_length, 500);
    assert!(config.enabled);
}

#[test]
#[should_panic(expected = "AlreadyInitialized")]
fn test_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin.clone());
    RefundsContract::initialize(env.clone(), admin);
}

#[test]
fn test_config_update() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin.clone());

    let new_config = RefundConfig {
        refund_window_seconds: 43200,
        auto_approve_threshold: 500,
        admin_required_threshold: 5000,
        max_refund_reason_length: 250,
        enabled: false,
    };

    // Admin can update config
    RefundsContract::set_config(env.clone(), admin.clone(), new_config.clone());

    let retrieved_config = RefundsContract::get_config(env.clone());
    assert_eq!(retrieved_config.refund_window_seconds, 43200);
    assert_eq!(retrieved_config.auto_approve_threshold, 500);
    assert_eq!(retrieved_config.admin_required_threshold, 5000);
    assert_eq!(retrieved_config.max_refund_reason_length, 250);
    assert!(!retrieved_config.enabled);
}

#[test]
#[should_panic(expected = "AdminRequired")]
fn test_config_update_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    let new_config = RefundConfig {
        refund_window_seconds: 43200,
        auto_approve_threshold: 500,
        admin_required_threshold: 5000,
        max_refund_reason_length: 250,
        enabled: false,
    };

    // Non-admin cannot update config
    RefundsContract::set_config(env.clone(), user, new_config);
}

#[test]
fn test_auto_approve_refund_request() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Set user balance for refund
    env.storage()
        .persistent()
        .set(&DataKey::Balance(user.clone()), &1000i128);

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64, // transaction_id
        recipient.clone(),
        500i128, // amount (below auto_approve_threshold)
        String::from_str(&env, "Test refund"),
    );

    let refund_request = RefundsContract::get_refund_request(env.clone(), refund_id).unwrap();
    assert_eq!(refund_request.status, RefundStatus::Approved);
    assert_eq!(refund_request.requester, user);
    assert_eq!(refund_request.amount, 500);
    assert_eq!(refund_request.transaction_id, 1);
}

#[test]
fn test_pending_refund_request() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Set user balance for refund
    env.storage()
        .persistent()
        .set(&DataKey::Balance(user.clone()), &15000i128);

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64, // transaction_id
        recipient.clone(),
        15000i128, // amount (above admin_required_threshold)
        String::from_str(&env, "Large refund request"),
    );

    let refund_request = RefundsContract::get_refund_request(env.clone(), refund_id).unwrap();
    assert_eq!(refund_request.status, RefundStatus::Pending);
    assert_eq!(refund_request.requester, user);
    assert_eq!(refund_request.amount, 15000);
}

#[test]
fn test_refund_approval() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin.clone());

    // Set user balance for refund
    env.storage()
        .persistent()
        .set(&DataKey::Balance(user.clone()), &15000i128);

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64,
        recipient.clone(),
        15000i128,
        String::from_str(&env, "Large refund request"),
    );

    // Admin approves the refund
    RefundsContract::approve_refund(env.clone(), admin.clone(), refund_id);

    let refund_request = RefundsContract::get_refund_request(env.clone(), refund_id).unwrap();
    assert_eq!(refund_request.status, RefundStatus::Approved);
    assert_eq!(refund_request.processed_by, Some(admin));
}

#[test]
#[should_panic(expected = "AdminRequired")]
fn test_refund_approval_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64,
        recipient.clone(),
        15000i128,
        String::from_str(&env, "Large refund request"),
    );

    // Unauthorized user tries to approve
    RefundsContract::approve_refund(env.clone(), unauthorized_user, refund_id);
}

#[test]
fn test_refund_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin.clone());

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64,
        recipient.clone(),
        15000i128,
        String::from_str(&env, "Large refund request"),
    );

    let rejection_reason = String::from_str(&env, "Invalid refund reason");

    // Admin rejects the refund
    RefundsContract::reject_refund(
        env.clone(),
        admin.clone(),
        refund_id,
        rejection_reason.clone(),
    );

    let refund_request = RefundsContract::get_refund_request(env.clone(), refund_id).unwrap();
    assert_eq!(refund_request.status, RefundStatus::Rejected);
    assert_eq!(refund_request.processed_by, Some(admin));
}

#[test]
fn test_refund_processing() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Set initial balance
    env.storage()
        .persistent()
        .set(&DataKey::Balance(user.clone()), &1000i128);

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64,
        recipient.clone(),
        500i128, // Auto-approved amount
        String::from_str(&env, "Test refund"),
    );

    // Process the refund
    RefundsContract::process_refund(env.clone(), user.clone(), refund_id);

    let refund_request = RefundsContract::get_refund_request(env.clone(), refund_id).unwrap();
    assert_eq!(refund_request.status, RefundStatus::Processed);
    assert_eq!(refund_request.processed_by, Some(user));

    // Check balance increased
    let final_balance = env
        .storage()
        .persistent()
        .get(&DataKey::Balance(user.clone()))
        .unwrap();
    assert_eq!(final_balance, 1500i128);
}

#[test]
#[should_panic(expected = "AlreadyExists")]
fn test_double_refund_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    env.storage()
        .persistent()
        .set(&DataKey::Balance(user.clone()), &2000i128);

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64,
        recipient.clone(),
        500i128,
        String::from_str(&env, "Test refund"),
    );

    // Process refund first time
    RefundsContract::process_refund(env.clone(), user.clone(), refund_id);

    // Try to process again - should fail
    RefundsContract::process_refund(env.clone(), user, refund_id);
}

#[test]
fn test_refund_eligibility_check() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Test eligible refund
    let eligibility = RefundsContract::check_refund_eligibility(env.clone(), &user, 1u64, 500i128);

    assert!(eligibility.is_eligible);
    assert!(!eligibility.requires_admin);

    // Test large amount requiring admin
    let eligibility =
        RefundsContract::check_refund_eligibility(env.clone(), &user, 2u64, 15000i128);

    assert!(eligibility.is_eligible);
    assert!(eligibility.requires_admin);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_invalid_refund_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Try to request refund with zero amount
    RefundsContract::request_refund(
        env.clone(),
        user,
        1u64,
        recipient,
        0i128,
        String::from_str(&env, "Invalid amount"),
    );
}

#[test]
#[should_panic(expected = "InvalidInput")]
fn test_refund_reason_too_long() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Create a reason longer than max length (500)
    let long_reason = "x".repeat(600);
    let reason = String::from_str(&env, &long_reason);

    RefundsContract::request_refund(env.clone(), user, 1u64, recipient, 500i128, reason);
}

#[test]
#[should_panic(expected = "Paused")]
fn test_refund_when_disabled() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin.clone());

    // Disable refunds
    let mut config = RefundsContract::get_config(env.clone());
    config.enabled = false;
    RefundsContract::set_config(env.clone(), admin, config);

    // Try to request refund when disabled
    RefundsContract::request_refund(
        env.clone(),
        user,
        1u64,
        recipient,
        500i128,
        String::from_str(&env, "Test refund"),
    );
}

#[test]
fn test_get_user_refunds() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Create refunds for user1
    let refund1_id = RefundsContract::request_refund(
        env.clone(),
        user1.clone(),
        1u64,
        recipient.clone(),
        500i128,
        String::from_str(&env, "User1 refund 1"),
    );

    let refund2_id = RefundsContract::request_refund(
        env.clone(),
        user1.clone(),
        2u64,
        recipient.clone(),
        300i128,
        String::from_str(&env, "User1 refund 2"),
    );

    // Create refund for user2
    let refund3_id = RefundsContract::request_refund(
        env.clone(),
        user2.clone(),
        3u64,
        recipient,
        700i128,
        String::from_str(&env, "User2 refund"),
    );

    // Get user1's refunds
    let user1_refunds = RefundsContract::get_user_refunds(env.clone(), user1.clone());
    assert_eq!(user1_refunds.len(), 2);

    let refund_ids: Vec<u64> = user1_refunds.iter().map(|r| r.id).collect();
    assert!(refund_ids.contains(&refund1_id));
    assert!(refund_ids.contains(&refund2_id));

    // Get user2's refunds
    let user2_refunds = RefundsContract::get_user_refunds(env.clone(), user2.clone());
    assert_eq!(user2_refunds.len(), 1);
    assert_eq!(user2_refunds.get(0).unwrap().id, refund3_id);
}

#[test]
fn test_get_pending_refunds() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin);

    // Create auto-approved refund (not pending)
    RefundsContract::request_refund(
        env.clone(),
        user1,
        1u64,
        recipient.clone(),
        500i128,
        String::from_str(&env, "Auto-approved refund"),
    );

    // Create pending refund (large amount)
    let pending_refund_id = RefundsContract::request_refund(
        env.clone(),
        user2,
        2u64,
        recipient,
        15000i128,
        String::from_str(&env, "Pending refund"),
    );

    // Get pending refunds
    let pending_refunds = RefundsContract::get_pending_refunds(env.clone());
    assert_eq!(pending_refunds.len(), 1);
    assert_eq!(pending_refunds.get(0).unwrap().id, pending_refund_id);
    assert_eq!(
        pending_refunds.get(0).unwrap().status,
        RefundStatus::Pending
    );
}

#[test]
fn test_refund_events() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin.clone());

    // Test refund requested event
    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64,
        recipient.clone(),
        500i128,
        String::from_str(&env, "Test refund"),
    );

    // Test refund approved event (for pending refunds)
    let large_refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        2u64,
        recipient.clone(),
        15000i128,
        String::from_str(&env, "Large refund"),
    );

    RefundsContract::approve_refund(env.clone(), admin.clone(), large_refund_id);

    // Test refund processed event
    RefundsContract::process_refund(env.clone(), user.clone(), refund_id);

    // Verify events were published by checking they don't panic
    // In a real test environment, you would capture and verify the actual event data
}

#[test]
fn test_refund_status_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let recipient = Address::generate(&env);

    RefundsContract::initialize(env.clone(), admin.clone());

    let refund_id = RefundsContract::request_refund(
        env.clone(),
        user.clone(),
        1u64,
        recipient.clone(),
        15000i128,
        String::from_str(&env, "Large refund"),
    );

    // Initial status should be Pending
    assert_eq!(
        RefundsContract::get_refund_status(env.clone(), refund_id).unwrap(),
        RefundStatus::Pending
    );

    // Approve refund
    RefundsContract::approve_refund(env.clone(), admin.clone(), refund_id);

    // Status should be Approved
    assert_eq!(
        RefundsContract::get_refund_status(env.clone(), refund_id).unwrap(),
        RefundStatus::Approved
    );

    // Process refund
    RefundsContract::process_refund(env.clone(), user.clone(), refund_id);

    // Status should be Processed
    assert_eq!(
        RefundsContract::get_refund_status(env.clone(), refund_id).unwrap(),
        RefundStatus::Processed
    );
}
