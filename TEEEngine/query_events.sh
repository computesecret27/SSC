#!/bin/bash

# Query Soroban events using RPC
# This script replicates the getEvents RPC call from TEEEngine

RPC_URL="https://soroban-testnet.stellar.org"
CONTRACT_ID="CD6OMWI5REMXRH4LPWHNU6CZJ6LDZBUC7V4TAY532A6PTRQLK5YRCXCZ"
START_LEDGER=420646

echo "📤 Query 1: All events for contract (no topic filter)..."
curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 8675309,
    "method": "getEvents",
    "params": {
      "startLedger": '$START_LEDGER',
      "filters": [
        {
          "type": "contract",
          "contractIds": [
            "'$CONTRACT_ID'"
          ]
        }
      ],
      "xdrFormat": "json"
    }
  }' | jq '.'

echo ""
echo "📤 Query 2: ENCRYPTED_INPUT_SUBMITTED events (filtered client-side)..."
curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 8675309,
    "method": "getEvents",
    "params": {
      "startLedger": '$START_LEDGER',
      "filters": [
        {
          "type": "contract",
          "contractIds": [
            "'$CONTRACT_ID'"
          ]
        }
      ],
      "xdrFormat": "json"
    }
  }' | jq '.result.events[] | select(.topicJson[0].string == "ENCRYPTED_INPUT_SUBMITTED")'

echo ""
echo "📤 Query 3: NEW_BATCH_CREATED events (filtered client-side)..."
curl -s -X POST "$RPC_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 8675309,
    "method": "getEvents",
    "params": {
      "startLedger": '$START_LEDGER',
      "filters": [
        {
          "type": "contract",
          "contractIds": [
            "'$CONTRACT_ID'"
          ]
        }
      ],
      "xdrFormat": "json"
    }
  }' | jq '.result.events[] | select(.topicJson[0].string == "NEW_BATCH_CREATED")'
