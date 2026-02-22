#![no_std]

mod types;
#[cfg(test)]
mod test;

use crate::types::{DataKey, RecurringPayment};
use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, Symbol};

#[contract]
pub struct RecurringPaymentContract;

#[contractimpl]
impl RecurringPaymentContract {
    /// Creates a new recurring payment.
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

        let mut count: u64 = env.storage().instance().get(&DataKey::PaymentCount).unwrap_or(0);
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

        env.storage().instance().set(&DataKey::Payment(count), &payment);
        env.storage().instance().set(&DataKey::PaymentCount, &count);

        // Emit creation event
        env.events().publish(
            (symbol_short!("recur"), symbol_short!("created"), count),
            sender,
        );

        count
    }

    /// Executes a recurring payment if the next execution time has passed.
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

        // Transfer tokens
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

        env.storage().instance().set(&DataKey::Payment(payment_id), &payment);

        // Emit execution event
        env.events().publish(
            (symbol_short!("recur"), symbol_short!("executed"), payment_id),
            (payment.amount, payment.next_execution),
        );
    }

    /// Cancels a recurring payment.
    pub fn cancel_payment(env: Env, payment_id: u64) {
        let mut payment: RecurringPayment = env
            .storage()
            .instance()
            .get(&DataKey::Payment(payment_id))
            .expect("Payment not found");

        payment.sender.require_auth();

        payment.active = false;
        env.storage().instance().set(&DataKey::Payment(payment_id), &payment);

        // Emit cancellation event
        env.events().publish(
            (symbol_short!("recur"), symbol_short!("canceled"), payment_id),
            payment.sender,
        );
    }

    /// Gets payment details.
    pub fn get_payment(env: Env, payment_id: u64) -> RecurringPayment {
        env.storage()
            .instance()
            .get(&DataKey::Payment(payment_id))
            .expect("Payment not found")
    }
}
