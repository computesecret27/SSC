#!/bin/bash

# Script to register a TEE with the Attestation Service contract
# This script hashes the Stellar address to 32 bytes (matching TEE Engine behavior)

CONTRACT_ID="CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ"
TEE_ADDRESS="${1:-GARMLJUW2O4H5OLBPEF3KALP5RINJK3CZQVQJ5CFDJ75SO4APKHACJG3}"

echo "🔑 Registering TEE with address: $TEE_ADDRESS"
echo "📝 Computing SHA-256 hash to get 32-byte pubkey..."

# Hash the address using SHA-256 (matching TEE Engine behavior)
# Using Python to compute the hash
TEE_PUBKEY_HEX=$(python3 -c "
import hashlib
import sys
address = sys.argv[1]
hash_bytes = hashlib.sha256(address.encode()).digest()
print(hash_bytes.hex())
" "$TEE_ADDRESS")

echo "✅ TEE pubkey (32 bytes, hex): $TEE_PUBKEY_HEX"
echo ""
echo "📤 Registering TEE with contract..."

# Register using stellar CLI
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source ssc \
  --network testnet \
  -- \
  register_tee \
  --tee_pubkey "$TEE_PUBKEY_HEX"

echo ""
echo "✅ Registration complete!"
echo ""
echo "To verify, check if TEE is registered:"
echo "  stellar contract invoke --id $CONTRACT_ID --source ssc --network testnet -- is_tee_registered --tee_pubkey $TEE_PUBKEY_HEX"
