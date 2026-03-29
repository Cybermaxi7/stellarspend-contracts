#![allow(dead_code)]

use fee::{FeeContract, FeeContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};

pub(crate) struct TestContext {
    pub env: Env,
    pub admin: Address,
    pub payer: Address,
    pub treasury: Address,
    pub alt_treasury: Address,
    pub contract_id: Address,
    pub token_id: Address,
    pub token_client: token::Client<'static>,
    pub asset_client: token::StellarAssetClient<'static>,
    pub client: FeeContractClient<'static>,
}

pub(crate) fn setup() -> TestContext {
    let env = Env::default();
    env.mock_all_auths();

    let issuer = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(issuer);
    let token_id = stellar_asset.address();
    let token_client = token::Client::new(&env, &token_id);
    let asset_client = token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(FeeContract, ());
    let client = FeeContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let payer = Address::generate(&env);
    let treasury = Address::generate(&env);
    let alt_treasury = Address::generate(&env);

    client.initialize(&admin, &token_id, &treasury, &250u32, &1u64);
    asset_client.mint(&payer, &1_000_000i128);

    TestContext {
        env,
        admin,
        payer,
        treasury,
        alt_treasury,
        contract_id,
        token_id,
        token_client,
        asset_client,
        client,
    }
}

pub(crate) fn amounts(env: &Env, items: &[i128]) -> soroban_sdk::Vec<i128> {
    let mut result = soroban_sdk::Vec::new(env);
    for item in items {
        result.push_back(*item);
    }
    result
}
