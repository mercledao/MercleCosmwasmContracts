import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { ripemd160, Secp256k1, sha256 } from "@cosmjs/crypto";
import { fromHex, toBase64, toBech32 } from "@cosmjs/encoding";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import { coin, GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";

async function getMembershipContract(
  deployer: DirectSecp256k1Wallet,
  params: any
) {
  const [acc] = await deployer.getAccounts();
  const client = await getClientForSigner(deployer);

  const upload = await client.upload(
    acc.address,
    readFileSync(
      "./contracts/MembershipNFT/artifacts/mercle_nft_membership.wasm"
    ),
    "auto"
  );

  const instance = await client.instantiate(
    acc.address,
    upload.codeId,
    params,
    "test",
    "auto"
  );

  return instance;
}

const defaultParams = {
  name: "TEST TOKEN",
  symbol: "TEST",
  minter: "",
  claim_issuer: "",

  is_open_mint: false,
  is_single_mint: true,
  is_tradable: false,
};

function pubkeyToBech32(pubkey: Uint8Array, prefix: string) {
  const sha256Hash = sha256(pubkey);

  // Hash the SHA-256 hash using RIPEMD-160
  const ripemd160Hash = ripemd160(sha256Hash);

  // Convert the RIPEMD-160 hash to a Bech32 address
  const bech32Address = toBech32(prefix, ripemd160Hash);

  return bech32Address;
}

const rpc = "http://localhost:26657";

describe("MintWithClaim", async () => {
  const [signer1, signer2, signer3, signer4, signer5] = await Promise.all([
    getSigner(1),
    getSigner(2),
    getSigner(3),
    getSigner(4),
    getSigner(5),
  ]);

  const [[account1], [account2], [account3], [account4], [account5]] =
    await Promise.all([
      signer1.getAccounts(),
      signer2.getAccounts(),
      signer3.getAccounts(),
      signer4.getAccounts(),
      signer5.getAccounts(),
    ]);
  describe("Claim works", async () => {
    it("Mints an NFT", async () => {
      const instance = await getContract(signer1, {
        treasury: account5.address,
      });
      let client = await getClientForSigner(signer1);
      const membershipInstance = await getMembershipContract(signer1, {
        ...defaultParams,
        minter: instance.contractAddress,
        claim_issuer: account2.address,
      });

      const message = {
        from: account2.address,
        to: account3.address,
        token_uri: "TEST",
        fee: {
          denom: "uxion",
          amount: "800",
        },
        verifying_contract: membershipInstance.contractAddress,
        chain_id: "xion-testnet-1",
        bech32_hre: "xion",
      };

      const messageHash = new TextEncoder().encode(JSON.stringify(message));
      const keypair = await getSecpKeypair(2);
      const signature = await Secp256k1.createSignature(
        sha256(messageHash),
        keypair.privkey
      );
      // Extract the 64-byte signature and recovery byte
      const signatureBytes = signature.toFixedLength();
      const signatureWithoutRecoveryByte = signatureBytes.slice(0, 64);

      client = await getClientForSigner(signer3);
      const res = await client.execute(
        account3.address,
        instance.contractAddress,
        {
          mint_with_claim: {
            message: message,
            signature: toBase64(signatureWithoutRecoveryByte),
            recovery_byte: signature.recovery,
          },
        },
        "auto",
        "",
        [coin(800, "uxion")]
      );

      console.log(res);
    });

    // it("Verifies claim correctly", async () => {
    //   const instance = await getContract(signer1, {
    //     treasury: account5.address,
    //   });
    //   const client = await getClientForSigner(signer1);
    //   const message = {
    //     from: account5.address,
    //     to: account1.address,
    //     token_uri: "TEST",
    //     fee: {
    //       denom: "uxion",
    //       amount: "8",
    //     },
    //     verifying_contract: instance.contractAddress,
    //     chain_id: "xion-testnet-1",
    //     bech32_hre: "xion",
    //   };
    //   const messageHash = new TextEncoder().encode(JSON.stringify(message));
    //   const keypair = await getSecpKeypair(1);
    //   const signature = await Secp256k1.createSignature(
    //     sha256(messageHash),
    //     keypair.privkey
    //   );
    //   // Extract the 64-byte signature and recovery byte
    //   const signatureBytes = signature.toFixedLength();
    //   const signatureWithoutRecoveryByte = signatureBytes.slice(0, 64);
    //   const res = await client.queryContractSmart(instance.contractAddress, {
    //     verify_sign: {
    //       message: message,
    //       signature: toBase64(signatureWithoutRecoveryByte),
    //       recovery_byte: signature.recovery,
    //     },
    //   });
    //   expect(res.value).equal(account1.address);
    // });
  });
  // describe("Initialization", async () => {
  //   it("Correctly initializes the contract", async () => {
  //     const instance = await getContract(signer1, {
  //       treasury: account5.address,
  //     });
  //     const client = await getClientForSigner(signer1);
  //     const [treasury, creatorHasRole] = await Promise.all([
  //       client.queryContractSmart(instance.contractAddress, {
  //         get_treasury: {},
  //       }),
  //       client.queryContractSmart(instance.contractAddress, {
  //         has_role: {
  //           address: account1.address,
  //           role: "DefaultAdmin",
  //         },
  //       }),
  //     ]);

  //     expect(treasury.value).equal(account5.address);
  //     expect(creatorHasRole.value).equal(true);
  //   });
  // });
});

async function getContract(deployer: DirectSecp256k1Wallet, params: any) {
  const [acc] = await deployer.getAccounts();
  const client = await getClientForSigner(deployer);

  const upload = await client.upload(
    acc.address,
    readFileSync(
      "./contracts/MintWithClaim/artifacts/mercle_mint_with_claim.wasm"
    ),
    "auto"
  );

  const instance = await client.instantiate(
    acc.address,
    upload.codeId,
    params,
    "test",
    "auto"
  );

  return instance;
}

async function getClientForSigner(
  signer: DirectSecp256k1Wallet
): Promise<SigningCosmWasmClient> {
  const client = await SigningCosmWasmClient.connectWithSigner(rpc, signer, {
    gasPrice: GasPrice.fromString("4uxion"),
  });

  return client;
}

async function getSigner(num: number): Promise<DirectSecp256k1Wallet> {
  const key = readFileSync(`keys/account${num}.key`).toString().trim();
  return DirectSecp256k1Wallet.fromKey(fromHex(key), "xion");
}
async function getSecpKeypair(num: number) {
  const key = readFileSync(`keys/account${num}.key`).toString().trim();
  return await Secp256k1.makeKeypair(fromHex(key));
}
