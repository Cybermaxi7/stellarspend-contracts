use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Symbol, Vec,
};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    LinkedWallet(Address),
    WalletVerificationNonce(Address),
    VerificationChallenge(Address, u64),
    WalletOwner(Address),
    VerificationStatus(Address),
}

#[derive(Clone)]
#[contracttype]
pub struct LinkedWalletInfo {
    pub wallet_address: Address,
    pub owner_address: Address,
    pub verification_timestamp: u64,
    pub is_verified: bool,
    pub verification_nonce: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct VerificationChallenge {
    pub challenge_id: u64,
    pub wallet_address: Address,
    pub challenger_address: Address,
    pub nonce: u64,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_completed: bool,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum WalletError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    WalletNotLinked = 4,
    WalletAlreadyLinked = 5,
    VerificationFailed = 6,
    InvalidSignature = 7,
    ChallengeExpired = 8,
    ChallengeNotFound = 9,
    AlreadyVerified = 10,
    SpoofedCall = 11,
    InvalidNonce = 12,
    WalletOwnershipMismatch = 13,
}

pub struct WalletEvents;

impl WalletEvents {
    pub fn wallet_linked(env: &Env, wallet: &Address, owner: &Address) {
        let topics = (symbol_short!("wallet"), symbol_short!("linked"));
        env.events().publish(
            topics,
            (wallet.clone(), owner.clone(), env.ledger().timestamp()),
        );
    }

    pub fn wallet_unlinked(env: &Env, wallet: &Address, owner: &Address) {
        let topics = (symbol_short!("wallet"), symbol_short!("unlinked"));
        env.events().publish(
            topics,
            (wallet.clone(), owner.clone(), env.ledger().timestamp()),
        );
    }

    pub fn verification_started(
        env: &Env,
        challenge_id: u64,
        wallet: &Address,
        challenger: &Address,
    ) {
        let topics = (symbol_short!("verification"), symbol_short!("started"));
        env.events().publish(
            topics,
            (
                challenge_id,
                wallet.clone(),
                challenger.clone(),
                env.ledger().timestamp(),
            ),
        );
    }

    pub fn verification_completed(env: &Env, challenge_id: u64, wallet: &Address, verified: bool) {
        let topics = (symbol_short!("verification"), symbol_short!("completed"));
        env.events().publish(
            topics,
            (
                challenge_id,
                wallet.clone(),
                verified,
                env.ledger().timestamp(),
            ),
        );
    }

    pub fn ownership_verified(env: &Env, wallet: &Address, owner: &Address, nonce: u64) {
        let topics = (symbol_short!("ownership"), symbol_short!("verified"));
        env.events().publish(
            topics,
            (
                wallet.clone(),
                owner.clone(),
                nonce,
                env.ledger().timestamp(),
            ),
        );
    }

    pub fn spoofed_call_blocked(env: &Env, caller: &Address, wallet: &Address) {
        let topics = (symbol_short!("security"), symbol_short!("spoof_blocked"));
        env.events().publish(
            topics,
            (caller.clone(), wallet.clone(), env.ledger().timestamp()),
        );
    }
}

pub fn initialize_wallet_contract(env: &Env, admin: Address) {
    if env.storage().instance().has(&DataKey::Admin) {
        panic_with_error!(env, WalletError::AlreadyInitialized);
    }

    env.storage().instance().set(&DataKey::Admin, &admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| panic_with_error!(env, WalletError::NotInitialized))
}

pub fn require_admin(env: &Env, caller: &Address) {
    caller.require_auth();
    let admin = get_admin(env);
    if admin != caller.clone() {
        panic_with_error!(env, WalletError::Unauthorized);
    }
}

pub fn link_wallet(env: &Env, caller: Address, wallet_address: Address, owner_address: Address) {
    caller.require_auth();

    // Validate that wallet_address and owner_address are different
    if wallet_address == owner_address {
        panic_with_error!(env, WalletError::InvalidSignature);
    }

    // Validate caller is either admin or the owner
    let admin = get_admin(env);
    if caller != admin && caller != owner_address {
        panic_with_error!(env, WalletError::Unauthorized);
    }

    // Check if wallet is already linked
    if env
        .storage()
        .instance()
        .has(&DataKey::LinkedWallet(wallet_address.clone()))
    {
        panic_with_error!(env, WalletError::WalletAlreadyLinked);
    }

    // Create linked wallet info
    let wallet_info = LinkedWalletInfo {
        wallet_address: wallet_address.clone(),
        owner_address: owner_address.clone(),
        verification_timestamp: 0,
        is_verified: false,
        verification_nonce: 0,
    };

    // Store the linking
    env.storage()
        .instance()
        .set(&DataKey::LinkedWallet(wallet_address.clone()), &wallet_info);
    env.storage().instance().set(
        &DataKey::WalletOwner(owner_address.clone()),
        &wallet_address,
    );

    WalletEvents::wallet_linked(env, &wallet_address, &owner_address);
}

pub fn unlink_wallet(env: &Env, caller: Address, wallet_address: Address) {
    caller.require_auth();

    let wallet_info: LinkedWalletInfo = env
        .storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, WalletError::WalletNotLinked));

    // Verify caller is the wallet owner or admin
    let admin = get_admin(env);
    if caller != wallet_info.owner_address && caller != admin {
        panic_with_error!(env, WalletError::Unauthorized);
    }

    // Remove the linking
    env.storage()
        .instance()
        .remove(&DataKey::LinkedWallet(wallet_address.clone()));
    env.storage()
        .instance()
        .remove(&DataKey::WalletOwner(wallet_info.owner_address.clone()));
    env.storage()
        .instance()
        .remove(&DataKey::VerificationStatus(wallet_address.clone()));

    WalletEvents::wallet_unlinked(env, &wallet_address, &wallet_info.owner_address);
}

