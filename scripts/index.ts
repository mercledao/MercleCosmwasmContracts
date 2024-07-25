import { SigningStargateClient } from "@cosmjs/stargate";
import { DirectSecp256k1HdWallet, OfflineDirectSigner } from "cosmwasm";
import { readFile } from "fs/promises";

const rpc = "http://localhost:26657";

async function main() {
  const signer = await getAliceSignerFromMnemonic();
  const [acc] = await signer.getAccounts();

  const client = await SigningStargateClient.connectWithSigner(rpc, signer);
  console.log(await client.getAllBalances(acc.address));
}

main();

async function getAliceSignerFromMnemonic(): Promise<OfflineDirectSigner> {
  return DirectSecp256k1HdWallet.fromMnemonic(
    (await readFile("./keys/account1.key")).toString(),
    {
      prefix: "wasm",
    }
  );
}
