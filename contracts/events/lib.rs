use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

// ─── Operation Types ──────────────────────────────────────────────────────────

/// Every event topic includes an operation type so subscribers can filter
/// without deserialising the full data payload.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum OperationType {
    Initialize,
    Stake,
    Unstake,
    ClaimReward,
}

// ─── Event Topics ─────────────────────────────────────────────────────────────
// Soroban events are identified by a (contract_id, topics, data) triple.
// We use a two-element topic vec: [operation_symbol, contract_symbol]
// This lets off-chain indexers filter by operation cheaply.

pub const CONTRACT_TOPIC: Symbol = symbol_short!("STAKING");

pub fn topic_initialize() -> Symbol { symbol_short!("INIT")      }
pub fn topic_stake()       -> Symbol { symbol_short!("STAKE")     }
pub fn topic_unstake()     -> Symbol { symbol_short!("UNSTAKE")   }
pub fn topic_reward()      -> Symbol { symbol_short!("REWARD")    }

// ─── Event Payloads ───────────────────────────────────────────────────────────

/// Emitted once when the contract is first initialised.
///
/// Fields
/// - `admin`        : address that initialised the contract
/// - `reward_rate`  : configured reward rate (basis points, e.g. 1200 = 12 %)
/// - `min_stake`    : minimum stake amount enforced by the contract
/// - `timestamp`    : ledger timestamp at the time of initialisation
#[contracttype]
#[derive(Clone, Debug)]
pub struct InitializeEventData {
    pub admin:       Address,
    pub reward_rate: u32,
    pub min_stake:   i128,
    pub timestamp:   u64,
}

/// Emitted every time a user stakes tokens.
///
/// Fields
/// - `staker`     : address of the user staking
/// - `amount`     : tokens locked in this operation
/// - `total`      : user's cumulative staked balance after this operation
/// - `timestamp`  : ledger timestamp
#[contracttype]
#[derive(Clone, Debug)]
pub struct StakeEventData {
    pub staker:    Address,
    pub amount:    i128,
    pub total:     i128,
    pub timestamp: u64,
}

/// Emitted every time a user unstakes tokens.
///
/// Fields
/// - `staker`      : address of the user unstaking
/// - `amount`      : tokens unlocked in this operation
/// - `reward`      : reward tokens distributed alongside the principal
/// - `remaining`   : user's staked balance after this operation
/// - `timestamp`   : ledger timestamp
#[contracttype]
#[derive(Clone, Debug)]
pub struct UnstakeEventData {
    pub staker:    Address,
    pub amount:    i128,
    pub reward:    i128,
    pub remaining: i128,
    pub timestamp: u64,
}

// ─── Emit Helpers ─────────────────────────────────────────────────────────────
// Each public function in lib.rs calls one of these helpers so event emission
// is always consistent — same topic ordering, same schema version.

/// Emit the contract initialisation event.
pub fn emit_initialize(env: &Env, data: InitializeEventData) {
    validate_initialize_event(&data);
    env.events().publish(
        (CONTRACT_TOPIC, topic_initialize()),
        data,
    );
}

/// Emit a stake event.
pub fn emit_stake(env: &Env, data: StakeEventData) {
    validate_stake_event(&data);
    env.events().publish(
        (CONTRACT_TOPIC, topic_stake()),
        data,
    );
}

/// Emit an unstake event.
pub fn emit_unstake(env: &Env, data: UnstakeEventData) {
    validate_unstake_event(&data);
    env.events().publish(
        (CONTRACT_TOPIC, topic_unstake()),
        data,
    );
}

// ─── Validation ───────────────────────────────────────────────────────────────
// Validation is kept in this module so tests can call it directly without
// going through the full contract entry points.

/// Panics if the InitializeEventData is invalid.
/// Called by emit_initialize before publishing.
pub fn validate_initialize_event(data: &InitializeEventData) {
    assert!(
        data.reward_rate > 0,
        "event validation: reward_rate must be greater than zero"
    );
    assert!(
        data.min_stake > 0,
        "event validation: min_stake must be greater than zero"
    );
}

/// Panics if the StakeEventData is invalid.
pub fn validate_stake_event(data: &StakeEventData) {
    assert!(
        data.amount > 0,
        "event validation: stake amount must be greater than zero"
    );
    assert!(
        data.total >= data.amount,
        "event validation: total staked cannot be less than the staked amount"
    );
}

/// Panics if the UnstakeEventData is invalid.
pub fn validate_unstake_event(data: &UnstakeEventData) {
    assert!(
        data.amount > 0,
        "event validation: unstake amount must be greater than zero"
    );
    assert!(
        data.reward >= 0,
        "event validation: reward cannot be negative"
    );
    assert!(
        data.remaining >= 0,
        "event validation: remaining balance cannot be negative"
    );
}