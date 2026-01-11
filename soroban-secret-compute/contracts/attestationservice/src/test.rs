#![cfg(test)]
use crate::{InputCommitment, OnchainCommitment, OnchainCommitmentClient};
use soroban_sdk::{
    log, testutils::Ledger, BytesN, Env, String,
};
extern crate std;

#[test]
fn test_current_batch_id() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Initially should be 0
    let batch_id = client.current_batch_id();
    assert_eq!(batch_id, 0);

    // Create a new batch
    let new_batch_id = client.create_new_batch();
    assert_eq!(new_batch_id, 1);

    // Verify current batch ID is updated
    let current_batch_id = client.current_batch_id();
    assert_eq!(current_batch_id, 1);
}

#[test]
fn test_register_tee() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Create a test TEE public key (32 bytes)
    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);

    // Register TEE
    let result = client.register_tee(&tee_pubkey);
    assert_eq!(result, true);

    // Verify TEE is registered
    let is_registered = client.is_tee_registered(&tee_pubkey);
    assert_eq!(is_registered, true);

    // Try to register again (should return false)
    let result2 = client.register_tee(&tee_pubkey);
    assert_eq!(result2, false);
}

#[test]
fn test_submit_encrypted_input() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Create a test submitter address
    let submitter = String::from_str(&env, "GBM3EUBXO6SBBS2JF7RJBXF4EPGWH3TJVTUKWY242RT3EJQAQ6RHXQWE");
    let encrypted_data = String::from_str(&env, "0x1234567890abcdef");

    // Submit encrypted input (should go to batch 0)
    let batch_id = client.submit_encrypted_input(&submitter, &encrypted_data);
    assert_eq!(batch_id, 0);

    // Get batch inputs
    let inputs = client.batch_inputs(&batch_id);
    assert_eq!(inputs.len(), 1);

    // Verify the input commitment
    let input = inputs.get(0).unwrap();
    assert_eq!(input.encrypted_data, encrypted_data);
    assert_eq!(input.submitter, submitter);
}

#[test]
fn test_batch_inputs() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Submit multiple inputs to batch 0
    let submitter1 = String::from_str(&env, "GBM3EUBXO6SBBS2JF7RJBXF4EPGWH3TJVTUKWY242RT3EJQAQ6RHXQWE");
    let submitter2 = String::from_str(&env, "GCNY5OXYSY4FKHOPT2SPOQZAOEIGXB5LBYW3HVU3OWEQOEHFTKQK2MQM");
    let encrypted_data1 = String::from_str(&env, "0x1111111111");
    let encrypted_data2 = String::from_str(&env, "0x2222222222");

    client.submit_encrypted_input(&submitter1, &encrypted_data1);
    client.submit_encrypted_input(&submitter2, &encrypted_data2);

    // Get all inputs for batch 0
    let inputs = client.batch_inputs(&0);
    assert_eq!(inputs.len(), 2);

    // Verify inputs
    let input1 = inputs.get(0).unwrap();
    assert_eq!(input1.encrypted_data, encrypted_data1);
    assert_eq!(input1.submitter, submitter1);

    let input2 = inputs.get(1).unwrap();
    assert_eq!(input2.encrypted_data, encrypted_data2);
    assert_eq!(input2.submitter, submitter2);

    // Test empty batch
    let empty_inputs = client.batch_inputs(&999);
    assert_eq!(empty_inputs.len(), 0);
}

#[test]
fn test_create_new_batch() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Initially batch ID is 0
    assert_eq!(client.current_batch_id(), 0);

    // Create new batch
    let batch_id_1 = client.create_new_batch();
    assert_eq!(batch_id_1, 1);
    assert_eq!(client.current_batch_id(), 1);

    // Create another batch
    let batch_id_2 = client.create_new_batch();
    assert_eq!(batch_id_2, 2);
    assert_eq!(client.current_batch_id(), 2);

    // Submit input to new batch
    let submitter = String::from_str(&env, "GBM3EUBXO6SBBS2JF7RJBXF4EPGWH3TJVTUKWY242RT3EJQAQ6RHXQWE");
    let encrypted_data = String::from_str(&env, "0xabcdef");
    let returned_batch_id = client.submit_encrypted_input(&submitter, &encrypted_data);
    assert_eq!(returned_batch_id, 2); // Should go to current batch (2)
}

#[test]
fn test_state_root() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Initially no state root
    let state_root = client.current_state_root();
    assert_eq!(state_root, None);

    // Update state root
    let new_state_root = BytesN::from_array(&env, &[42u8; 32]);
    client.update_state_root(&new_state_root);

    // Verify state root is updated
    let current_root = client.current_state_root();
    assert_eq!(current_root, Some(new_state_root));
}

