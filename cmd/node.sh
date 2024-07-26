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

npm run gen-key --silent > keys/validator.key
VAL_KEY=$(cat keys/validator.key)
echo $VAL_KEY | xiond keys add validator1 --keyring-backend test --home $HOME --recover 
MY_VALIDATOR_ADDRESS=$(xiond keys show validator1 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_VALIDATOR_ADDRESS 10000000uxion --home $HOME

npm run gen-key --silent > keys/account1.key
ACC1_KEY=$(cat keys/account1.key)
echo $ACC1_KEY | xiond keys add account1  --keyring-backend test --home $HOME --recover 
MY_ACC1_ADDRESS=$(xiond keys show account1 -a --keyring-backend test --home $HOME)
xiond genesis add-genesis-account $MY_ACC1_ADDRESS 1000000000uxion --home $HOME

# npm run gen-key --silent > keys/account2.key
# ACC2_KEY=$(cat keys/account2.key)
# echo $ACC2_KEY | xiond keys add account2  --keyring-backend test --home $HOME --recover 
# MY_ACC2_ADDRESS=$(xiond keys show account2 -a --keyring-backend test --home $HOME)
# xiond genesis add-genesis-account $MY_ACC2_ADDRESS 1000000000stake --home $HOME

# npm run gen-key --silent > keys/account3.key
# ACC3_KEY=$(cat keys/account3.key)
# echo $ACC3_KEY | xiond keys add account3  --keyring-backend test --home $HOME --recover 
# MY_ACC3_ADDRESS=$(xiond keys show account3 -a --keyring-backend test --home $HOME)
# xiond genesis add-genesis-account $MY_ACC3_ADDRESS 1000000000stake --home $HOME

# npm run gen-key --silent > keys/account4.key
# ACC4_KEY=$(cat keys/account4.key)
# echo $ACC4_KEY | xiond keys add account4  --keyring-backend test --home $HOME --recover 
# MY_ACC4_ADDRESS=$(xiond keys show account4 -a --keyring-backend test --home $HOME)
# xiond genesis add-genesis-account $MY_ACC4_ADDRESS 1000000000stake --home $HOME

# npm run gen-key --silent > keys/account5.key
# ACC5_KEY=$(cat keys/account5.key)
# echo $ACC5_KEY | xiond keys add account5  --keyring-backend test --home $HOME --recover 
# MY_ACC5_ADDRESS=$(xiond keys show account5 -a --keyring-backend test --home $HOME)
# xiond genesis add-genesis-account $MY_ACC5_ADDRESS 1000000000stake --home $HOME

# Create a gentx.
xiond genesis gentx validator1 10000000uxion --chain-id $CHAIN_ID --keyring-backend test --home $HOME

# Add the gentx to the genesis file.
xiond genesis collect-gentxs --home $HOME

xiond start --home $HOME --moniker $MONIKER
