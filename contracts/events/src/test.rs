#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    Address, Env, IntoVal,
};

use crate::{
    events::{
        validate_initialize_event, validate_stake_event, validate_unstake_event,
        InitializeEventData, StakeEventData, UnstakeEventData,
        CONTRACT_TOPIC,
        topic_initialize, topic_stake, topic_unstake,
    },
    StakingContract, StakingContractClient,
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Spin up a fresh test environment with a deterministic ledger timestamp.
fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        timestamp:          1_700_000_000,
        protocol_version:   20,
        sequence_number:    1,
        network_id:         Default::default(),
        base_reserve:       10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl:      6_312_000,
    });
    env
}

/// Register the staking contract and return (client, admin, token_address).
/// For event-only tests we use a dummy token address — no real token client needed.
fn deploy_contract(env: &Env) -> (StakingContractClient, Address, Address) {
    let admin = Address::generate(env);
    let token = Address::generate(env); // dummy — not backed by a real token contract
    let contract_id = env.register_contract(None, StakingContract);
    let client = StakingContractClient::new(env, &contract_id);
    (client, admin, token)
}

// ─────────────────────────────────────────────────────────────────────────────
// Section 1 — Unit tests for event payload validation
// These test the validate_* helpers directly, without touching the contract.
// ─────────────────────────────────────────────────────────────────────────────

mod validate_initialize_event_tests {
    use super::*;

    #[test]
    fn valid_initialize_event_passes() {
        let env  = setup_env();
        let data = InitializeEventData {
            admin:       Address::generate(&env),
            reward_rate: 1200,
            min_stake:   100,
            timestamp:   1_700_000_000,
        };
        validate_initialize_event(&data); // must not panic
    }

    #[test]
    #[should_panic(expected = "reward_rate must be greater than zero")]
    fn zero_reward_rate_fails() {
        let env  = setup_env();
        let data = InitializeEventData {
            admin:       Address::generate(&env),
            reward_rate: 0,
            min_stake:   100,
            timestamp:   1_700_000_000,
        };
        validate_initialize_event(&data);
    }

    #[test]
    #[should_panic(expected = "min_stake must be greater than zero")]
    fn zero_min_stake_fails() {
        let env  = setup_env();
        let data = InitializeEventData {
            admin:       Address::generate(&env),
            reward_rate: 1200,
            min_stake:   0,
            timestamp:   1_700_000_000,
        };
        validate_initialize_event(&data);
    }
}

mod validate_stake_event_tests {
    use super::*;

    #[test]
    fn valid_stake_event_passes() {
        let env  = setup_env();
        let data = StakeEventData {
            staker:    Address::generate(&env),
            amount:    500,
            total:     1_000,
            timestamp: 1_700_000_000,
        };
        validate_stake_event(&data);
    }

    #[test]
    #[should_panic(expected = "stake amount must be greater than zero")]
    fn zero_stake_amount_fails() {
        let env  = setup_env();
        let data = StakeEventData {
            staker:    Address::generate(&env),
            amount:    0,
            total:     0,
            timestamp: 1_700_000_000,
        };
        validate_stake_event(&data);
    }

    #[test]
    #[should_panic(expected = "total staked cannot be less than the staked amount")]
    fn total_less_than_amount_fails() {
        let env  = setup_env();
        let data = StakeEventData {
            staker:    Address::generate(&env),
            amount:    1_000,
            total:     500, // total < amount — impossible state
            timestamp: 1_700_000_000,
        };
        validate_stake_event(&data);
    }
}

mod validate_unstake_event_tests {
    use super::*;

    #[test]
    fn valid_unstake_event_passes() {
        let env  = setup_env();
        let data = UnstakeEventData {
            staker:    Address::generate(&env),
            amount:    500,
            reward:    10,
            remaining: 500,
            timestamp: 1_700_000_000,
        };
        validate_unstake_event(&data);
    }

