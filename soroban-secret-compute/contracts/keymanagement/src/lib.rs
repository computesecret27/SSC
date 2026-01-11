#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, BytesN, Env, IntoVal, Map,
    Symbol, Val, Vec,
};

const REGISTERED_TEES: Symbol = symbol_short!("TEES");

#[contract]
pub struct TeeManagement;

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub enum TeeStatus {
    Enabled,
    Disabled,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub struct TeeInfo {
    pub pubkey: BytesN<32>,
    pub status: TeeStatus,
    pub registered_at: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    TeeAlreadyRegistered = 1,
    TeeNotRegistered = 2,
    TeeDisabled = 3,
}

#[contractimpl]
impl TeeManagement {
    /// Register a TEE with its public key
    /// Returns true if registration was successful, false if already registered
    pub fn register_tee(env: Env, tee_pubkey: BytesN<32>) -> bool {
        let mut tees: Map<BytesN<32>, TeeInfo> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        // Check if TEE is already registered
        if tees.contains_key(tee_pubkey.clone()) {
            return false;
        }

        // Register the TEE with enabled status
        let tee_info = TeeInfo {
            pubkey: tee_pubkey.clone(),
            status: TeeStatus::Enabled,
            registered_at: env.ledger().timestamp(),
        };

        tees.set(tee_pubkey.clone(), tee_info);
        env.storage().instance().set(&REGISTERED_TEES, &tees);
        env.storage().instance().extend_ttl(100, 100);

        // Emit event
        let tee_pubkey_val: Val = tee_pubkey.into_val(&env);
        env.events().publish(("TEE_REGISTERED",), (tee_pubkey_val,));

        true
    }

    /// Disable a TEE (marks it as disabled but keeps it in the registry)
    pub fn disable_tee(env: Env, tee_pubkey: BytesN<32>) -> bool {
        let mut tees: Map<BytesN<32>, TeeInfo> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        // Check if TEE exists
        if let Some(mut tee_info) = tees.get(tee_pubkey.clone()) {
            // Update status to disabled
            tee_info.status = TeeStatus::Disabled;
            tees.set(tee_pubkey.clone(), tee_info);
            env.storage().instance().set(&REGISTERED_TEES, &tees);
            env.storage().instance().extend_ttl(100, 100);

            // Emit event
            let tee_pubkey_val: Val = tee_pubkey.into_val(&env);
            env.events().publish(("TEE_DISABLED",), (tee_pubkey_val,));

            true
        } else {
            false
        }
    }

    /// Enable a previously disabled TEE
    pub fn enable_tee(env: Env, tee_pubkey: BytesN<32>) -> bool {
        let mut tees: Map<BytesN<32>, TeeInfo> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        // Check if TEE exists
        if let Some(mut tee_info) = tees.get(tee_pubkey.clone()) {
            // Update status to enabled
            tee_info.status = TeeStatus::Enabled;
            tees.set(tee_pubkey.clone(), tee_info);
            env.storage().instance().set(&REGISTERED_TEES, &tees);
            env.storage().instance().extend_ttl(100, 100);

            // Emit event
            let tee_pubkey_val: Val = tee_pubkey.into_val(&env);
            env.events().publish(("TEE_ENABLED",), (tee_pubkey_val,));

            true
        } else {
            false
        }
    }

    /// Check if a TEE is valid (registered AND enabled)
    pub fn is_valid_tee(env: Env, tee_pubkey: BytesN<32>) -> bool {
        let tees: Map<BytesN<32>, TeeInfo> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        if let Some(tee_info) = tees.get(tee_pubkey) {
            tee_info.status == TeeStatus::Enabled
        } else {
            false
        }
    }

    /// Check if a TEE is registered (regardless of status)
    pub fn is_tee_registered(env: Env, tee_pubkey: BytesN<32>) -> bool {
        let tees: Map<BytesN<32>, TeeInfo> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        tees.contains_key(tee_pubkey)
    }

    /// Get TEE information
    pub fn get_tee_info(env: Env, tee_pubkey: BytesN<32>) -> Option<TeeInfo> {
        let tees: Map<BytesN<32>, TeeInfo> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        tees.get(tee_pubkey)
    }

    /// Get all registered TEEs
    pub fn get_all_tees(env: Env) -> Vec<TeeInfo> {
        let tees: Map<BytesN<32>, TeeInfo> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        let mut all_tees = Vec::<TeeInfo>::new(&env);
        for (_, tee_info) in tees.iter() {
            all_tees.push_back(tee_info);
        }

        all_tees
    }
}

mod test;
