use crate::types::{BatchResult, NotificationPayload};
use soroban_sdk::{symbol_short, Env, Vec};

pub fn execute_dispatch(env: Env, payloads: Vec<NotificationPayload>) -> BatchResult {
    let mut success_count = 0;
    let mut failures = Vec::new(&env);

    for payload in payloads.iter() {
        // Requirement: Handle partial failure gracefully
        // We consider an empty message a "soft failure" instead of panicking
        if !payload.message.is_empty() {
            // Requirement: Emit events for notification delivery
            env.events().publish(
                (symbol_short!("notif"), payload.user.clone()),
                payload.message,
            );
            success_count += 1;
        } else {
            // If it fails, add the user to the failure list
            failures.push_back(payload.user);
        }
    }

    BatchResult {
        successful_count: success_count,
        failed_addresses: failures,
    }
}
