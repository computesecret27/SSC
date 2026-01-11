# Soroban Secret Compute

A decentralized system for submitting encrypted inputs on-chain and processing them through Trusted Execution Environments (TEEs) with batch-based attestation on the Soroban blockchain.

## Overview

This project implements a secure computation system where users can submit encrypted data on-chain, which is then processed in batches by registered TEE nodes. The system ensures data privacy through encryption while maintaining transparency and verifiability through on-chain commitments and attestations.

## Architecture

The system consists of four main smart contracts:

### 1. Attestation Service (`attestationservice`)

The core contract that handles encrypted input submissions, batch management, and attestation verification.

**Key Features:**
- **Batch-based Processing**: Inputs are organized into batches for efficient processing
- **Encrypted Input Storage**: Users submit encrypted data that is stored on-chain
- **State Root Management**: Tracks the current state root (bytes32) representing the system state
- **Attestation Verification**: TEEs attest to batch processing results

**Core Functions:**
- `current_batch_id() -> u64` - Get the current batch ID
- `current_state_root() -> Option<BytesN<32>>` - Get the current state root
- `batch_inputs(batch_id) -> Vec<InputCommitment>` - Get all inputs for a batch
- `batch_attested(batch_id) -> bool` - Check if a batch has been attested
- `submit_encrypted_input(submitter, encrypted_data) -> u64` - Submit encrypted input
- `create_new_batch() -> u64` - Create a new batch
- `update_state_root(state_root)` - Update the state root
- `submit_attestation(tee_pubkey, batch_id, state_root)` - Submit attestation for a batch

**Data Structures:**
```rust
pub struct InputCommitment {
    pub encrypted_data: String,
    pub submitter: String,  // Stellar address as string
    pub timestamp: u64,
}
```

### 2. TEE Management (`keymanagement`)

Manages the lifecycle of Trusted Execution Environment nodes.

**Key Features:**
- **TEE Registration**: Register TEE nodes with their public keys
- **Status Management**: Enable/disable TEEs without removing them from registry
- **Validation**: Check if TEEs are valid (registered and enabled)

**Core Functions:**
- `register_tee(tee_pubkey) -> bool` - Register a new TEE
- `disable_tee(tee_pubkey) -> bool` - Disable a TEE
- `enable_tee(tee_pubkey) -> bool` - Re-enable a disabled TEE
- `is_valid_tee(tee_pubkey) -> bool` - Check if TEE is valid (registered AND enabled)
- `is_tee_registered(tee_pubkey) -> bool` - Check if TEE is registered
- `get_tee_info(tee_pubkey) -> Option<TeeInfo>` - Get TEE information
- `get_all_tees() -> Vec<TeeInfo>` - Get all registered TEEs

**Data Structures:**
```rust
pub enum TeeStatus {
    Enabled,
    Disabled,
}

pub struct TeeInfo {
    pub pubkey: BytesN<32>,
    pub status: TeeStatus,
    pub registered_at: u64,
}
```

### 3. Staking Contract (`Staking`)

Handles token staking for users who want to participate in the system.

**Key Features:**
- **Token Staking**: Users can stake tokens to participate
- **Balance Management**: Tracks user balances and staked amounts
- **Token Operations**: Minting and transferring tokens

**Core Functions:**
- `stake(user_addr, staking_contract, amount)` - Stake tokens
- `unstake(user_addr, staking_contract, amount)` - Unstake tokens
- `mint(to, amount)` - Mint new tokens
- `transfer(from, to, amount)` - Transfer tokens

### 4. Shared Module (`shared`)

Contains shared data structures and types used across contracts.

## Workflow

### 1. TEE Registration
1. TEE nodes register themselves using `register_tee()` in the TEE Management contract
2. TEEs can be enabled/disabled as needed without removal from registry

### 2. Input Submission
1. Users encrypt their data off-chain
2. Users call `submit_encrypted_input()` with their encrypted data
3. Input is added to the current batch as an `InputCommitment`
4. Event `ENCRYPTED_INPUT_SUBMITTED` is emitted

### 3. Batch Processing
1. When ready, a new batch is created via `create_new_batch()`
2. TEEs process the encrypted inputs in the batch off-chain
3. TEEs compute a state root representing the batch results

### 4. Attestation
1. Valid TEEs call `submit_attestation()` with:
   - Their public key
   - The batch ID
   - The computed state root
2. System verifies:
   - TEE is registered and enabled
   - Batch hasn't been attested yet
3. Batch is marked as attested and state root is updated
4. Event `BATCH_ATTESTED` is emitted

## Contract Interaction Flow

```
User → [Encrypt Data] → AttestationService.submit_encrypted_input()
                                 ↓
                    [Input added to current batch]
                                 ↓
                    [Batch processing ready]
                                 ↓
                    AttestationService.create_new_batch()
                                 ↓
                    [TEEs process batch off-chain]
                                 ↓
                    AttestationService.submit_attestation()
                                 ↓
                    [Batch marked as attested]
                                 ↓
                    [State root updated]
```

## Storage Structure

