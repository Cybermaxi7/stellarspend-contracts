use crate::{BatchHistoryContract, BatchHistoryContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn test_batch_retrieval() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BatchHistoryContract, ());
    let client = BatchHistoryContractClient::new(&env, &contract_id);

    let requester = Address::generate(&env);
    let user_1 = Address::generate(&env);

    let users = vec![&env, user_1.clone()];
    let results = client.retrieve_histories(&requester, &users);

    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap().user, user_1);
}
