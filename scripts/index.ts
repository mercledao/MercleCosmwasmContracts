import { Secp256k1HdWallet } from "@cosmjs/amino";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Decimal } from "@cosmjs/math";
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
    readFileSync("./contracts/artifacts/mercle_wasm_contracts.wasm"),
    "auto"
  );

  console.log("Instantiating contract.....");
  const instance = await client.instantiate(
    acc.address,
    upload.codeId,
    {
      name: "test",
      symbol: "test",
      minter: acc.address,
      claim_issuer: acc.address,
    },
    "test",
    "auto"
  );

  console.log("Deployed at ", instance.contractAddress);

  await client.execute(
    acc.address,
    instance.contractAddress,
    {
      mint: {
        owner: acc.address,
        token_uri: "adf",
      },
    },
    "auto"
  );

  const res = await client.queryContractSmart(instance.contractAddress, {
    nft_info: {
      token_id: "1",
    },
  });

  console.log(res);
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
