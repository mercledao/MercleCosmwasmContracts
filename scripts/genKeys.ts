import { DirectSecp256k1HdWallet } from "cosmwasm";

const generateKey = async (): Promise<void> => {
  const wallet = await DirectSecp256k1HdWallet.generate(24);
  process.stdout.write(wallet.mnemonic);
};

generateKey();