#[test]
fn test_batch_attested() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Initially batch is not attested
    assert_eq!(client.batch_attested(&0), false);

    // Register a TEE
    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);
    client.register_tee(&tee_pubkey);

    // Submit some inputs to batch 0
    let submitter = String::from_str(&env, "GBM3EUBXO6SBBS2JF7RJBXF4EPGWH3TJVTUKWY242RT3EJQAQ6RHXQWE");
    let encrypted_data = String::from_str(&env, "0x123456");
    client.submit_encrypted_input(&submitter, &encrypted_data);

    // Attest the batch
    let state_root = BytesN::from_array(&env, &[99u8; 32]);
    client.submit_attestation(&tee_pubkey, &0, &state_root);

    // Verify batch is attested
    assert_eq!(client.batch_attested(&0), true);

    // Verify state root was updated
    let current_root = client.current_state_root();
    assert_eq!(current_root, Some(state_root));
}

#[test]
#[should_panic(expected = "TeeNotRegistered")]
fn test_submit_attestation_with_unregistered_tee() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Try to attest with unregistered TEE
    let unregistered_tee = BytesN::from_array(&env, &[99u8; 32]);
    let state_root = BytesN::from_array(&env, &[42u8; 32]);
    client.submit_attestation(&unregistered_tee, &0, &state_root);
}

#[test]
#[should_panic(expected = "BatchAlreadyAttested")]
fn test_submit_attestation_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // Register TEE
    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);
    client.register_tee(&tee_pubkey);

    // Attest batch 0
    let state_root1 = BytesN::from_array(&env, &[42u8; 32]);
    client.submit_attestation(&tee_pubkey, &0, &state_root1);

    // Try to attest the same batch again (should panic)
    let state_root2 = BytesN::from_array(&env, &[99u8; 32]);
    client.submit_attestation(&tee_pubkey, &0, &state_root2);
}

#[test]
#[should_panic(expected = "InvalidEncryptedData")]
fn test_submit_empty_encrypted_input() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    let submitter = String::from_str(&env, "GBM3EUBXO6SBBS2JF7RJBXF4EPGWH3TJVTUKWY242RT3EJQAQ6RHXQWE");
    let empty_data = String::from_str(&env, "");
    client.submit_encrypted_input(&submitter, &empty_data);
}

#[test]
fn test_full_workflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, OnchainCommitment);
    let client = OnchainCommitmentClient::new(&env, &contract_id);

    // 1. Register TEE
    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);
    assert_eq!(client.register_tee(&tee_pubkey), true);
    assert_eq!(client.is_tee_registered(&tee_pubkey), true);

    // 2. Submit multiple encrypted inputs to batch 0
    let submitter1 = String::from_str(&env, "GBM3EUBXO6SBBS2JF7RJBXF4EPGWH3TJVTUKWY242RT3EJQAQ6RHXQWE");
    let submitter2 = String::from_str(&env, "GCNY5OXYSY4FKHOPT2SPOQZAOEIGXB5LBYW3HVU3OWEQOEHFTKQK2MQM");
    let data1 = String::from_str(&env, "0x111111");
    let data2 = String::from_str(&env, "0x222222");

    let batch_id1 = client.submit_encrypted_input(&submitter1, &data1);
    let batch_id2 = client.submit_encrypted_input(&submitter2, &data2);
    assert_eq!(batch_id1, 0);
    assert_eq!(batch_id2, 0);

    // 3. Verify batch inputs
    let inputs = client.batch_inputs(&0);
    assert_eq!(inputs.len(), 2);

    // 4. Attest the batch
    let state_root = BytesN::from_array(&env, &[123u8; 32]);
    client.submit_attestation(&tee_pubkey, &0, &state_root);

    // 5. Verify batch is attested
    assert_eq!(client.batch_attested(&0), true);
    assert_eq!(client.current_state_root(), Some(state_root));

    // 6. Create new batch and submit to it
    let new_batch_id = client.create_new_batch();
    assert_eq!(new_batch_id, 1);
    assert_eq!(client.current_batch_id(), 1);

    let submitter3 = String::from_str(&env, "GDQERENWDSGE6TH6PIN5EJ5ABMFQQJDBHH5XKCDD5DUCKG6GLCTQIGLL");
    let data3 = String::from_str(&env, "0x333333");
    let batch_id3 = client.submit_encrypted_input(&submitter3, &data3);
    assert_eq!(batch_id3, 1);

    // 7. Verify new batch inputs
    let new_batch_inputs = client.batch_inputs(&1);
    assert_eq!(new_batch_inputs.len(), 1);
    assert_eq!(new_batch_inputs.get(0).unwrap().encrypted_data, data3);
}
