#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Version,
}

#[contract]
pub struct UpgradeableContract;

#[contractimpl]
impl UpgradeableContract {
    pub fn __constructor(e: Env, admin: Address) {
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage().instance().set(&DataKey::Version, &1u32);
    }

    pub fn version(e: Env) -> u32 {
        e.storage().instance().get(&DataKey::Version).unwrap_or(0)
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>, new_version: u32) {
        let admin: Address = e.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        admin.require_auth();

        // [SEC-UPGRADE-01] Version check: Prevent downgrades
        let current_version = Self::version(e.clone());
        if new_version <= current_version {
            panic!("Upgrade failed: new version must be greater than current version");
        }

        // [SEC-UPGRADE-02] Migration validation: Ensure critical state remains intact
        if !e.storage().instance().has(&DataKey::Admin) {
            panic!("Upgrade failed: critical state validation failed (Admin missing)");
        }

        // Update to new version
        e.storage().instance().set(&DataKey::Version, &new_version);

        e.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        
        e.events()
            .publish((symbol_short!("upgrade"), current_version, new_version), new_wasm_hash);
    }
}

mod test;

#[cfg(test)]
mod upgrade_tests;
