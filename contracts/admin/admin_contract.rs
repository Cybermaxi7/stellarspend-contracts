use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::{ContractUtils, DataKey};

#[contract]
pub struct AdminContract;

#[contractimpl]
impl AdminContract {
    /// Initialize contract with admin
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Retrieve the stored admin address
    ///
    /// This function does not require authentication.
    pub fn get_admin(env: Env) -> Address {
        ContractUtils::get_admin(&env)
    }
}