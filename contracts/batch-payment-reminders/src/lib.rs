#![no_std]

mod logic;
mod types;
mod validation;

#[cfg(test)]
mod test;

use crate::types::{BatchReminderResult, PaymentReminderRequest};
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct BatchPaymentRemindersContract;

#[contractimpl]
impl BatchPaymentRemindersContract {
    /// Send batch payment reminders to multiple users.
    ///
    /// Validates each (user, due_date); valid entries get a reminder_sent event,
    /// invalid ones are skipped and recorded in the result (partial failure handling).
    ///
    /// # Arguments
    /// * `admin` - Caller must authorize (admin).
    /// * `requests` - List of (user, due_date) reminder requests.
    /// # Returns
    /// * `BatchReminderResult` with successful_count and failed_addresses.
    pub fn dispatch_batch_reminders(
        env: Env,
        admin: Address,
        requests: Vec<PaymentReminderRequest>,
    ) -> BatchReminderResult {
        admin.require_auth();

        let batch_id = env.ledger().sequence() as u64;
        logic::execute_dispatch(env, batch_id, requests)
    }
}
