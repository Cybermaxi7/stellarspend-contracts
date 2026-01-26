use crate::types::NotificationPayload;
use crate::{BatchNotificationContract, BatchNotificationContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

#[test]
fn test_batch_dispatch_mixed_results() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BatchNotificationContract, ());
    let client = BatchNotificationContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user_1 = Address::generate(&env);
    let user_2 = Address::generate(&env);

    let payloads = vec![
        &env,
        NotificationPayload {
            user: user_1.clone(),
            message: String::from_str(&env, "Successful Message"),
        },
        NotificationPayload {
            user: user_2.clone(),
            message: String::from_str(&env, ""), // This will trigger a "Failure"
        },
    ];

    let result = client.batch_notify(&admin, &payloads);

    assert_eq!(result.successful_count, 1);
    assert_eq!(result.failed_addresses.len(), 1);
    assert_eq!(result.failed_addresses.get(0).unwrap(), user_2);
}
