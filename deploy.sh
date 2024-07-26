#!/bin/bash
set -eu

PATH=build:$PATH

CHAIN_ID=my-test-chain
MONIKER=devnet
HOME=$(pwd)/wasmd/node

# bash
TXFLAG="--chain-id $CHAIN_ID --gas-prices 0.25stake --gas auto --gas-adjustment 1.4 --keyring-backend test --home $HOME"
RES=$(xiond tx wasm store contracts/artifacts/cw_nameservice.wasm --from account1 $TXFLAG -y --output json)

echo $RES

# HASH=$(xiond tx wasm store $(pwd)/contracts/artifacts/contract.wasm --from account1 --keyring-backend test --home $HOME --chain-id $CHAIN_ID -y --gas 10000000000 --output json | jq -r '.txhash' )

# sleep 5

# RES=$(xiond query tx $HASH --output json)


# echo $RES

# CODE_ID=$(echo $RES | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

# echo $CODE_ID
# # Prepare the instantiation message
# INITIAL_STATE='{"count":100}'
# # Instantiate the contract
# xiond tx wasm instantiate $CODE_ID $INITIAL_STATE  --from account1 --label "test contract" --keyring-backend test --home $HOME --chain-id $CHAIN_ID -y --gas 10000000000 -y --no-admin