    #[test]
    #[should_panic(expected = "unstake amount must be greater than zero")]
    fn zero_unstake_amount_fails() {
        let env  = setup_env();
        let data = UnstakeEventData {
            staker:    Address::generate(&env),
            amount:    0,
            reward:    0,
            remaining: 0,
            timestamp: 1_700_000_000,
        };
        validate_unstake_event(&data);
    }

    #[test]
    #[should_panic(expected = "reward cannot be negative")]
    fn negative_reward_fails() {
        let env  = setup_env();
        let data = UnstakeEventData {
            staker:    Address::generate(&env),
            amount:    500,
            reward:    -1,
            remaining: 0,
            timestamp: 1_700_000_000,
        };
        validate_unstake_event(&data);
    }

    #[test]
    #[should_panic(expected = "remaining balance cannot be negative")]
    fn negative_remaining_fails() {
        let env  = setup_env();
        let data = UnstakeEventData {
            staker:    Address::generate(&env),
            amount:    500,
            reward:    10,
            remaining: -1,
            timestamp: 1_700_000_000,
        };
        validate_unstake_event(&data);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Section 2 — Integration tests: event emission via contract entry points
// These verify that calling the public contract functions actually publishes
// correctly structured events into the Soroban event log.
// ─────────────────────────────────────────────────────────────────────────────

mod emit_initialize_event_tests {
    use super::*;

    #[test]
    fn initialize_emits_correct_event() {
        let env = setup_env();
        let (client, admin, token) = deploy_contract(&env);

        client.initialize(&admin, &token, &1200_u32, &100_i128);

        let events = env.events().all();
        assert_eq!(events.len(), 1, "expected exactly one event after initialize");

        let (_, topics, data) = events.first().unwrap();

        // Verify topics
        assert_eq!(
            topics,
            soroban_sdk::vec![&env, CONTRACT_TOPIC.into_val(&env), topic_initialize().into_val(&env)]
        );

        // Verify payload
        let payload: InitializeEventData = data.into_val(&env);
        assert_eq!(payload.admin,       admin);
        assert_eq!(payload.reward_rate, 1200);
        assert_eq!(payload.min_stake,   100);
        assert_eq!(payload.timestamp,   1_700_000_000);
    }

    #[test]
    #[should_panic(expected = "contract already initialised")]
    fn double_initialize_panics() {
        let env = setup_env();
        let (client, admin, token) = deploy_contract(&env);
        client.initialize(&admin, &token, &1200_u32, &100_i128);
        client.initialize(&admin, &token, &1200_u32, &100_i128); // must panic
    }
}

mod emit_stake_event_tests {
    use super::*;

    #[test]
    fn stake_emits_correct_event() {
        let env = setup_env();
        let (client, admin, token) = deploy_contract(&env);
        client.initialize(&admin, &token, &1200_u32, &100_i128);

        let staker = Address::generate(&env);
        env.events().all(); // clear init event

        client.stake(&staker, &500_i128);

        let events = env.events().all();
        // The last event should be the stake event
        let (_, topics, data) = events.last().unwrap();

        assert_eq!(
            topics,
            soroban_sdk::vec![&env, CONTRACT_TOPIC.into_val(&env), topic_stake().into_val(&env)]
        );

        let payload: StakeEventData = data.into_val(&env);
        assert_eq!(payload.staker, staker);
        assert_eq!(payload.amount, 500);
        assert_eq!(payload.total,  500); // first stake, so total == amount
    }

    #[test]
    fn stake_twice_accumulates_total() {
        let env = setup_env();
        let (client, admin, token) = deploy_contract(&env);
        client.initialize(&admin, &token, &1200_u32, &100_i128);

        let staker = Address::generate(&env);
        client.stake(&staker, &300_i128);
        client.stake(&staker, &700_i128);

        let events  = env.events().all();
        let (_, _, data) = events.last().unwrap();
        let payload: StakeEventData = data.into_val(&env);

        assert_eq!(payload.amount, 700);
        assert_eq!(payload.total,  1_000); // 300 + 700
    }

    #[test]
    #[should_panic(expected = "amount is below the minimum stake")]
    fn stake_below_minimum_panics() {
        let env = setup_env();
        let (client, admin, token) = deploy_contract(&env);
        client.initialize(&admin, &token, &1200_u32, &100_i128);

        let staker = Address::generate(&env);
        client.stake(&staker, &50_i128); // below min_stake of 100
    }
}

mod emit_unstake_event_tests {
    use super::*;

    /// Helper that initialises + stakes so we have a balance to unstake.
    fn setup_with_stake(env: &Env, amount: i128) -> (StakingContractClient, Address) {
        let (client, admin, token) = deploy_contract(env);
        client.initialize(&admin, &token, &1200_u32, &100_i128);
        let staker = Address::generate(env);
        client.stake(&staker, &amount);
        (client, staker)
    }

    #[test]
    fn unstake_emits_correct_event() {
        let env = setup_env();
        let (client, staker) = setup_with_stake(&env, 1_000);

        // Advance ledger time so reward > 0
        env.ledger().set(LedgerInfo {
            timestamp: 1_700_000_000 + 30 * 24 * 60 * 60, // +30 days
            ..env.ledger().get()
        });

        client.unstake(&staker, &600_i128);

        let events = env.events().all();
        let (_, topics, data) = events.last().unwrap();

        assert_eq!(
            topics,
            soroban_sdk::vec![&env, CONTRACT_TOPIC.into_val(&env), topic_unstake().into_val(&env)]
        );

        let payload: UnstakeEventData = data.into_val(&env);
        assert_eq!(payload.staker,    staker);
        assert_eq!(payload.amount,    600);
        assert_eq!(payload.remaining, 400);   // 1000 - 600
        assert!(payload.reward >= 0,  "reward must be non-negative");
        assert_eq!(payload.timestamp, 1_700_000_000 + 30 * 24 * 60 * 60);
    }

    #[test]
    fn full_unstake_leaves_zero_remaining() {
        let env = setup_env();
        let (client, staker) = setup_with_stake(&env, 500);

        client.unstake(&staker, &500_i128);

        let balance = client.get_stake(&staker);
        assert_eq!(balance, 0);

        let events = env.events().all();
        let (_, _, data) = events.last().unwrap();
        let payload: UnstakeEventData = data.into_val(&env);

        assert_eq!(payload.remaining, 0);
    }

    #[test]
    #[should_panic(expected = "insufficient staked balance")]
    fn unstake_more_than_staked_panics() {
        let env = setup_env();
        let (client, staker) = setup_with_stake(&env, 500);
        client.unstake(&staker, &1_000_i128); // more than the 500 staked
    }

    #[test]
    #[should_panic(expected = "unstake amount must be greater than zero")]
    fn unstake_zero_panics() {
        let env = setup_env();
        let (client, staker) = setup_with_stake(&env, 500);
        client.unstake(&staker, &0_i128);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Section 3 — Schema consistency tests
// Verify that every event topic is unique and that topic symbols are correct.
// ─────────────────────────────────────────────────────────────────────────────

mod event_schema_tests {
    use super::*;

    #[test]
    fn all_operation_topics_are_distinct() {
        let t_init    = topic_initialize();
        let t_stake   = topic_stake();
        let t_unstake = topic_unstake();

        assert_ne!(t_init,    t_stake,   "initialize and stake topics must differ");
        assert_ne!(t_init,    t_unstake, "initialize and unstake topics must differ");
        assert_ne!(t_stake,   t_unstake, "stake and unstake topics must differ");
    }

    #[test]
    fn contract_topic_is_stable() {
        // Ensures nobody accidentally changes the root topic, which would
        // break off-chain indexers subscribed to it.
        let env = setup_env();
        assert_eq!(
            CONTRACT_TOPIC.to_string().as_str(),
            "STAKING",
        );
    }
}