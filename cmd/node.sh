#!/bin/bash
set -eu

PATH=build:$PATH

MONIKER=devnet
CHAIN_ID=my-test-chain
HOME=$(pwd)/wasmd/node

if [ -d "$HOME" ]; then
  rm -rf "$HOME"
  echo "Directory $HOME and its contents have been removed."
else
  echo "Directory $HOME does not exist."
fi


xiond init $MONIKER --chain-id $CHAIN_ID --home $HOME --default-denom uxion


xiond keys add validator1 --keyring-backend test --home $HOME 
yes | xiond keys export validator1 --unsafe --unarmored-hex --keyring-backend test --home $HOME > keys/validator1.key
MY_VALIDATOR_ADDRESS=$(xiond keys show validator1 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_VALIDATOR_ADDRESS 1000000000uxion --home $HOME

xiond keys add account1 --keyring-backend test --home $HOME 
yes | xiond keys export account1 --unsafe --unarmored-hex --keyring-backend test --home $HOME > keys/account1.key
MY_ACC1_ADDRESS=$(xiond keys show account1 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_ACC1_ADDRESS 1000000000uxion --home $HOME

xiond keys add account2 --keyring-backend test --home $HOME 
yes | xiond keys export account2 --unsafe --unarmored-hex --keyring-backend test --home $HOME > keys/account2.key
MY_ACC2_ADDRESS=$(xiond keys show account2 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_ACC2_ADDRESS 1000000000uxion --home $HOME

xiond keys add account3 --keyring-backend test --home $HOME 
yes | xiond keys export account3 --unsafe --unarmored-hex --keyring-backend test --home $HOME > keys/account3.key
MY_ACC3_ADDRESS=$(xiond keys show account3 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_ACC3_ADDRESS 1000000000uxion --home $HOME

xiond keys add account4 --keyring-backend test --home $HOME 
yes | xiond keys export account4 --unsafe --unarmored-hex --keyring-backend test --home $HOME > keys/account4.key
MY_ACC4_ADDRESS=$(xiond keys show account4 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_ACC4_ADDRESS "1000000000uxion,10000000utest" --home $HOME

xiond keys add account5 --keyring-backend test --home $HOME 
yes | xiond keys export account5 --unsafe --unarmored-hex --keyring-backend test --home $HOME > keys/account5.key
MY_ACC5_ADDRESS=$(xiond keys show account5 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_ACC5_ADDRESS 1000000000uxion --home $HOME

# Create a gentx.
xiond genesis gentx validator1 10000000uxion --chain-id $CHAIN_ID --keyring-backend test --home $HOME

# Add the gentx to the genesis file.
xiond genesis collect-gentxs --home $HOME

xiond start --home $HOME --moniker $MONIKER
