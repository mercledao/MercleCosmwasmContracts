#!/bin/bash
set -eu

PATH=build:$PATH

echo "Compiling MembershipNFT Contracts...."
cd contracts/MembershipNFT && docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0

cd ../..

echo "Compiling MintWithClaim Contracts...."
cd contracts/MintWithClaim && docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0