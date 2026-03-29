use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Symbol, Vec, Map, String};
use crate::errors::StellarSpendError;

#[derive(Clone)]
#[contracttype]
pub struct RefundRequest {
    pub id: u64,
    pub transaction_id: u64,
    pub requester: Address,
    pub original_recipient: Address,
    pub amount: i128,
    pub reason: String,
    pub status: RefundStatus,
    pub created_at: u64,
    pub processed_at: Option<u64>,
    pub processed_by: Option<Address>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum RefundStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Processed = 3,
    Expired = 4,
}

#[derive(Clone)]
#[contracttype]
pub struct RefundConfig {
    pub refund_window_seconds: u64,
    pub auto_approve_threshold: i128,
    pub admin_required_threshold: i128,
    pub max_refund_reason_length: u32,
    pub enabled: bool,
}

#[derive(Clone)]
#[contracttype]
pub struct RefundEligibility {
    pub is_eligible: bool,
    pub reason: String,
    pub requires_admin: bool,
}

#[contract]
pub struct RefundsContract;

#[contractimpl]
impl RefundsContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, StellarSpendError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        
        let config = RefundConfig {
            refund_window_seconds: 86400, // 24 hours
            auto_approve_threshold: 1000, // Auto-approve refunds <= 1000
            admin_required_threshold: 10000, // Require admin approval for refunds > 10000
            max_refund_reason_length: 500,
            enabled: true,
        };
        env.storage().instance().set(&DataKey::Config, &config);
        
        env.storage().instance().set(&DataKey::NextRefundId, &1u64);
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, StellarSpendError::NotInitialized))
    }

    pub fn set_config(env: Env, caller: Address, config: RefundConfig) {
        Self::require_admin(&env, &caller);
        env.storage().instance().set(&DataKey::Config, &config);
        RefundEvents::config_updated(&env, &config);
    }

    pub fn get_config(env: Env) -> RefundConfig {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .unwrap_or_else(|| panic_with_error!(&env, StellarSpendError::NotInitialized))
    }

    pub fn request_refund(
        env: Env,
        caller: Address,
        transaction_id: u64,
        original_recipient: Address,
        amount: i128,
        reason: String,
    ) -> u64 {
        caller.require_auth();

        let config = Self::get_config(env.clone());
        if !config.enabled {
            panic_with_error!(&env, StellarSpendError::Paused);
        }

        if amount <= 0 {
            panic_with_error!(&env, StellarSpendError::InvalidAmount);
        }

        if reason.len() > config.max_refund_reason_length as usize {
            panic_with_error!(&env, StellarSpendError::InvalidInput);
        }

        let eligibility = Self::check_refund_eligibility(
            env.clone(),
            &caller,
            transaction_id,
            amount,
        );

        if !eligibility.is_eligible {
            panic_with_error!(&env, StellarSpendError::InvalidTransaction);
        }

        let refund_id = Self::next_refund_id(&env);
        
        let status = if amount <= config.auto_approve_threshold {
            RefundStatus::Approved
        } else if amount > config.admin_required_threshold {
            RefundStatus::Pending
        } else {
            RefundStatus::Approved
        };

        let refund_request = RefundRequest {
            id: refund_id,
            transaction_id,
            requester: caller.clone(),
            original_recipient,
            amount,
            reason,
            status,
            created_at: env.ledger().timestamp(),
            processed_at: None,
            processed_by: None,
        };

        env.storage()
            .instance()
            .set(&DataKey::RefundRequest(refund_id), &refund_request);

        RefundEvents::refund_requested(&env, &refund_request);

        refund_id
    }

    pub fn approve_refund(env: Env, caller: Address, refund_id: u64) {
        Self::require_admin(&env, &caller);

        let mut refund_request: RefundRequest = env
            .storage()
            .instance()
            .get(&DataKey::RefundRequest(refund_id))
            .unwrap_or_else(|| panic_with_error!(&env, StellarSpendError::NotFound));

        if refund_request.status != RefundStatus::Pending {
            panic_with_error!(&env, StellarSpendError::InvalidState);
        }

        refund_request.status = RefundStatus::Approved;
        refund_request.processed_at = Some(env.ledger().timestamp());
        refund_request.processed_by = Some(caller.clone());

        env.storage()
            .instance()
            .set(&DataKey::RefundRequest(refund_id), &refund_request);

        RefundEvents::refund_approved(&env, &refund_request, &caller);
    }

    pub fn reject_refund(env: Env, caller: Address, refund_id: u64, reason: String) {
        Self::require_admin(&env, &caller);

        let mut refund_request: RefundRequest = env
            .storage()
            .instance()
            .get(&DataKey::RefundRequest(refund_id))
            .unwrap_or_else(|| panic_with_error!(&env, StellarSpendError::NotFound));

        if refund_request.status != RefundStatus::Pending {
            panic_with_error!(&env, StellarSpendError::InvalidState);
        }

        refund_request.status = RefundStatus::Rejected;
        refund_request.processed_at = Some(env.ledger().timestamp());
        refund_request.processed_by = Some(caller.clone());

        env.storage()
            .instance()
            .set(&DataKey::RefundRequest(refund_id), &refund_request);

        RefundEvents::refund_rejected(&env, &refund_request, &caller, &reason);
    }

    pub fn process_refund(env: Env, caller: Address, refund_id: u64) {
        caller.require_auth();

        let mut refund_request: RefundRequest = env
            .storage()
            .instance()
            .get(&DataKey::RefundRequest(refund_id))
            .unwrap_or_else(|| panic_with_error!(&env, StellarSpendError::NotFound));

        if refund_request.status != RefundStatus::Approved {
            panic_with_error!(&env, StellarSpendError::InvalidState);
        }

        if Self::has_refund_been_processed(&env, refund_id) {
            panic_with_error!(&env, StellarSpendError::AlreadyExists);
        }

        let config = Self::get_config(env.clone());
        let current_time = env.ledger().timestamp();

        if current_time > refund_request.created_at + config.refund_window_seconds {
            refund_request.status = RefundStatus::Expired;
            env.storage()
                .instance()
                .set(&DataKey::RefundRequest(refund_id), &refund_request);
            panic_with_error!(&env, StellarSpendError::Expired);
        }

        Self::execute_refund(&env, &refund_request);

        refund_request.status = RefundStatus::Processed;
        refund_request.processed_at = Some(current_time);
        refund_request.processed_by = Some(caller.clone());

        env.storage()
            .instance()
            .set(&DataKey::RefundRequest(refund_id), &refund_request);

        env.storage()
            .instance()
            .set(&DataKey::ProcessedRefund(refund_id), &true);

        RefundEvents::refund_processed(&env, &refund_request, &caller);
    }

    pub fn get_refund_request(env: Env, refund_id: u64) -> Option<RefundRequest> {
        env.storage().instance().get(&DataKey::RefundRequest(refund_id))
    }

    pub fn get_refund_status(env: Env, refund_id: u64) -> Option<RefundStatus> {
        env.storage()
            .instance()
            .get(&DataKey::RefundRequest(refund_id))
            .map(|request: RefundRequest| request.status)
    }

    pub fn check_refund_eligibility(
        env: Env,
        requester: &Address,
        transaction_id: u64,
        amount: i128,
    ) -> RefundEligibility {
        let config = Self::get_config(env.clone());
        let current_time = env.ledger().timestamp();

        if Self::has_refund_for_transaction(&env, transaction_id) {
            return RefundEligibility {
                is_eligible: false,
                reason: String::from_str(&env, "Refund already processed for this transaction"),
                requires_admin: false,
            };
        }

        if current_time > Self::get_transaction_timestamp(&env, transaction_id) + config.refund_window_seconds {
            return RefundEligibility {
                is_eligible: false,
                reason: String::from_str(&env, "Refund window expired"),
                requires_admin: false,
            };
        }

        let requires_admin = amount > config.admin_required_threshold;

        RefundEligibility {
            is_eligible: true,
            reason: String::from_str(&env, "Eligible for refund"),
            requires_admin,
        }
    }

    pub fn get_user_refunds(env: Env, user: Address) -> Vec<RefundRequest> {
        let mut refunds = Vec::new(&env);
        let mut current_id = 1u64;

        while let Some(refund) = env.storage().instance().get(&DataKey::RefundRequest(current_id)) {
            if refund.requester == user {
                refunds.push_back(refund);
            }
            current_id += 1;
        }

        refunds
    }

    pub fn get_pending_refunds(env: Env) -> Vec<RefundRequest> {
        let mut refunds = Vec::new(&env);
        let mut current_id = 1u64;

        while let Some(refund) = env.storage().instance().get(&DataKey::RefundRequest(current_id)) {
            if refund.status == RefundStatus::Pending {
                refunds.push_back(refund);
            }
            current_id += 1;
        }

        refunds
    }
}

