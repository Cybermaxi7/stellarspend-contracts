#![no_std]

#[cfg(test)]
mod test;
mod types;

use crate::types::{DataKey, RecurringPayment};
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env};

#[contract]
pub struct RecurringPaymentContract;

#[contractimpl]
impl RecurringPaymentContract {
    /// Creates a new recurring payment schedule.
    ///
    /// # Arguments
    /// * `sender`     - The address funding the payments (must authorize)
    /// * `recipient`  - The address that receives each payment
    /// * `token`      - The token contract address
    /// * `amount`     - Amount transferred on each execution (must be > 0)
    /// * `interval`   - Seconds between executions (must be > 0)
    /// * `start_time` - Ledger timestamp of the first allowed execution
    ///
    /// # Returns
    /// The unique payment ID assigned to this schedule.
    pub fn create_payment(
        env: Env,
        sender: Address,
        recipient: Address,
        token: Address,
        amount: i128,
        interval: u64,
        start_time: u64,
    ) -> u64 {
        sender.require_auth();

        if amount <= 0 {
            panic!("Amount must be positive");
        }
        if interval == 0 {
            panic!("Interval must be positive");
        }

        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PaymentCount)
            .unwrap_or(0);
        count += 1;

        let payment = RecurringPayment {
            sender: sender.clone(),
            recipient,
            token,
            amount,
            interval,
            next_execution: start_time,
            active: true,
        };

        env.storage()
            .instance()
            .set(&DataKey::Payment(count), &payment);
        env.storage().instance().set(&DataKey::PaymentCount, &count);

        env.events().publish(
            (symbol_short!("recur"), symbol_short!("created"), count),
            sender,
        );

        count
    }

    /// # Arguments
    /// * `payment_id` - The ID returned by `create_payment`
    pub fn execute_payment(env: Env, payment_id: u64) {
        let mut payment: RecurringPayment = env
            .storage()
            .instance()
            .get(&DataKey::Payment(payment_id))
            .expect("Payment not found");

        if !payment.active {
            panic!("Payment is not active");
        }

        let current_time = env.ledger().timestamp();
        if current_time < payment.next_execution {
            panic!("Too early for next execution");
        }

        // Transfer tokens from sender to recipient.
        let token_client = token::Client::new(&env, &payment.token);
        token_client.transfer(&payment.sender, &payment.recipient, &payment.amount);

        // Update next execution time
        payment.next_execution += payment.interval;

        // If the execution was delayed, we might want to skip or catch up.
        // For simplicity, we just add the interval to the scheduled time.
        // If current_time is way past next_execution, catch up.
        if payment.next_execution <= current_time {
            // Option 1: Catch up to the next interval in the future
            // (current_time - scheduled) / interval * interval + scheduled + interval
            let intervals_passed = (current_time - payment.next_execution) / payment.interval;
            payment.next_execution += (intervals_passed + 1) * payment.interval;
        }

        env.storage()
            .instance()
            .set(&DataKey::Payment(payment_id), &payment);

        env.events().publish(
            (
                symbol_short!("recur"),
                symbol_short!("executed"),
                payment_id,
            ),
            (payment.amount, payment.next_execution),
        );
    }

    /// Cancels a recurring payment. Only the original sender may cancel.
    ///
    /// # Arguments
    /// * `payment_id` - The ID returned by `create_payment`
    pub fn cancel_payment(env: Env, payment_id: u64) {
        let mut payment: RecurringPayment = env
            .storage()
            .instance()
            .get(&DataKey::Payment(payment_id))
            .expect("Payment not found");

        payment.sender.require_auth();

        if !payment.active {
            panic!("Payment is already canceled");
        }

        payment.active = false;
        env.storage()
            .instance()
            .set(&DataKey::Payment(payment_id), &payment);

        env.events().publish(
            (
                symbol_short!("recur"),
                symbol_short!("canceled"),
                payment_id,
            ),
            payment.sender,
        );
    }

    /// Returns the full details of a payment schedule.
    ///
    /// # Arguments
    /// * `payment_id` - The ID returned by `create_payment`
    pub fn get_payment(env: Env, payment_id: u64) -> RecurringPayment {
        env.storage()
            .instance()
            .get(&DataKey::Payment(payment_id))
            .expect("Payment not found")
    }
}