pub fn verify_wallet_ownership(
    env: &Env,
    wallet_address: Address,
    signer_address: Address,
    signature: Vec<u8>,
) -> bool {
    signer_address.require_auth();

    // Get wallet info
    let wallet_info: LinkedWalletInfo = env
        .storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, WalletError::WalletNotLinked));

    // Verify the signer matches the stored wallet owner
    if wallet_info.owner_address != signer_address {
        WalletEvents::spoofed_call_blocked(env, &signer_address, &wallet_address);
        panic_with_error!(env, WalletError::WalletOwnershipMismatch);
    }

    // Generate and verify nonce
    let current_nonce = wallet_info
        .verification_nonce
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(env, WalletError::InvalidNonce));

    // Update nonce
    let updated_wallet_info = LinkedWalletInfo {
        verification_nonce: current_nonce,
        verification_timestamp: env.ledger().timestamp(),
        is_verified: true,
        ..wallet_info
    };

    env.storage().instance().set(
        &DataKey::LinkedWallet(wallet_address.clone()),
        &updated_wallet_info,
    );
    env.storage()
        .instance()
        .set(&DataKey::VerificationStatus(wallet_address.clone()), &true);

    WalletEvents::ownership_verified(env, &wallet_address, &signer_address, current_nonce);

    true
}

pub fn create_verification_challenge(
    env: &Env,
    challenger: Address,
    wallet_address: Address,
) -> u64 {
    challenger.require_auth();

    // Verify wallet exists
    let _wallet_info: LinkedWalletInfo = env
        .storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, WalletError::WalletNotLinked));

    // Generate challenge ID
    let challenge_id = env.ledger().sequence();
    let nonce = env.ledger().timestamp();
    let expires_at = env.ledger().timestamp() + 3600; // 1 hour expiry

    let challenge = VerificationChallenge {
        challenge_id,
        wallet_address: wallet_address.clone(),
        challenger_address: challenger.clone(),
        nonce,
        created_at: env.ledger().timestamp(),
        expires_at,
        is_completed: false,
    };

    env.storage().instance().set(
        &DataKey::VerificationChallenge(wallet_address.clone(), challenge_id),
        &challenge,
    );

    WalletEvents::verification_started(env, challenge_id, &wallet_address, &challenger);

    challenge_id
}

pub fn complete_verification_challenge(
    env: &Env,
    wallet_address: Address,
    challenge_id: u64,
    signature: Vec<u8>,
) -> bool {
    // Get challenge
    let mut challenge: VerificationChallenge = env
        .storage()
        .instance()
        .get(&DataKey::VerificationChallenge(
            wallet_address.clone(),
            challenge_id,
        ))
        .unwrap_or_else(|| panic_with_error!(env, WalletError::ChallengeNotFound));

    // Check if challenge is expired
    if env.ledger().timestamp() > challenge.expires_at {
        panic_with_error!(env, WalletError::ChallengeExpired);
    }

    // Check if already completed
    if challenge.is_completed {
        panic_with_error!(env, WalletError::AlreadyVerified);
    }

    // Get wallet info to verify ownership
    let wallet_info: LinkedWalletInfo = env
        .storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, WalletError::WalletNotLinked));

    // Verify signature (simplified - in real implementation would verify cryptographic signature)
    let is_valid_signature = signature.len() > 0; // Basic validation

    if !is_valid_signature {
        panic_with_error!(env, WalletError::InvalidSignature);
    }

    // Mark challenge as completed
    challenge.is_completed = true;
    env.storage().instance().set(
        &DataKey::VerificationChallenge(wallet_address.clone(), challenge_id),
        &challenge,
    );

    // Update wallet verification status
    let updated_wallet_info = LinkedWalletInfo {
        is_verified: true,
        verification_timestamp: env.ledger().timestamp(),
        ..wallet_info
    };

    env.storage().instance().set(
        &DataKey::LinkedWallet(wallet_address.clone()),
        &updated_wallet_info,
    );
    env.storage()
        .instance()
        .set(&DataKey::VerificationStatus(wallet_address.clone()), &true);

    WalletEvents::verification_completed(env, challenge_id, &wallet_address, true);

    true
}

