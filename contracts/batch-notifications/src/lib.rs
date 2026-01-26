#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

mod logic;
mod types;

#[cfg(test)]
mod test;

use crate::types::{BatchResult, NotificationPayload};

#[contract]
pub struct BatchNotificationContract;

#[contractimpl]
impl BatchNotificationContract {
    pub fn batch_notify(
        env: Env,
        admin: Address,
        payloads: Vec<NotificationPayload>,
    ) -> BatchResult {
        // Requirement: Validate user/admin addresses
        admin.require_auth();

        // Run the batch logic
        logic::execute_dispatch(env, payloads)
    }
}