impl RefundsContract {
    fn require_admin(env: &Env, caller: &Address) {
        caller.require_auth();
        let admin = Self::get_admin(env.clone());
        if caller != &admin {
            panic_with_error!(env, StellarSpendError::AdminRequired);
        }
    }

    fn next_refund_id(env: &Env) -> u64 {
        let id = env
            .storage()
            .instance()
            .get(&DataKey::NextRefundId)
            .unwrap_or(1u64);
        
        env.storage()
            .instance()
            .set(&DataKey::NextRefundId, &(id + 1));
        
        id
    }

    fn has_refund_been_processed(env: &Env, refund_id: u64) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::ProcessedRefund(refund_id))
            .unwrap_or(false)
    }

    fn has_refund_for_transaction(env: &Env, transaction_id: u64) -> bool {
        let mut current_id = 1u64;
        while let Some(refund) = env.storage().instance().get(&DataKey::RefundRequest(current_id)) {
            if refund.transaction_id == transaction_id && refund.status == RefundStatus::Processed {
                return true;
            }
            current_id += 1;
        }
        false
    }

    fn get_transaction_timestamp(env: &Env, transaction_id: u64) -> u64 {
        env.ledger().timestamp()
    }

    fn execute_refund(env: &Env, refund_request: &RefundRequest) {
        let requester_balance = Self::get_balance(env, &refund_request.requester);
        let new_balance = requester_balance
            .checked_add(refund_request.amount)
            .unwrap_or_else(|| panic_with_error!(env, StellarSpendError::Overflow));

        env.storage()
            .persistent()
            .set(&DataKey::Balance(refund_request.requester.clone()), &new_balance);
    }

    fn get_balance(env: &Env, user: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user.clone()))
            .unwrap_or(0)
    }
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Config,
    NextRefundId,
    RefundRequest(u64),
    ProcessedRefund(u64),
    Balance(Address),
}