pub fn is_wallet_verified(env: &Env, wallet_address: Address) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::VerificationStatus(wallet_address))
        .unwrap_or(false)
}

pub fn get_wallet_info(env: &Env, wallet_address: Address) -> Option<LinkedWalletInfo> {
    env.storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address))
}

pub fn require_verified_wallet(env: &Env, wallet_address: Address, caller: &Address) {
    // Check if wallet is linked
    let wallet_info: LinkedWalletInfo = env
        .storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, WalletError::WalletNotLinked));

    // Check if wallet is verified
    if !wallet_info.is_verified {
        panic_with_error!(env, WalletError::VerificationFailed);
    }

    // Verify caller is the wallet owner (prevents spoofed calls)
    if wallet_info.owner_address != caller.clone() {
        WalletEvents::spoofed_call_blocked(env, caller, &wallet_address);
        panic_with_error!(env, WalletError::SpoofedCall);
    }

    caller.require_auth();
}

pub fn get_wallet_owner(env: &Env, wallet_address: Address) -> Option<Address> {
    let wallet_info: Option<LinkedWalletInfo> = env
        .storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address));

    wallet_info.map(|info| info.owner_address)
}

pub fn validate_wallet_action(
    env: &Env,
    wallet_address: Address,
    signer: &Address,
) -> Result<(), WalletError> {
    // Check if wallet is linked
    let wallet_info: LinkedWalletInfo = env
        .storage()
        .instance()
        .get(&DataKey::LinkedWallet(wallet_address.clone()))
        .ok_or(WalletError::WalletNotLinked)?;

    // Check if wallet is verified
    if !wallet_info.is_verified {
        return Err(WalletError::VerificationFailed);
    }

    // Verify signer matches wallet owner
    if wallet_info.owner_address != signer.clone() {
        WalletEvents::spoofed_call_blocked(env, signer, &wallet_address);
        return Err(WalletError::SpoofedCall);
    }

    Ok(())
}

#[contract]
pub struct WalletContract;

#[contractimpl]
impl WalletContract {
    pub fn initialize(env: Env, admin: Address) {
        initialize_wallet_contract(&env, admin);
    }

    pub fn get_admin(env: Env) -> Address {
        get_admin(&env)
    }

    pub fn link_wallet(env: Env, caller: Address, wallet_address: Address, owner_address: Address) {
        link_wallet(&env, caller, wallet_address, owner_address);
    }

    pub fn unlink_wallet(env: Env, caller: Address, wallet_address: Address) {
        unlink_wallet(&env, caller, wallet_address);
    }

    pub fn verify_wallet_ownership(
        env: Env,
        wallet_address: Address,
        signer_address: Address,
        signature: Vec<u8>,
    ) -> bool {
        verify_wallet_ownership(&env, wallet_address, signer_address, signature)
    }

    pub fn create_verification_challenge(
        env: Env,
        challenger: Address,
        wallet_address: Address,
    ) -> u64 {
        create_verification_challenge(&env, challenger, wallet_address)
    }

    pub fn complete_verification_challenge(
        env: Env,
        wallet_address: Address,
        challenge_id: u64,
        signature: Vec<u8>,
    ) -> bool {
        complete_verification_challenge(&env, wallet_address, challenge_id, signature)
    }

    pub fn is_wallet_verified(env: Env, wallet_address: Address) -> bool {
        is_wallet_verified(&env, wallet_address)
    }

    pub fn get_wallet_info(env: Env, wallet_address: Address) -> Option<LinkedWalletInfo> {
        get_wallet_info(&env, wallet_address)
    }

    pub fn require_verified_wallet(env: Env, wallet_address: Address, caller: Address) {
        require_verified_wallet(&env, wallet_address, &caller);
    }

    pub fn get_wallet_owner(env: Env, wallet_address: Address) -> Option<Address> {
        get_wallet_owner(&env, wallet_address)
    }

    pub fn validate_wallet_action(env: Env, wallet_address: Address, signer: Address) {
        match validate_wallet_action(&env, wallet_address, &signer) {
            Ok(()) => {}
            Err(e) => panic_with_error!(&env, e),
        }
    }
}
