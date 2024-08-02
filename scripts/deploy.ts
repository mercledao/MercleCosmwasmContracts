import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Secp256k1 } from "@cosmjs/crypto";
import { fromHex } from "@cosmjs/encoding";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";

const rpc = "https://rpc.xion-testnet-1.burnt.com:443";

const defaultParams = {
  name: "TEST TOKEN",
  symbol: "TEST",
  minter: "",
  claim_issuer: "",

  is_open_mint: false,
  is_single_mint: false,
  is_tradable: false,
};

async function main() {
  const signer = await getSigner();
  const [acc] = await signer.getAccounts();

  const mwc = await deployAndInstantiateMintWithClaim(signer, {
    treasury: acc.address,
  });

  console.log("mwc", mwc);
  console.log("mwc ca", mwc.contractAddress);

  const membership = await deployAndInstantiateMembershipContract(signer, {
    ...defaultParams,
    minter: mwc.contractAddress,
    claim_issuer: "xion10vkx48vsls206gyrw3chz657zxu2cttd2nc7x0",
  });

  console.log("mem", membership);
  console.log("mem ca", membership.contractAddress);
}

main();

async function deployAndInstantiateMintWithClaim(
  deployer: DirectSecp256k1Wallet,
  params: any
) {
  const [acc] = await deployer.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(rpc, deployer, {
    gasPrice: GasPrice.fromString("0uxion"),
  });

  const upload = await client.upload(
    acc.address,
    readFileSync(
      "./contracts/MintWithClaim/artifacts/mercle_mint_with_claim.wasm"
    ),
    "auto"
  );

  console.log("Instantiating MWC contract.....");
  const instance = await client.instantiate(
    acc.address,
    upload.codeId,
    params,
    "test",
    "auto"
  );

  return instance;
}

async function deployAndInstantiateMembershipContract(
  deployer: DirectSecp256k1Wallet,
  params: any
) {
  const [acc] = await deployer.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(rpc, deployer, {
    gasPrice: GasPrice.fromString("0uxion"),
  });

  const upload = await client.upload(
    acc.address,
    readFileSync(
      "./contracts/MembershipNFT/artifacts/mercle_nft_membership.wasm"
    ),
    "auto"
  );

  console.log("Instantiating Membership contract.....");
  const instance = await client.instantiate(
    acc.address,
    upload.codeId,
    params,
    "test",
    "auto"
  );

  return instance;
}

async function getSigner() {
  const key = readFileSync(`keys/main.key`).toString().trim();
  return DirectSecp256k1Wallet.fromKey(fromHex(key), "xion");
}

async function getSecpKeypair(num: number) {
  const key = readFileSync(`keys/main.key`).toString().trim();
  return await Secp256k1.makeKeypair(fromHex(key));
}
