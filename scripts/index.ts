import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Secp256k1, keccak256 } from "@cosmjs/crypto";
import { fromHex } from "@cosmjs/encoding";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";

const rpc = "http://localhost:26657";

async function main() {
  const signer = await getSigner(1);
  const [acc] = await signer.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(rpc, signer, {
    gasPrice: GasPrice.fromString("0.43uxion"),
  });

  const upload = await client.upload(
    acc.address,
    readFileSync(
      "./contracts/MintWithClaim/artifacts/mercle_mint_with_claim.wasm"
    ),
    "auto"
  );

  console.log("Instantiating contract.....");
  const instance = await client.instantiate(
    acc.address,
    upload.codeId,
    {
      treasury: acc.address,
    },
    "test",
    "auto"
  );

  console.log("Deployed at ", instance.contractAddress);

  const message = "hi";

  const messageString = JSON.stringify(message);
  const messageHash = keccak256(new TextEncoder().encode(messageString));

  const keypair = await getSecpKeypair(1);

  const signature = await Secp256k1.createSignature(
    messageHash,
    keypair.privkey
  );

  const res = await client.queryContractSmart(instance.contractAddress, {
    verify_sign: {
      message: messageString,
      signature: signature.toDer(),
    },
  });

  console.log("Verification result:", res);
}

main().catch(console.error);

async function getSigner(num: number) {
  const key = readFileSync(`keys/account${num}.key`).toString().trim();
  return DirectSecp256k1Wallet.fromKey(fromHex(key), "xion");
}

async function getSecpKeypair(num: number) {
  const key = readFileSync(`keys/account${num}.key`).toString().trim();
  return await Secp256k1.makeKeypair(fromHex(key));
}
