#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, panic_with_error, symbol_short,
    BytesN, Env, IntoVal, Map, String, Symbol, Val, Vec,
};

const CURRENT_BATCH_ID: Symbol = symbol_short!("BATCHID");
const CURRENT_STATE_ROOT: Symbol = symbol_short!("STROOT");
const BATCH_INPUTS: Symbol = symbol_short!("BATCHIN");
const REGISTERED_TEES: Symbol = symbol_short!("TEES");
const BATCH_ATTESTED: Symbol = symbol_short!("BATCHAT");

#[contract]
pub struct OnchainCommitment;

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub struct InputCommitment {
    pub encrypted_data: String,
    pub submitter: String,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidEncryptedData = 1,
    BatchNotFound = 2,
    TeeAlreadyRegistered = 3,
    TeeNotRegistered = 4,
    BatchAlreadyAttested = 5,
    Unauthorized = 6,
}

#[contractimpl]
impl OnchainCommitment {
    /// Get the current batch ID
    pub fn current_batch_id(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&CURRENT_BATCH_ID)
            .unwrap_or(0)
    }

    /// Get the current state root
    pub fn current_state_root(env: Env) -> Option<BytesN<32>> {
        env.storage().instance().get(&CURRENT_STATE_ROOT)
    }

    /// Get all input commitments for a specific batch
    pub fn batch_inputs(env: Env, batch_id: u64) -> Vec<InputCommitment> {
        let batch_storage: Map<u64, Vec<InputCommitment>> = env
            .storage()
            .instance()
            .get(&BATCH_INPUTS)
            .unwrap_or_else(|| Map::new(&env));

        batch_storage.get(batch_id).unwrap_or_else(|| Vec::new(&env))
    }

    /// Register a TEE with its public key
    /// Returns true if registration was successful, false if already registered
    pub fn register_tee(env: Env, tee_pubkey: BytesN<32>) -> bool {
        let mut tees: Map<BytesN<32>, bool> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        // Check if TEE is already registered
        if tees.get(tee_pubkey.clone()).unwrap_or(false) {
            return false;
        }

        // Register the TEE
        tees.set(tee_pubkey.clone(), true);
        env.storage().instance().set(&REGISTERED_TEES, &tees);
        env.storage().instance().extend_ttl(100, 100);

        // Emit event
        let tee_pubkey_val: Val = tee_pubkey.into_val(&env);
        env.events().publish(("TEE_REGISTERED",), (tee_pubkey_val,));

        true
    }

    /// Check if a batch has been attested
    pub fn batch_attested(env: Env, batch_id: u64) -> bool {
        let attested_batches: Map<u64, bool> = env
            .storage()
            .instance()
            .get(&BATCH_ATTESTED)
            .unwrap_or_else(|| Map::new(&env));

        attested_batches.get(batch_id).unwrap_or(false)
    }

    /// Submit encrypted input to the current batch
    pub fn submit_encrypted_input(
        env: Env,
        submitter: String,
        encrypted_data: String,
    ) -> u64 {
        // Validate encrypted data is not empty
        if encrypted_data.len() == 0 {
            panic_with_error!(&env, Error::InvalidEncryptedData);
        }

        // Get current batch ID
        let current_batch_id = Self::current_batch_id(env.clone());

        // Create input commitment
        let input_commitment = InputCommitment {
            encrypted_data: encrypted_data.clone(),
            submitter: submitter.clone(),
            timestamp: env.ledger().timestamp(),
        };

        // Get existing batch inputs
        let mut batch_storage: Map<u64, Vec<InputCommitment>> = env
            .storage()
            .instance()
            .get(&BATCH_INPUTS)
            .unwrap_or_else(|| Map::new(&env));

        let mut inputs = batch_storage
            .get(current_batch_id)
            .unwrap_or_else(|| Vec::new(&env));
        inputs.push_back(input_commitment.clone());
        batch_storage.set(current_batch_id, inputs);

        // Persist storage
        env.storage().instance().set(&BATCH_INPUTS, &batch_storage);
        env.storage().instance().extend_ttl(100, 100);

        // Emit event
        let batch_id_val: Val = current_batch_id.into_val(&env);
        let submitter_val: Val = submitter.into_val(&env);
        env.events().publish(
            ("ENCRYPTED_INPUT_SUBMITTED",),
            (batch_id_val, submitter_val),
        );

        current_batch_id
    }

    /// Create a new batch (increment batch ID)
    /// Only callable by authorized addresses (TODO: add access control)
    pub fn create_new_batch(env: Env) -> u64 {
        let current_batch_id = Self::current_batch_id(env.clone());
        let new_batch_id = current_batch_id + 1;

        env.storage()
            .instance()
            .set(&CURRENT_BATCH_ID, &new_batch_id);
        env.storage().instance().extend_ttl(100, 100);

        // Emit event
        let batch_id_val: Val = new_batch_id.into_val(&env);
        env.events().publish(("NEW_BATCH_CREATED",), (batch_id_val,));

        new_batch_id
    }

    /// Update the current state root
    /// Only callable by registered TEEs (TODO: add access control)
    pub fn update_state_root(env: Env, state_root: BytesN<32>) {
        env.storage().instance().set(&CURRENT_STATE_ROOT, &state_root);
        env.storage().instance().extend_ttl(100, 100);

        // Emit event
        let state_root_val: Val = state_root.into_val(&env);
        env.events().publish(("STATE_ROOT_UPDATED",), (state_root_val,));
    }

    /// Submit attestation for a batch
    /// Only callable by registered TEEs
    pub fn submit_attestation(
        env: Env,
        tee_pubkey: BytesN<32>,
        batch_id: u64,
        state_root: BytesN<32>,
    ) {
        // Verify TEE is registered
        let tees: Map<BytesN<32>, bool> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        if !tees.get(tee_pubkey.clone()).unwrap_or(false) {
            panic_with_error!(&env, Error::TeeNotRegistered);
        }

        // Check if batch already attested
        if Self::batch_attested(env.clone(), batch_id) {
            panic_with_error!(&env, Error::BatchAlreadyAttested);
        }

        // Mark batch as attested
        let mut attested_batches: Map<u64, bool> = env
            .storage()
            .instance()
            .get(&BATCH_ATTESTED)
            .unwrap_or_else(|| Map::new(&env));
        attested_batches.set(batch_id, true);
        env.storage().instance().set(&BATCH_ATTESTED, &attested_batches);

        // Update state root
        env.storage().instance().set(&CURRENT_STATE_ROOT, &state_root);

        env.storage().instance().extend_ttl(100, 100);

        // Emit event
        let batch_id_val: Val = batch_id.into_val(&env);
        let tee_pubkey_val: Val = tee_pubkey.into_val(&env);
        let state_root_val: Val = state_root.into_val(&env);
        env.events().publish(
            ("BATCH_ATTESTED",),
            (batch_id_val, tee_pubkey_val, state_root_val),
        );
    }

    /// Check if a TEE is registered
    pub fn is_tee_registered(env: Env, tee_pubkey: BytesN<32>) -> bool {
        let tees: Map<BytesN<32>, bool> = env
            .storage()
            .instance()
            .get(&REGISTERED_TEES)
            .unwrap_or_else(|| Map::new(&env));

        tees.get(tee_pubkey).unwrap_or(false)
    }
}

mod test;
