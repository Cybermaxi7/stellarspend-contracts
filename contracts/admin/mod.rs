use soroban_sdk::{Address, Env};

mod storage;

pub fn initialize_admin(env: &Env, admin: &Address) {
    storage::set_admin(env, admin);
}

pub fn get_admin(env: &Env) -> Address {
    storage::get_admin(env)
}