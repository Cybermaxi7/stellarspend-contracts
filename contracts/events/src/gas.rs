//! # events.rs
//!
//! Standardised event schema for the staking contract.
//!
//! ## Gas optimizations applied
//! - Topics are emitted as a fixed 2-tuple `(CONTRACT_TOPIC, op_topic)` —
//!   Soroban charges per topic element, so we use the minimum (2) that still
//!   allows off-chain filtering.
//! - Payload structs carry only primitive / already-owned values so the emit
//!   helpers never perform extra heap allocations before publishing.
//! - `validate_*` guards are `#[inline]` so the compiler can fold them into
//!   the caller and eliminate the function-call overhead on the hot path.

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

// ─── Contract-level topic ─────────────────────────────────────────────────────

pub const CONTRACT_TOPIC: Symbol = symbol_short!("STAKING");

// ─── Per-operation topics ─────────────────────────────────────────────────────

#[inline(always)] pub fn topic_initialize() -> Symbol { symbol_short!("INIT")    }
#[inline(always)] pub fn topic_stake()       -> Symbol { symbol_short!("STAKE")  }
#[inline(always)] pub fn topic_unstake()     -> Symbol { symbol_short!("UNSTK")  }
#[inline(always)] pub fn topic_escrow()      -> Symbol { symbol_short!("ESCROW") }
#[inline(always)] pub fn topic_batch()       -> Symbol { symbol_short!("BATCH")  }

// ─── Event payloads ───────────────────────────────────────────────────────────

/// Emitted once at contract initialisation.
#[contracttype]
#[derive(Clone, Debug)]
pub struct InitializeEventData {
    pub admin:       Address,
    pub reward_rate: u32,
    pub min_stake:   i128,
    pub timestamp:   u64,
}

/// Emitted on every successful stake call.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StakeEventData {
    pub staker:    Address,
    pub amount:    i128,
    pub total:     i128,
    pub timestamp: u64,
}

/// Emitted on every successful unstake call.
#[contracttype]
#[derive(Clone, Debug)]
pub struct UnstakeEventData {
    pub staker:    Address,
    pub amount:    i128,
    pub reward:    i128,
    pub remaining: i128,
    pub timestamp: u64,
}

/// Emitted when funds are locked into escrow.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowLockedEventData {
    pub depositor:   Address,
    pub beneficiary: Address,
    pub amount:      i128,
    pub unlock_ts:   u64,
    pub timestamp:   u64,
}

/// Emitted when escrow funds are released to the beneficiary.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EscrowReleasedEventData {
    pub beneficiary: Address,
    pub amount:      i128,
    pub timestamp:   u64,
}

/// Emitted once per batch-reward run — summarises the entire batch.
///
/// ## Gas rationale
/// Emitting one summary event instead of N per-user events saves
/// `(N - 1) * event_base_cost` instructions per batch run. Off-chain
/// indexers reconcile individual amounts from the transaction's XDR
/// rather than relying on individual events.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BatchRewardEventData {
    /// Number of addresses that received a reward this run
    pub recipients:    u32,
    /// Sum of all reward tokens distributed
    pub total_rewards: i128,
    /// Ledger timestamp of the batch run
    pub timestamp:     u64,
}

// ─── Emit helpers ─────────────────────────────────────────────────────────────

pub fn emit_initialize(env: &Env, data: InitializeEventData) {
    validate_initialize_event(&data);
    env.events().publish((CONTRACT_TOPIC, topic_initialize()), data);
}

pub fn emit_stake(env: &Env, data: StakeEventData) {
    validate_stake_event(&data);
    env.events().publish((CONTRACT_TOPIC, topic_stake()), data);
}

pub fn emit_unstake(env: &Env, data: UnstakeEventData) {
    validate_unstake_event(&data);
    env.events().publish((CONTRACT_TOPIC, topic_unstake()), data);
}

pub fn emit_escrow_locked(env: &Env, data: EscrowLockedEventData) {
    validate_escrow_locked_event(&data);
    env.events().publish((CONTRACT_TOPIC, topic_escrow()), data);
}

pub fn emit_escrow_released(env: &Env, data: EscrowReleasedEventData) {
    validate_escrow_released_event(&data);
    env.events().publish((CONTRACT_TOPIC, topic_escrow()), data);
}

pub fn emit_batch_reward(env: &Env, data: BatchRewardEventData) {
    validate_batch_reward_event(&data);
    env.events().publish((CONTRACT_TOPIC, topic_batch()), data);
}

// ─── Validation (inlined for hot-path calls) ─────────────────────────────────

#[inline]
pub fn validate_initialize_event(data: &InitializeEventData) {
    assert!(data.reward_rate > 0, "event validation: reward_rate must be > 0");
    assert!(data.min_stake   > 0, "event validation: min_stake must be > 0");
}

#[inline]
pub fn validate_stake_event(data: &StakeEventData) {
    assert!(data.amount > 0,            "event validation: stake amount must be > 0");
    assert!(data.total  >= data.amount, "event validation: total < amount — impossible state");
}

#[inline]
pub fn validate_unstake_event(data: &UnstakeEventData) {
    assert!(data.amount    > 0,  "event validation: unstake amount must be > 0");
    assert!(data.reward    >= 0, "event validation: reward cannot be negative");
    assert!(data.remaining >= 0, "event validation: remaining cannot be negative");
}

#[inline]
pub fn validate_escrow_locked_event(data: &EscrowLockedEventData) {
    assert!(data.amount    > 0,              "event validation: escrow amount must be > 0");
    assert!(data.unlock_ts > data.timestamp, "event validation: unlock_ts must be in the future");
}

#[inline]
pub fn validate_escrow_released_event(data: &EscrowReleasedEventData) {
    assert!(data.amount > 0, "event validation: released amount must be > 0");
}

#[inline]
pub fn validate_batch_reward_event(data: &BatchRewardEventData) {
    assert!(data.recipients    > 0, "event validation: batch must have at least one recipient");
    assert!(data.total_rewards > 0, "event validation: total_rewards must be > 0");
}