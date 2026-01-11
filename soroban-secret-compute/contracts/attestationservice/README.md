# Attestation Service Contract

A Soroban smart contract for managing encrypted input submissions, batch processing, and TEE attestations on the Stellar blockchain.

## Contract Information

**Testnet Contract ID:** `CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ`  
**WASM Hash:** `a8eaec95d32f9550cd29ef181fe5f92efd619a15192b7e288934f092b9a31bee`  
**Network:** Testnet  
**Alias:** AttestationService

- [Stellar Expert](https://stellar.expert/explorer/testnet/contract/CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ)
- [Stellar Laboratory](https://lab.stellar.org/r/testnet/contract/CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ)

## Quick Start

### Submit Encrypted Input

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- submit_encrypted_input \
  --submitter "GARMLJUW2O4H5OLBPEF3KALP5RINJK3CZQVQJ5CFDJ75SO4APKHACJG3" \
  --encrypted_data "0x1234567890abcdef"
```

### Check Current Batch ID

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- current_batch_id
```

## Core Functions

### Batch Management

- `current_batch_id()` → `u64` - Get current batch ID
- `current_state_root()` → `Option<BytesN<32>>` - Get current state root
- `batch_inputs(batch_id: u64)` → `Vec<InputCommitment>` - Get all inputs for a batch
- `batch_attested(batch_id: u64)` → `bool` - Check if batch is attested
- `create_new_batch()` → `u64` - Create a new batch

### Input Submission

- `submit_encrypted_input(submitter: String, encrypted_data: String)` → `u64` - Submit encrypted input

### Attestation

- `submit_attestation(tee_pubkey: BytesN<32>, batch_id: u64, state_root: BytesN<32>)` - Submit batch attestation

## Deployment

### Build

```bash
cd contracts/attestationservice
make build
```

WASM location: `target/wasm32v1-none/release/attestation_service.wasm`

### Upload

```bash
stellar contract upload \
  --network testnet \
  --source ssc \
  --wasm target/wasm32v1-none/release/attestation_service.wasm
```

### Deploy

```bash
stellar contract deploy \
  --wasm-hash a8eaec95d32f9550cd29ef181fe5f92efd619a15192b7e288934f092b9a31bee \
  --source ssc \
  --network testnet \
  --alias AttestationService
```

## Usage Examples

### Register TEE

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- register_tee \
  --tee_pubkey <32_BYTE_HEX_STRING>
```

### Create New Batch

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- create_new_batch
```

### Get Batch Inputs

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- batch_inputs \
  --batch_id 0
```

### Submit Attestation

```bash
stellar contract invoke \
  --id CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ \
  --source ssc \
  --network testnet \
  -- submit_attestation \
  --tee_pubkey <32_BYTE_HEX_STRING> \
  --batch_id 0 \
  --state_root <32_BYTE_HEX_STRING>
```

## Events

- `ENCRYPTED_INPUT_SUBMITTED` - Emitted when encrypted input is submitted
- `NEW_BATCH_CREATED` - Emitted when a new batch is created
- `BATCH_ATTESTED` - Emitted when a batch is attested
- `TEE_REGISTERED` - Emitted when a TEE is registered

## Testing

```bash
make test
# or
cargo test
```

Run specific test:
```bash
cargo test test_full_workflow
```
