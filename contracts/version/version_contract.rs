use soroban_sdk::{contract, contractimpl, Env, String};

#[contract]
pub struct VersionContract;

const CONTRACT_VERSION: &str = "1.0.0";

#[contractimpl]
impl VersionContract {
    /// Returns the contract version
    ///
    /// This function does not require authentication.
    pub fn get_version(env: Env) -> String {
        String::from_str(&env, CONTRACT_VERSION)
    }
}