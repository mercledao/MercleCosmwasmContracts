import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Secp256k1 } from "@cosmjs/crypto";
import { fromHex } from "@cosmjs/encoding";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";

const rpc = "https://rpc.xion-testnet-1.burnt.com:443";

async function main() {
  const signer = await getSigner();
  const [acc] = await signer.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(rpc, signer, {
    gasPrice: GasPrice.fromString("0uxion"),
  });

  const res = await client.execute(
    acc.address,
    "xion1t29u0nvdadez08mkh7t6c3a02znjmkej9pu24halg2mlnl9jt9ts5p2h9m",
    {
      set_is_single_mint: {
        value: false,
      },
    },
    "auto"
  );

  console.log(res);
}

main();

async function getSigner() {
  const key = readFileSync(`keys/main.key`).toString().trim();
  return DirectSecp256k1Wallet.fromKey(fromHex(key), "xion");
}

async function getSecpKeypair(num: number) {
  const key = readFileSync(`keys/main.key`).toString().trim();
  return await Secp256k1.makeKeypair(fromHex(key));
}
