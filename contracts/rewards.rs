use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env,
};

#[derive(Clone)]
#[contracttype]
pub enum RewardDataKey {
    Admin,
    BaseRewardAmount,
    RewardMultiplier,
    UserMilestone(Address, u32), // User, Milestone ID -> bool (true if rewarded)
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RewardError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    DuplicateReward = 4,
    InvalidMilestone = 5,
}

pub struct RewardEvents;

impl RewardEvents {
    pub fn reward_distributed(env: &Env, user: &Address, milestone_id: u32, amount: i128) {
        let topics = (symbol_short!("reward"), symbol_short!("issued"));
        env.events().publish(
            topics,
            (user.clone(), milestone_id, amount, env.ledger().timestamp()),
        );
    }
}

pub fn initialize_rewards(env: &Env, admin: Address, base_reward: i128, multiplier: i128) {
    if env.storage().instance().has(&RewardDataKey::Admin) {
        panic_with_error!(env, RewardError::AlreadyInitialized);
    }
    env.storage().instance().set(&RewardDataKey::Admin, &admin);
    env.storage()
        .instance()
        .set(&RewardDataKey::BaseRewardAmount, &base_reward);
    env.storage()
        .instance()
        .set(&RewardDataKey::RewardMultiplier, &multiplier);
}

pub fn require_admin(env: &Env, caller: &Address) {
    caller.require_auth();
    let admin: Address = env
        .storage()
        .instance()
        .get(&RewardDataKey::Admin)
        .unwrap_or_else(|| panic_with_error!(env, RewardError::NotInitialized));
    if admin != *caller {
        panic_with_error!(env, RewardError::Unauthorized);
    }
}

pub fn calculate_reward(env: &Env, milestone_id: u32) -> i128 {
    let base_reward: i128 = env
        .storage()
        .instance()
        .get(&RewardDataKey::BaseRewardAmount)
        .unwrap_or_else(|| panic_with_error!(env, RewardError::NotInitialized));

    let multiplier: i128 = env
        .storage()
        .instance()
        .get(&RewardDataKey::RewardMultiplier)
        .unwrap_or_else(|| panic_with_error!(env, RewardError::NotInitialized));

    base_reward
        .checked_add(multiplier.checked_mul(milestone_id as i128).unwrap_or(0))
        .unwrap_or(base_reward)
}

pub fn distribute_reward(env: &Env, admin: Address, user: Address, milestone_id: u32) -> i128 {
    require_admin(env, &admin);

    if milestone_id == 0 {
        panic_with_error!(env, RewardError::InvalidMilestone);
    }

    let milestone_key = RewardDataKey::UserMilestone(user.clone(), milestone_id);
    let has_rewarded: bool = env
        .storage()
        .persistent()
        .get(&milestone_key)
        .unwrap_or(false);

    if has_rewarded {
        panic_with_error!(env, RewardError::DuplicateReward);
    }

    let reward_amount = calculate_reward(env, milestone_id);

    env.storage().persistent().set(&milestone_key, &true);

    RewardEvents::reward_distributed(env, &user, milestone_id, reward_amount);

    reward_amount
}

#[contract]
pub struct RewardsContract;

#[contractimpl]
impl RewardsContract {
    pub fn initialize(env: Env, admin: Address, base_reward: i128, multiplier: i128) {
        initialize_rewards(&env, admin, base_reward, multiplier);
    }

    pub fn distribute_reward(env: Env, admin: Address, user: Address, milestone_id: u32) -> i128 {
        distribute_reward(&env, admin, user, milestone_id)
    }

    pub fn calculate_reward(env: Env, milestone_id: u32) -> i128 {
        calculate_reward(&env, milestone_id)
    }

    pub fn has_user_been_rewarded(env: Env, user: Address, milestone_id: u32) -> bool {
        let milestone_key = RewardDataKey::UserMilestone(user, milestone_id);
        env.storage()
            .persistent()
            .get(&milestone_key)
            .unwrap_or(false)
    }
}
