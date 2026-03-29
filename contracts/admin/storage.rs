use soroban_sdk::{Address, Env, Symbol};

const ADMIN_KEY: Symbol = Symbol::short("ADMIN");

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&ADMIN_KEY, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&ADMIN_KEY)
        .expect("Admin not initialized")
}