### Attestation Service
- `CURRENT_BATCH_ID`: Current batch identifier (u64)
- `CURRENT_STATE_ROOT`: Current state root (BytesN<32>)
- `BATCH_INPUTS`: Map of batch_id → Vec<InputCommitment>
- `BATCH_ATTESTED`: Map of batch_id → bool

### TEE Management
- `REGISTERED_TEES`: Map of tee_pubkey → TeeInfo

## Events

### Attestation Service
- `ENCRYPTED_INPUT_SUBMITTED`: Emitted when encrypted input is submitted
- `NEW_BATCH_CREATED`: Emitted when a new batch is created
- `STATE_ROOT_UPDATED`: Emitted when state root is updated
- `BATCH_ATTESTED`: Emitted when a batch is attested by a TEE

### TEE Management
- `TEE_REGISTERED`: Emitted when a TEE is registered
- `TEE_DISABLED`: Emitted when a TEE is disabled
- `TEE_ENABLED`: Emitted when a TEE is re-enabled

## Error Handling

### Attestation Service Errors
- `InvalidEncryptedData`: Encrypted data is empty
- `BatchNotFound`: Batch ID doesn't exist
- `TeeNotRegistered`: TEE is not registered
- `BatchAlreadyAttested`: Batch has already been attested
- `Unauthorized`: Caller is not authorized

### TEE Management Errors
- `TeeAlreadyRegistered`: TEE is already registered
- `TeeNotRegistered`: TEE is not registered
- `TeeDisabled`: TEE is disabled

## Development

### Building Contracts

```bash
# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Build specific contract
cd contracts/attestationservice
cargo build --target wasm32-unknown-unknown --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for specific contract
cd contracts/attestationservice
cargo test
```

### Deployment

```bash
# Upload contract
stellar contract upload \
  --network testnet \
  --source alice \
  --wasm target/wasm32-unknown-unknown/release/attestationservice.wasm

# Deploy contract
stellar contract deploy \
  --wasm-hash <hash> \
  --source alice \
  --network testnet \
  --alias AttestationService
```

## Usage Examples

### Contract Constants

Replace these with your deployed contract IDs:
- `ATTESTATION_SERVICE_CONTRACT_ID`: `CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ`
- Replace `ssc` with your key name in the Stellar CLI

### 1. Register a TEE

**Option A: Using the helper script (recommended)**
```bash
cd TEEEngine
./register_tee.sh GARMLJUW2O4H5OLBPEF3KALP5RINJK3CZQVQJ5CFDJ75SO4APKHACJG3
```

**Option B: Direct CLI command**
```bash
# First, compute the SHA-256 hash of your TEE address (32 bytes)
TEE_ADDRESS="GARMLJUW2O4H5OLBPEF3KALP5RINJK3CZQVQJ5CFDJ75SO4APKHACJG3"
TEE_PUBKEY_HEX=$(echo -n "$TEE_ADDRESS" | shasum -a 256 | cut -d' ' -f1)

# Register the TEE
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  register_tee \
  --tee_pubkey "$TEE_PUBKEY_HEX"
```

### 2. Submit Encrypted Input

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  submit_encrypted_input \
  --submitter GARMLJUW2O4H5OLBPEF3KALP5RINJK3CZQVQJ5CFDJ75SO4APKHACJG3 \
  --encrypted_data "0x1234567890abcdef"
```

### 3. Create a New Batch

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  create_new_batch
```

### 4. Check Batch Status

```bash
# Check current batch ID
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  current_batch_id

# Check if a batch is attested
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  batch_attested \
  --batch_id 0

# Get inputs for a batch
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  batch_inputs \
  --batch_id 0

# Get current state root
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  current_state_root
```

### 5. Query Events

**Using the helper script:**
```bash
cd TEEEngine
./query_events.sh
```

**Using curl directly:**
```bash
curl -X POST "https://soroban-testnet.stellar.org" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 8675309,
    "method": "getEvents",
    "params": {
      "startLedger": 420646,
      "filters": [
        {
          "type": "contract",
          "contractIds": [
            "CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ"
          ]
        }
      ],
      "xdrFormat": "json"
    }
  }' | jq '.'
```

### 6. Run the TEE Engine

**Setup:**
1. Create a `.env` file in the `TEEEngine` directory:
```bash
cd TEEEngine
cat > .env << EOF
PUBLIC_KEY=YOUR_STELLAR_PUBLIC_KEY
SECRET_KEY=YOUR_STELLAR_SECRET_KEY
TEE_PUBKEY=YOUR_TEE_PUBKEY_OR_STELLAR_ADDRESS
TEE_MANAGEMENT_CONTRACT_ID=
EOF
```

2. Run the TEE Engine:
```bash
cd TEEEngine
cargo run
```

The TEE Engine will:
- Listen for `ENCRYPTED_INPUT_SUBMITTED` events
- Listen for `NEW_BATCH_CREATED` events
- Automatically process batches when a new batch is created
- Submit attestations for processed batches

### 7. Verify TEE Registration

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- \
  is_tee_registered \
  --tee_pubkey <32_byte_hex_string>
```

## Upcoming Enhancement
[] add nonce to prevent double submission of attestation 
[] add multi TEE nodes 
[] add custom logic in attestation 

## License