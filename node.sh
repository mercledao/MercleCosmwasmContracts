#!/bin/bash
set -eu

PATH=build:$PATH

MONIKER=devnet
HOME=$(pwd)/wasmd/node

if [ -d "$HOME" ]; then
  rm -rf "$HOME"
  echo "Directory $HOME and its contents have been removed."
else
  echo "Directory $HOME does not exist."
fi


wasmd init $MONIKER --chain-id my-test-chain --home $HOME

npm run gen-key --silent > keys/validator.key
VAL_KEY=$(cat keys/validator.key)
echo $VAL_KEY | wasmd keys add validator1 --keyring-backend test --home $HOME --recover 
MY_VALIDATOR_ADDRESS=$(wasmd keys show validator1 -a --keyring-backend test --home $HOME)
wasmd add-genesis-account $MY_VALIDATOR_ADDRESS 10000000stake --home $HOME

npm run gen-key --silent > keys/account1.key
ACC1_KEY=$(cat keys/account1.key)
echo $ACC1_KEY | wasmd keys add account1  --keyring-backend test --home $HOME --recover 
MY_ACC1_ADDRESS=$(wasmd keys show account1 -a --keyring-backend test --home $HOME)
wasmd add-genesis-account $MY_ACC1_ADDRESS 1000000000stake --home $HOME

npm run gen-key --silent > keys/account2.key
ACC2_KEY=$(cat keys/account2.key)
echo $ACC2_KEY | wasmd keys add account2  --keyring-backend test --home $HOME --recover 
MY_ACC2_ADDRESS=$(wasmd keys show account2 -a --keyring-backend test --home $HOME)
wasmd add-genesis-account $MY_ACC2_ADDRESS 1000000000stake --home $HOME

npm run gen-key --silent > keys/account3.key
ACC3_KEY=$(cat keys/account3.key)
echo $ACC3_KEY | wasmd keys add account3  --keyring-backend test --home $HOME --recover 
MY_ACC3_ADDRESS=$(wasmd keys show account3 -a --keyring-backend test --home $HOME)
wasmd add-genesis-account $MY_ACC3_ADDRESS 1000000000stake --home $HOME

npm run gen-key --silent > keys/account4.key
ACC4_KEY=$(cat keys/account4.key)
echo $ACC4_KEY | wasmd keys add account4  --keyring-backend test --home $HOME --recover 
MY_ACC4_ADDRESS=$(wasmd keys show account4 -a --keyring-backend test --home $HOME)
wasmd add-genesis-account $MY_ACC4_ADDRESS 1000000000stake --home $HOME

npm run gen-key --silent > keys/account5.key
ACC5_KEY=$(cat keys/account5.key)
echo $ACC5_KEY | wasmd keys add account5  --keyring-backend test --home $HOME --recover 
MY_ACC5_ADDRESS=$(wasmd keys show account5 -a --keyring-backend test --home $HOME)
wasmd add-genesis-account $MY_ACC5_ADDRESS 1000000000stake --home $HOME

# Create a gentx.
wasmd gentx validator1 10000000stake --chain-id my-test-chain --keyring-backend test --home $HOME

# Add the gentx to the genesis file.
wasmd collect-gentxs --home $HOME

wasmd start --home $HOME --moniker $MONIKER