pub struct RefundEvents;

impl RefundEvents {
    pub fn refund_requested(env: &Env, refund: &RefundRequest) {
        let topics = (
            soroban_sdk::symbol_short!("refund"),
            soroban_sdk::symbol_short!("requested"),
        );
        let data = (
            refund.id,
            refund.transaction_id,
            refund.requester.clone(),
            refund.original_recipient.clone(),
            refund.amount,
            refund.reason.clone(),
            refund.status as u32,
            refund.created_at,
        );
        env.events().publish(topics, data);
    }

    pub fn refund_approved(env: &Env, refund: &RefundRequest, approved_by: &Address) {
        let topics = (
            soroban_sdk::symbol_short!("refund"),
            soroban_sdk::symbol_short!("approved"),
        );
        let data = (
            refund.id,
            refund.requester.clone(),
            refund.amount,
            approved_by.clone(),
            env.ledger().timestamp(),
        );
        env.events().publish(topics, data);
    }

    pub fn refund_rejected(env: &Env, refund: &RefundRequest, rejected_by: &Address, reason: &String) {
        let topics = (
            soroban_sdk::symbol_short!("refund"),
            soroban_sdk::symbol_short!("rejected"),
        );
        let data = (
            refund.id,
            refund.requester.clone(),
            refund.amount,
            rejected_by.clone(),
            reason.clone(),
            env.ledger().timestamp(),
        );
        env.events().publish(topics, data);
    }

    pub fn refund_processed(env: &Env, refund: &RefundRequest, processed_by: &Address) {
        let topics = (
            soroban_sdk::symbol_short!("refund"),
            soroban_sdk::symbol_short!("processed"),
        );
        let data = (
            refund.id,
            refund.transaction_id,
            refund.requester.clone(),
            refund.original_recipient.clone(),
            refund.amount,
            processed_by.clone(),
            env.ledger().timestamp(),
        );
        env.events().publish(topics, data);
    }

    pub fn config_updated(env: &Env, config: &RefundConfig) {
        let topics = (
            soroban_sdk::symbol_short!("refund"),
            soroban_sdk::symbol_short!("config_updated"),
        );
        let data = (
            config.refund_window_seconds,
            config.auto_approve_threshold,
            config.admin_required_threshold,
            config.max_refund_reason_length,
            config.enabled,
        );
        env.events().publish(topics, data);
    }
}
