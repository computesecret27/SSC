use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use reqwest::blocking::Client;
use soroban_client::error::Error as SorobanError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::rc::Rc;
use std::cell::RefCell;
use soroban_client::*;
use soroban_client::account::Account;
use soroban_client::contract::Contracts;
use soroban_client::contract::ContractBehavior;
use soroban_client::network::{Networks, NetworkPassphrase};
use soroban_client::xdr::{ScVal, BytesM};
use soroban_client::keypair::Keypair;
use soroban_client::keypair::KeypairBehavior;
use soroban_client::transaction::{AccountBehavior, Transaction, TransactionBehavior};
use soroban_client::transaction::TransactionBuilderBehavior;
use soroban_client::transaction_builder::TransactionBuilder;
use tokio::time::{sleep, Duration};
use tracing::{info, error};
use tracing_subscriber;

#[derive(Serialize, Deserialize, Debug)]
struct RpcRequest {
    jsonrpc: String, 
    id: u128,
    method: String,
    params: RpcParams,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcParams {
    startLedger: u64,
    filters: Vec<Filter>,
    xdrFormat: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Filter {
    r#type: String,
    contractIds: Vec<String>,
    topics: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct EncryptedInputSubmittedEvent {
    batch_id: u64,
    submitter: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NewBatchCreatedEvent {
    batch_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventTopics {
    ENCRYPTED_INPUT_SUBMITTED,
    NEW_BATCH_CREATED,
}

// Constants
const ATTESTATION_SERVICE_CONTRACT_ID: &str = "CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ";
const RPC_URL: &str = "https://soroban-testnet.stellar.org";
const FEE: u32 = 100;

struct Config {
    public_key: String,
    secret_key: String,
    tee_pubkey: String,
    tee_management_contract_id: String,
}

impl Config {
    fn from_env() -> Self {
        dotenv::dotenv().ok(); // Load .env file, ignore if it doesn't exist
        
        Self {
            public_key: env::var("PUBLIC_KEY")
                .expect("PUBLIC_KEY must be set in .env file")
                .trim()
                .to_string(),
            secret_key: env::var("SECRET_KEY")
                .expect("SECRET_KEY must be set in .env file")
                .trim()
                .to_string(),
            tee_pubkey: env::var("TEE_PUBKEY")
                .expect("TEE_PUBKEY must be set in .env file")
                .trim()
                .to_string(),
            tee_management_contract_id: env::var("TEE_MANAGEMENT_CONTRACT_ID")
                .unwrap_or_default()
                .trim()
                .to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load configuration from .env file
    let config = Config::from_env();

    info!("🚀 TEE Engine starting...");
    info!("Listening for encrypted inputs and processing batches");

    let mut processed_batches: HashSet<u64> = HashSet::new();
    let mut batch_inputs: HashMap<u64, Vec<EncryptedInputSubmittedEvent>> = HashMap::new();

    loop {
        info!("Running batch processing cycle...");
        process_batches(&mut processed_batches, &mut batch_inputs, &config).await;
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

fn listen_for_events_rpc(
    contract_id: &str,
    read_event: &EventTopics,
) -> (
    Vec<EncryptedInputSubmittedEvent>,
    Vec<NewBatchCreatedEvent>,
) {
    let client = Client::new();
    let url = RPC_URL;
    // Start from ledger 0 or use a reasonable default
    // TODO: Fetch actual latest ledger from RPC to look back properly
    let _latest_ledger = get_latest_ledger();
    let start_ledger = 420646; // Look back 100 ledgers, but don't underflow

    let request = create_rpc_request(contract_id, start_ledger);
    let response_body = send_rpc_request(&client, url, &request);
    let processed_event = process_events(response_body, read_event);

    processed_event
}

fn get_latest_ledger() -> u64 {
    // TODO: Implement actual ledger fetching from RPC
    // For now, return a reasonable default to prevent underflow
    // You can fetch this from: https://soroban-testnet.stellar.org/getLatestLedger
    1000 // Default to a reasonable ledger number
}

fn create_rpc_request(contract_id: &str, start_ledger: u64) -> RpcRequest {
    RpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 8675309,
        method: "getEvents".to_string(),
        params: RpcParams {
            startLedger: start_ledger,
            filters: vec![Filter {
                r#type: "contract".to_string(),
                contractIds: vec![contract_id.to_string()],
                topics: vec![], // No topics filter - we'll filter client-side by topicJson
            }],
            xdrFormat: "json".to_string(),
        },
    }
}

fn send_rpc_request(client: &Client, url: &str, request: &RpcRequest) -> String {
    println!("📤 Sending RPC request to: {}", url);
    println!("📤 Request: {}", serde_json::to_string_pretty(request).unwrap_or_default());
    
    let response_body = client
        .post(url)
        .json(request)
        .send()
        .expect("Failed to send request")
        .text()
        .expect("Failed to read response body");
    
    println!("📥 Response body: {}", response_body);
    response_body
}

fn process_events(
    response_body: String,
    read_event: &EventTopics,
) -> (Vec<EncryptedInputSubmittedEvent>, Vec<NewBatchCreatedEvent>) {
    let response_json: Value =
        serde_json::from_str(&response_body).expect("Failed to parse response body");
    let mut encrypted_input_events = Vec::new();
    let mut new_batch_events = Vec::new();

    if let Some(events) = response_json["result"]["events"].as_array() {  
        for event in events {
            // Filter by topicJson client-side (like the curl script)
            let topic_string = event["topicJson"]
                .as_array()
                .and_then(|topics| topics.get(0))
                .and_then(|topic| topic.get("string"))
                .and_then(Value::as_str);
            
            // Only process events that match the expected topic
            let should_process = match read_event {
                EventTopics::ENCRYPTED_INPUT_SUBMITTED => {
                    topic_string == Some("ENCRYPTED_INPUT_SUBMITTED")
                }
                EventTopics::NEW_BATCH_CREATED => {
                    topic_string == Some("NEW_BATCH_CREATED")
                }
            };
            
            if should_process {
                if let Some(vec_data) = event["valueJson"].get("vec").and_then(Value::as_array) {
                    match read_event {
                        EventTopics::ENCRYPTED_INPUT_SUBMITTED => {
                            if let Some(parsed_event) = parse_encrypted_input_event(&vec_data) {
                                encrypted_input_events.push(parsed_event);
                            }
                        }
                        EventTopics::NEW_BATCH_CREATED => {
                            if let Some(parsed_event) = parse_new_batch_event(&vec_data) {
                                new_batch_events.push(parsed_event);
                            }
                        }
                    }
                }
            }
        }
    }

    (encrypted_input_events, new_batch_events)
}

fn parse_encrypted_input_event(event_arr: &[Value]) -> Option<EncryptedInputSubmittedEvent> {
    if event_arr.len() >= 2 {
        // Handle both string and numeric u64 values
        let batch_id = event_arr[0]
            .get("u64")
            .and_then(|v| {
                // Try as u64 first
                v.as_u64().or_else(|| {
                    // If that fails, try as string and parse
                    v.as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                })
            })
            .unwrap_or(0);
        
        Some(EncryptedInputSubmittedEvent {
            batch_id,
            submitter: event_arr[1]
                .get("string")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        })
    } else {
        None
    }
}

fn parse_new_batch_event(event_arr: &[Value]) -> Option<NewBatchCreatedEvent> {
    if event_arr.len() >= 1 {
        // Handle both string and numeric u64 values
        let batch_id = event_arr[0]
            .get("u64")
            .and_then(|v| {
                // Try as u64 first
                v.as_u64().or_else(|| {
                    // If that fails, try as string and parse
                    v.as_str()
                        .and_then(|s| s.parse::<u64>().ok())
                })
            })
            .unwrap_or(0);
        
        Some(NewBatchCreatedEvent {
            batch_id,
        })
    } else {
        None
    }
}

async fn process_batches(
    processed_batches: &mut HashSet<u64>,
    batch_inputs: &mut HashMap<u64, Vec<EncryptedInputSubmittedEvent>>,
    config: &Config,
) {
    info!("Starting batch processing cycle...");

    let server = match Server::new(RPC_URL, Options::default()) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create server: {:?}", e);
            return;
        }
    };

    // Listen for encrypted input submissions (no topic filter - filtered client-side)
    let encrypted_input_events = tokio::task::spawn_blocking(|| {
        listen_for_events_rpc(
            ATTESTATION_SERVICE_CONTRACT_ID,
            &EventTopics::ENCRYPTED_INPUT_SUBMITTED,
        )
    })
    .await
    .expect("Failed to fetch encrypted input events");

    // Listen for new batch creation (no topic filter - filtered client-side)
    let new_batch_events = tokio::task::spawn_blocking(|| {
        listen_for_events_rpc(
            ATTESTATION_SERVICE_CONTRACT_ID,
            &EventTopics::NEW_BATCH_CREATED,
        )
    })
    .await
    .expect("Failed to fetch new batch events");

    info!(
        "📥 Found {} encrypted input submissions",
        encrypted_input_events.0.len()
    );
    info!("📥 Found {} new batches", new_batch_events.1.len());

    // Accumulate inputs by batch ID
    for event in encrypted_input_events.0 {
        batch_inputs
            .entry(event.batch_id)
            .or_insert_with(Vec::new)
            .push(event);
    }

    // Process batches that are ready for attestation
    for new_batch_event in new_batch_events.1 {
        let batch_id = new_batch_event.batch_id;
        info!("🔄 Processing new batch event: batch {} created, will process previous batch {}", batch_id, batch_id.saturating_sub(1));

        // Get inputs for the previous batch (batch_id - 1)
        let previous_batch_id = batch_id.saturating_sub(1);
        
        // Skip if the previous batch is already processed
        if processed_batches.contains(&previous_batch_id) {
            info!("⏭️  Previous batch {} (triggered by new batch {}) already processed, skipping", previous_batch_id, batch_id);
            continue;
        }

        // Check if we have inputs for this batch
        if let Some(inputs) = batch_inputs.get(&previous_batch_id) {
            info!(
                "📦 Processing batch {} with {} inputs",
                previous_batch_id,
                inputs.len()
            );

            // Check if batch is already attested
            if is_batch_attested(&server, previous_batch_id).await {
                info!("Batch {} already attested", previous_batch_id);
                processed_batches.insert(previous_batch_id);
                continue;
            }

            // Process the batch (decrypt and compute in TEE)
            match process_batch_inputs(&server, previous_batch_id, inputs).await {
                Ok(state_root) => {
                    // Submit attestation
                    if let Ok(_) = submit_attestation(
                        &server,
                        previous_batch_id,
                        state_root,
                        processed_batches,
                        config,
                    )
                    .await
                    {
                        processed_batches.insert(previous_batch_id);
                        info!("Batch {} attested successfully", previous_batch_id);
                    }
                }
                Err(e) => {
                    error!("Failed to process batch {}: {:?}", previous_batch_id, e);
                }
            }
        } else {
            // No inputs found for this batch - might be empty batch or inputs not yet received
            info!("⚠️  No inputs found for batch {} (triggered by new batch {}). Batch might be empty or inputs not yet received.", previous_batch_id, batch_id);
            // Still mark as processed to avoid retrying empty batches
            processed_batches.insert(previous_batch_id);
        }
    }
}

async fn is_batch_attested(_server: &Server, _batch_id: u64) -> bool {
    // Note: We rely on error handling in submit_attestation to detect if batch is already attested
    // The prepare_transaction will fail with SimulationFailed if the batch is already attested
    // This is more efficient than making a separate contract call
    false
}

async fn process_batch_inputs(
    _server: &Server,
    batch_id: u64,
    inputs: &[EncryptedInputSubmittedEvent],
) -> Result<[u8; 32], String> {
    info!("Processing batch {} with {} encrypted inputs", batch_id, inputs.len());

    // TODO: In a real TEE implementation:
    // 1. Fetch encrypted inputs from the contract using batch_inputs(batch_id)
    // 2. Decrypt inputs inside the TEE (using TEE-specific decryption keys)
    // 3. Process/compute on the decrypted data
    // 4. Compute state root from results using cryptographic hash (e.g., SHA-256)
    // 5. Return the state root

    // For now, generate a mock state root based on batch ID and input count
    // In production, this would be computed from actual processed data
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    batch_id.hash(&mut hasher);
    inputs.len().hash(&mut hasher);
    let hash = hasher.finish();

    let mut state_root = [0u8; 32];
    state_root[0..8].copy_from_slice(&batch_id.to_le_bytes());
    state_root[8..16].copy_from_slice(&(inputs.len() as u64).to_le_bytes());
    state_root[16..24].copy_from_slice(&hash.to_le_bytes());

    info!("Computed state root for batch {}", batch_id);
    Ok(state_root)
}

async fn submit_attestation(
    server: &Server, 
    batch_id: u64,
    state_root: [u8; 32],
    processed_batches: &mut HashSet<u64>,
    config: &Config,
) -> Result<(), SorobanError> {
    info!(
        "🧾 Submitting attestation for batch {} (seq: checking...)",
        batch_id
    );

    let account = server.get_account(&config.public_key).await.unwrap();
    let seq_num: u64 = account.sequence_number().parse::<u64>().unwrap();
 
    let source_account = Rc::new(RefCell::new(
        Account::new(&config.public_key, &seq_num.to_string()).unwrap(),
    ));

    // Convert state root to ScVal (BytesN<32>)
    let state_root_bytes: BytesM<{ u32::MAX }> = BytesM::try_from(state_root.to_vec().as_slice())
        .expect("Failed to convert state root to BytesM");
    let state_root_scval = ScVal::Bytes(state_root_bytes.into());

    // Convert TEE pubkey to ScVal (32 bytes)
    // TEE_PUBKEY can be either:
    // 1. A 64-character hex string (32 bytes) - will be decoded directly
    // 2. A Stellar address string - will be hashed to 32 bytes
    use sha2::{Sha256, Digest};
    let tee_pubkey_bytes: [u8; 32] = if config.tee_pubkey.len() == 64 {
        // Try hex decoding first (for hex strings)
        hex::decode(&config.tee_pubkey)
            .ok()
            .and_then(|v| v.try_into().ok())
            .unwrap_or_else(|| {
                // If hex decode fails, hash the string to get 32 bytes
                let mut hasher = Sha256::new();
                hasher.update(config.tee_pubkey.as_bytes());
                hasher.finalize().into()
            })
    } else {
        // Hash the string to get 32 bytes
        let mut hasher = Sha256::new();
        hasher.update(config.tee_pubkey.as_bytes());
        hasher.finalize().into()
    };
    let tee_pubkey_bytes_m: BytesM<{ u32::MAX }> = BytesM::try_from(tee_pubkey_bytes.as_slice())
        .expect("Failed to convert TEE pubkey to BytesM");
    let tee_pubkey_scval = ScVal::Bytes(tee_pubkey_bytes_m.into());

    let transaction = build_attestation_transaction(
        source_account,
        batch_id,
        tee_pubkey_scval,
        state_root_scval,
    );

    let mut prepared_tx = match server.prepare_transaction(transaction).await {
        Ok(tx) => tx,
        Err(e) => {
            let error_str = format!("{:?}", e);
            error!("Failed to prepare transaction for batch {}: {:?}", batch_id, e);
            
            // Check if it's because batch is already attested
            // The contract will reject with an error if batch is already attested
            if error_str.contains("BatchAlreadyAttested") || 
               error_str.contains("already attested") ||
               error_str.contains("SimulationFailed") {
                info!("⚠️  Batch {} is already attested or transaction simulation failed, marking as processed", batch_id);
                processed_batches.insert(batch_id);
                return Ok(()); // Return success since batch is already handled
            }
            return Err(SorobanError::JsonError(format!("Transaction preparation failed: {:?}", e)));
        }
    };
    
    // Remove any quotes that might be in the .env file and trim whitespace
    let secret_key_clean = config.secret_key.trim_matches('"').trim_matches('\'').trim();
    
    // Debug: log secret key info (for debugging, don't log full key)
    info!("Secret key length: {}, starts with: {}, ends with: {}", 
        secret_key_clean.len(), 
        secret_key_clean.chars().next().unwrap_or('?'),
        secret_key_clean.chars().last().unwrap_or('?'));
    
    // Validate secret key format BEFORE trying to create keypair
    if !secret_key_clean.starts_with('S') {
        error!("❌ Secret key must start with 'S', got: {}", secret_key_clean.chars().next().unwrap_or('?'));
        return Err(SorobanError::JsonError(
            format!("Secret key must start with 'S'")
        ));
    }
    
    if secret_key_clean.len() != 56 {
        error!("❌ Secret key must be exactly 56 characters, but got: {} characters", secret_key_clean.len());
        error!("   Your secret key appears to be missing {} character(s). Please check your .env file.", 56 - secret_key_clean.len());
        return Err(SorobanError::JsonError(
            format!("Secret key must be exactly 56 characters, got: {} characters", secret_key_clean.len())
        ));
    }
    
    // Try to create keypair - use try_from or from_secret
    let secret_key = match Keypair::from_secret(secret_key_clean) {
        Ok(kp) => {
            // Verify the keypair matches the public key
            let derived_public = kp.public_key();
            if derived_public != config.public_key.trim() {
                error!("Secret key does not match public key! Derived: {}, Expected: {}", derived_public, config.public_key);
                return Err(SorobanError::JsonError(format!("Secret key does not match public key")));
            }
            info!("✅ Keypair verified - secret key matches public key");
            kp
        },
        Err(e) => {
            error!("Failed to create keypair from secret key. Length: {}, First char: {}. Error: {:?}", 
                secret_key_clean.len(), 
                secret_key_clean.chars().next().unwrap_or('?'),
                e);
            return Err(SorobanError::JsonError(format!("Invalid secret key format: {:?}", e)));
        }
    };
    prepared_tx.sign(&[secret_key]);

    match server.send_transaction(prepared_tx).await {
        Ok(transaction_result) => {
            info!(
                "Attestation submitted for batch {} — latest ledger: {:?}",
                batch_id, transaction_result
            );
            processed_batches.insert(batch_id);
            sleep(Duration::from_secs(5)).await;
            Ok(())
        }
        Err(err) => {
            error!("Failed to submit attestation: {:?}", err);
            Err(SorobanError::JsonError(err.to_string()))
        }
    }
}

fn build_attestation_transaction(
    source_account: Rc<RefCell<Account>>,
    batch_id: u64,
    tee_pubkey: ScVal,
    state_root: ScVal,
) -> Transaction {
    let attestation_contract = Contracts::new(ATTESTATION_SERVICE_CONTRACT_ID).unwrap();

    TransactionBuilder::new(source_account, Networks::testnet(), None)
        .fee(FEE)
        .add_operation(attestation_contract.call(
            "submit_attestation",
            Some(vec![
                tee_pubkey,
                ScVal::U64(batch_id),
                state_root,
            ]),
        ))
        .build()
}
