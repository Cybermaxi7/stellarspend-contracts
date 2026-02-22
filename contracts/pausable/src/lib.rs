//! # Pausable Contract
//! 
//! Emergency pause functionality for StellarSpend contracts.
//! Allows admin to pause/unpause critical operations during emergencies.

#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, panic_with_error, Address, Env};

/// Storage keys for the pausable contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Contract admin
    Admin,
    /// Pause state (true = paused, false = active)
    Paused,
}

/// Error codes for pausable operations
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PausableError {
    /// Contract not initialized
    NotInitialized = 1,
    /// Caller is not authorized
    Unauthorized = 2,
    /// Contract is paused
    ContractPaused = 3,
    /// Contract is not paused
    ContractNotPaused = 4,
}

impl From<PausableError> for soroban_sdk::Error {
    fn from(e: PausableError) -> Self {
        soroban_sdk::Error::from_contract_error(e as u32)
    }
}

#[contract]
pub struct PausableContract;

#[contractimpl]
impl PausableContract {
    /// Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);

        env.events()
            .publish(("pausable", "initialized"), admin);
    }

    /// Pause the contract (admin only)
    pub fn pause(env: Env, caller: Address) {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        let is_paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);

        if is_paused {
            panic_with_error!(&env, PausableError::ContractPaused);
        }

        env.storage().instance().set(&DataKey::Paused, &true);

        env.events()
            .publish(("pausable", "paused"), caller);
    }

    /// Unpause the contract (admin only)
    pub fn unpause(env: Env, caller: Address) {
        caller.require_auth();
        Self::require_admin(&env, &caller);

        let is_paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);

        if !is_paused {
            panic_with_error!(&env, PausableError::ContractNotPaused);
        }

        env.storage().instance().set(&DataKey::Paused, &false);

        env.events()
            .publish(("pausable", "unpaused"), caller);
    }

    /// Check if the contract is paused
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized")
    }

    /// Update the admin address (current admin only)
    pub fn set_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        Self::require_admin(&env, &current_admin);

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        env.events()
            .publish(("pausable", "admin_changed"), (current_admin, new_admin));
    }

    /// Require that the caller is the admin
    pub fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Contract not initialized");

        if *caller != admin {
            panic_with_error!(env, PausableError::Unauthorized);
        }
    }

    /// Require that the contract is not paused
    pub fn require_not_paused(env: &Env) {
        let is_paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);

        if is_paused {
            panic_with_error!(env, PausableError::ContractPaused);
        }
    }
}

#[cfg(test)]
mod test;
