import { Secp256k1HdWallet } from "@cosmjs/amino";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Decimal } from "@cosmjs/math";
import { expect } from "chai";
import { readFileSync } from "fs";
import { readFile } from "fs/promises";

const rpc = "http://localhost:26657";

async function main() {
  const signer = await getAliceSignerFromMnemonic();
  const [acc] = await signer.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(rpc, signer, {
    gasPrice: { amount: Decimal.fromUserInput("0.1", 3), denom: "uxion" },
  });
  const upload = await client.upload(
    acc.address,
    readFileSync("./contracts/artifacts/contract.wasm"),
    "auto"
  );

  console.log("Instantiating contract.....");
  const instance = await client.instantiate(
    acc.address,
    upload.codeId,
    {
      count: "100",
    },
    "test",
    "auto"
  );

  const countRes = await client.queryContractSmart(instance.contractAddress, {
    current: {},
  });

  console.log("Testing if correctly initialized");
  expect(+countRes.message).equal(100);

  await client.execute(
    acc.address,
    instance.contractAddress,
    "increment",
    "auto"
  );

  const countResAgain = await client.queryContractSmart(
    instance.contractAddress,
    {
      current: {},
    }
  );

  console.log("Expecting increment");
  expect(+countResAgain.message).equal(101);

  console.log("Deploy success");
}

main();

async function getAliceSignerFromMnemonic() {
  return Secp256k1HdWallet.fromMnemonic(
    (await readFile("./keys/account1.key")).toString(),
    {
      prefix: "xion",
    }
  );
}
