use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol};

use crate::storage::{DEFAULT_FEE_BPS, DEFAULT_MIN_FEE};
use crate::utils::format_amount;

pub struct FeeEvents;

impl FeeEvents {
    pub fn fee_collected(env: &Env, user: &Address, amount: i128) {
        let topics = (symbol_short!("fee"), symbol_short!("collect"));
        env.events().publish(topics, (user.clone(), amount));
    }

    pub fn fee_escrowed(env: &Env, payer: &Address, amount: i128, cycle: u64) {
        let topics = (symbol_short!("fee"), symbol_short!("escrowed"));
        env.events().publish(topics, (payer.clone(), amount, cycle));
    }

    pub fn fee_batched(
        env: &Env,
        payer: &Address,
        total_amount: i128,
        batch_size: u32,
        cycle: u64,
    ) {
        let topics = (symbol_short!("fee"), symbol_short!("batched"));
        env.events()
            .publish(topics, (payer.clone(), total_amount, batch_size, cycle));
    }

    pub fn fee_released(env: &Env, cycle: u64, amount: i128, treasury: &Address) {
        let topics = (symbol_short!("fee"), symbol_short!("released"));
        env.events()
            .publish(topics, (cycle, amount, treasury.clone()));
    }

    pub fn fee_rolled(env: &Env, from_cycle: u64, to_cycle: u64, amount: i128) {
        let topics = (symbol_short!("fee"), symbol_short!("rollover"));
        env.events().publish(topics, (from_cycle, to_cycle, amount));
    }

    pub fn locked(env: &Env) {
        let topics = (symbol_short!("fee"), symbol_short!("locked"));
        env.events().publish(topics, ());
    }

    pub fn unlocked(env: &Env) {
        let topics = (symbol_short!("fee"), symbol_short!("unlocked"));
        env.events().publish(topics, ());
    }

    pub fn fee_bps_updated(env: &Env, fee_bps: u32) {
        let topics = (symbol_short!("fee"), symbol_short!("config"));
        env.events()
            .publish(topics, (symbol_short!("bps"), fee_bps));
    }

    pub fn treasury_updated(env: &Env, treasury: &Address) {
        let topics = (symbol_short!("fee"), symbol_short!("config"));
        env.events()
            .publish(topics, (symbol_short!("treasury"), treasury.clone()));
    }

    pub fn min_fee_updated(env: &Env, min_fee: i128) {
        let topics = (symbol_short!("fee"), symbol_short!("config"));
        env.events()
            .publish(topics, (symbol_short!("min_fee"), min_fee));
    }

    pub fn fee_reconciled(env: &Env, stored: i128, calculated: i128) {
        let topics = (symbol_short!("fee"), symbol_short!("recon"));
        env.events().publish(topics, (stored, calculated));
    }

    pub fn fee_discrepancy(env: &Env, stored: i128, calculated: i128, discrepancy: i128) {
        let topics = (symbol_short!("fee"), symbol_short!("discrep"));
        env.events()
            .publish(topics, (stored, calculated, discrepancy));
    }
}

pub struct TierEvents;

impl TierEvents {
    /// Emitted when an admin assigns a tier to a user.
    pub fn tier_set(env: &Env, admin: &Address, user: &Address, tier: &Symbol) {
        let topics = (symbol_short!("tier"), symbol_short!("set"));
        env.events()
            .publish(topics, (admin.clone(), user.clone(), tier.clone()));
    }

    /// Emitted when an admin removes a tier from a user.
    pub fn tier_removed(env: &Env, admin: &Address, user: &Address) {
        let topics = (symbol_short!("tier"), symbol_short!("removed"));
        env.events().publish(topics, (admin.clone(), user.clone()));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct FeeResetEventData {
    pub admin: Address,
    pub fee_bps: u32,
    pub min_fee: i128,
    pub formatted_min_fee: String,
}

pub struct ConfigEvents;

impl ConfigEvents {
    /// Emitted when an admin resets fee configuration to defaults.
    pub fn fee_reset(env: &Env, admin: &Address) {
        let topics = (symbol_short!("fee"), symbol_short!("reset"));
        env.events().publish(
            topics,
            FeeResetEventData {
                admin: admin.clone(),
                fee_bps: DEFAULT_FEE_BPS,
                min_fee: DEFAULT_MIN_FEE,
                formatted_min_fee: format_amount(env, DEFAULT_MIN_FEE),
            },
        );
    }
}
