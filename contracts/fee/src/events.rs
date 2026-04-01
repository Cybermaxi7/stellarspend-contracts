use soroban_sdk::{symbol_short, Address, Env, Symbol};

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
