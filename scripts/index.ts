import {
    DirectSecp256k1HdWallet,
    OfflineDirectSigner,
} from "@cosmjs/proto-signing";
import { StargateClient } from "@cosmjs/stargate";
import { readFile } from "fs/promises";

const rpc = "http://localhost:26657";
async function main() {
  const signer = await getAliceSignerFromMnemonic();
  const [acc] = await signer.getAccounts();
  console.log(acc.address);

  const client = await StargateClient.connect(rpc);
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
