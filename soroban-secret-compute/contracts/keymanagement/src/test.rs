#![cfg(test)]
use crate::{TeeManagement, TeeManagementClient, TeeInfo, TeeStatus};
use soroban_sdk::{
    assert_eq, testutils::Ledger, BytesN, Env,
};
extern crate std;

#[test]
fn test_register_tee() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    // Create a test TEE public key (32 bytes)
    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);

    // Register TEE
    let result = client.register_tee(&tee_pubkey);
    assert_eq!(result, true);

    // Verify TEE is registered
    let is_registered = client.is_tee_registered(&tee_pubkey);
    assert_eq!(is_registered, true);

    // Verify TEE is valid (enabled)
    let is_valid = client.is_valid_tee(&tee_pubkey);
    assert_eq!(is_valid, true);

    // Try to register again (should return false)
    let result2 = client.register_tee(&tee_pubkey);
    assert_eq!(result2, false);
}

#[test]
fn test_disable_tee() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);

    // Register TEE
    client.register_tee(&tee_pubkey);
    assert_eq!(client.is_valid_tee(&tee_pubkey), true);

    // Disable TEE
    let result = client.disable_tee(&tee_pubkey);
    assert_eq!(result, true);

    // Verify TEE is still registered but not valid
    assert_eq!(client.is_tee_registered(&tee_pubkey), true);
    assert_eq!(client.is_valid_tee(&tee_pubkey), false);

    // Try to disable non-existent TEE
    let non_existent = BytesN::from_array(&env, &[99u8; 32]);
    let result2 = client.disable_tee(&non_existent);
    assert_eq!(result2, false);
}

#[test]
fn test_enable_tee() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);

    // Register and disable TEE
    client.register_tee(&tee_pubkey);
    client.disable_tee(&tee_pubkey);
    assert_eq!(client.is_valid_tee(&tee_pubkey), false);

    // Enable TEE
    let result = client.enable_tee(&tee_pubkey);
    assert_eq!(result, true);

    // Verify TEE is now valid
    assert_eq!(client.is_valid_tee(&tee_pubkey), true);

    // Try to enable non-existent TEE
    let non_existent = BytesN::from_array(&env, &[99u8; 32]);
    let result2 = client.enable_tee(&non_existent);
    assert_eq!(result2, false);
}

#[test]
fn test_get_tee_info() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);

    // Register TEE
    client.register_tee(&tee_pubkey);

    // Get TEE info
    let tee_info = client.get_tee_info(&tee_pubkey);
    assert_eq!(tee_info.is_some(), true);
    let info = tee_info.unwrap();
    assert_eq!(info.pubkey, tee_pubkey);
    assert_eq!(info.status, TeeStatus::Enabled);

    // Get non-existent TEE info
    let non_existent = BytesN::from_array(&env, &[99u8; 32]);
    let no_info = client.get_tee_info(&non_existent);
    assert_eq!(no_info.is_none(), true);
}

#[test]
fn test_get_all_tees() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    // Initially no TEEs
    let all_tees = client.get_all_tees();
    assert_eq!(all_tees.len(), 0);

    // Register multiple TEEs
    let tee1 = BytesN::from_array(&env, &[1u8; 32]);
    let tee2 = BytesN::from_array(&env, &[2u8; 32]);
    let tee3 = BytesN::from_array(&env, &[3u8; 32]);

    client.register_tee(&tee1);
    client.register_tee(&tee2);
    client.register_tee(&tee3);

    // Get all TEEs
    let all_tees = client.get_all_tees();
    assert_eq!(all_tees.len(), 3);

    // Disable one TEE
    client.disable_tee(&tee2);

    // All TEEs should still be in the list
    let all_tees_after_disable = client.get_all_tees();
    assert_eq!(all_tees_after_disable.len(), 3);

    // Verify disabled TEE is still in list but with disabled status
    let tee2_info = client.get_tee_info(&tee2);
    assert_eq!(tee2_info.is_some(), true);
    assert_eq!(tee2_info.unwrap().status, TeeStatus::Disabled);
}

#[test]
fn test_tee_status_transitions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);

    // Initially not registered
    assert_eq!(client.is_tee_registered(&tee_pubkey), false);
    assert_eq!(client.is_valid_tee(&tee_pubkey), false);

    // Register -> should be registered and valid
    client.register_tee(&tee_pubkey);
    assert_eq!(client.is_tee_registered(&tee_pubkey), true);
    assert_eq!(client.is_valid_tee(&tee_pubkey), true);

    // Disable -> should be registered but not valid
    client.disable_tee(&tee_pubkey);
    assert_eq!(client.is_tee_registered(&tee_pubkey), true);
    assert_eq!(client.is_valid_tee(&tee_pubkey), false);

    // Enable -> should be registered and valid again
    client.enable_tee(&tee_pubkey);
    assert_eq!(client.is_tee_registered(&tee_pubkey), true);
    assert_eq!(client.is_valid_tee(&tee_pubkey), true);
}

#[test]
fn test_multiple_tee_operations() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    // Register multiple TEEs
    let tee1 = BytesN::from_array(&env, &[1u8; 32]);
    let tee2 = BytesN::from_array(&env, &[2u8; 32]);
    let tee3 = BytesN::from_array(&env, &[3u8; 32]);

    assert_eq!(client.register_tee(&tee1), true);
    assert_eq!(client.register_tee(&tee2), true);
    assert_eq!(client.register_tee(&tee3), true);

    // All should be valid
    assert_eq!(client.is_valid_tee(&tee1), true);
    assert_eq!(client.is_valid_tee(&tee2), true);
    assert_eq!(client.is_valid_tee(&tee3), true);

    // Disable tee2
    client.disable_tee(&tee2);

    // tee1 and tee3 should still be valid, tee2 should not
    assert_eq!(client.is_valid_tee(&tee1), true);
    assert_eq!(client.is_valid_tee(&tee2), false);
    assert_eq!(client.is_valid_tee(&tee3), true);

    // Re-enable tee2
    client.enable_tee(&tee2);
    assert_eq!(client.is_valid_tee(&tee2), true);

    // Verify all are registered
    assert_eq!(client.is_tee_registered(&tee1), true);
    assert_eq!(client.is_tee_registered(&tee2), true);
    assert_eq!(client.is_tee_registered(&tee3), true);
}

#[test]
fn test_tee_info_timestamp() {
    let env = Env::default();
    env.ledger().with_mut(|l| {
        l.timestamp = 1000;
    });

    let contract_id = env.register_contract(None, TeeManagement);
    let client = TeeManagementClient::new(&env, &contract_id);

    let tee_pubkey = BytesN::from_array(&env, &[1u8; 32]);

    // Register TEE
    client.register_tee(&tee_pubkey);

    // Get TEE info and verify timestamp
    let tee_info = client.get_tee_info(&tee_pubkey).unwrap();
    assert_eq!(tee_info.registered_at, 1000);
    assert_eq!(tee_info.pubkey, tee_pubkey);
    assert_eq!(tee_info.status, TeeStatus::Enabled);
}
