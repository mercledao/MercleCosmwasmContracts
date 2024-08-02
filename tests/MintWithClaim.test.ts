import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { Secp256k1, sha256 } from "@cosmjs/crypto";
import { fromHex, toBase64 } from "@cosmjs/encoding";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import { coin, GasPrice } from "@cosmjs/stargate";
import { expect } from "chai";
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

async function getSignatureForMessage(message: any, keypairNum: number) {
  const messageHash = new TextEncoder().encode(JSON.stringify(message));
  const keypair = await getSecpKeypair(keypairNum);
  const signature = await Secp256k1.createSignature(
    sha256(messageHash),
    keypair.privkey
  );
  // Extract the 64-byte signature and recovery byte
  const signatureBytes = signature.toFixedLength();
  const signatureWithoutRecoveryByte = signatureBytes.slice(0, 64);

  return {
    signature: toBase64(signatureWithoutRecoveryByte),
    recovery: signature.recovery,
  };
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

  describe("Initialization", async () => {
    it("Correctly initializes the contract", async () => {
      const instance = await getContract(signer1, {
        treasury: account5.address,
      });
      const client = await getClientForSigner(signer1);
      const [treasury, creatorHasRole] = await Promise.all([
        client.queryContractSmart(instance.contractAddress, {
          get_treasury: {},
        }),
        client.queryContractSmart(instance.contractAddress, {
          has_role: {
            address: account1.address,
            role: "DefaultAdmin",
          },
        }),
      ]);

      expect(treasury.value).equal(account5.address);
      expect(creatorHasRole.value).equal(true);
    });
  });

  describe("Claim Verification", async () => {
    it("Allows mint for legit claims", async () => {
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
        token_uri: "TEST-URI",
        fee: {
          denom: "uxion",
          amount: "1000",
        },
        verifying_contract: membershipInstance.contractAddress,
        chain_id: "xion-testnet-1",
        bech32_hre: "xion",
        timestamp: String(+Date.now()),
      };

      client = await getClientForSigner(signer3);

      try {
        await client.queryContractSmart(membershipInstance.contractAddress, {
          get_active_token_id: {
            address: account3.address,
          },
        });

        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("No tokens"));
      }

      const { signature, recovery } = await getSignatureForMessage(message, 2);

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          mint_with_claim: {
            message: message,
            signature: signature,
            recovery_byte: recovery,
          },
        },
        "auto",
        "",
        [coin(1000, "uxion")]
      );

      const afterClaim = await client.queryContractSmart(
        membershipInstance.contractAddress,
        {
          get_active_token_id: {
            address: account3.address,
          },
        }
      );

      expect(+afterClaim.value).equal(1);
    });

    it("Verifies claim correctly", async () => {
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
        token_uri: "TEST-URI",
        fee: {
          denom: "uxion",
          amount: "1000",
        },
        verifying_contract: membershipInstance.contractAddress,
        chain_id: "xion-testnet-1",
        bech32_hre: "xion",
        timestamp: String(+Date.now()),
      };

      const { signature, recovery } = await getSignatureForMessage(message, 2);

      const res = await client.queryContractSmart(instance.contractAddress, {
        verify_sign: {
          message: message,
          signature: signature,
          recovery_byte: recovery,
        },
      });

      expect(res.value).equal(true);
    });

    it("Verification fails for fake claims", async () => {
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
        token_uri: "TEST-URI",
        fee: {
          denom: "uxion",
          amount: "1000",
        },
        verifying_contract: membershipInstance.contractAddress,
        chain_id: "xion-testnet-1",
        bech32_hre: "xion",
        timestamp: String(+Date.now()),
      };

      // Create a sign from a non-claim issuer address
      const { signature: nonIssuerSig, recovery: nonIssuerRecovery } =
        await getSignatureForMessage(message, 3);

      const nonIssuerRes = await client.queryContractSmart(
        instance.contractAddress,
        {
          verify_sign: {
            message: message,
            signature: nonIssuerSig,
            recovery_byte: nonIssuerRecovery,
          },
        }
      );

      expect(nonIssuerRes.value).equal(false);

      const { signature, recovery } = await getSignatureForMessage(message, 2);

      // Try changing the message
      message.fee.amount = "100";
      const fakeClaimRes = await client.queryContractSmart(
        instance.contractAddress,
        {
          verify_sign: {
            message: message,
            signature: signature,
            recovery_byte: recovery,
          },
        }
      );

      expect(fakeClaimRes.value).equal(false);

      // Should fail for signature reuse:
      client = await getClientForSigner(signer3);

      message.fee.amount = "1000";

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          mint_with_claim: {
            message: message,
            signature: signature,
            recovery_byte: recovery,
          },
        },
        "auto",
        "",
        [coin(1000, "uxion")]
      );

      const reuseRes = await client.queryContractSmart(
        instance.contractAddress,
        {
          verify_sign: {
            message: message,
            signature: signature,
            recovery_byte: recovery,
          },
        }
      );

      expect(reuseRes.value).equal(false);
    });
  });

  describe("Treasury", async () => {
    it("Transfers funds to treasury upon mint", async () => {
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
        token_uri: "TEST-URI",
        fee: {
          denom: "uxion",
          amount: "1000",
        },
        verifying_contract: membershipInstance.contractAddress,
        chain_id: "xion-testnet-1",
        bech32_hre: "xion",
        timestamp: String(+Date.now()),
      };

      client = await getClientForSigner(signer3);

      const { signature, recovery } = await getSignatureForMessage(message, 2);

      const treasuryBalBefore = await client.getBalance(
        account5.address,
        "uxion"
      );

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          mint_with_claim: {
            message: message,
            signature: signature,
            recovery_byte: recovery,
          },
        },
        "auto",
        "",
        [coin(1000, "uxion")]
      );

      const treasuryBalAfter = await client.getBalance(
        account5.address,
        "uxion"
      );

      expect(+treasuryBalAfter.amount - +treasuryBalBefore.amount).equal(1000);
    });

    it("Allows changing treasury address by admin", async () => {
      const instance = await getContract(signer1, {
        treasury: account5.address,
      });
      let client = await getClientForSigner(signer1);
      const membershipInstance = await getMembershipContract(signer1, {
        ...defaultParams,
        minter: instance.contractAddress,
        claim_issuer: account2.address,
      });

      client = await getClientForSigner(signer4);

      try {
        await client.execute(
          account4.address,
          instance.contractAddress,
          {
            set_treasury: {
              address: account4.address,
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Unauthorized")).equal(true);
      }

      client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          set_treasury: {
            address: account4.address,
          },
        },
        "auto"
      );

      const message = {
        from: account2.address,
        to: account3.address,
        token_uri: "TEST-URI",
        fee: {
          denom: "uxion",
          amount: "1000",
        },
        verifying_contract: membershipInstance.contractAddress,
        chain_id: "xion-testnet-1",
        bech32_hre: "xion",
        timestamp: String(+Date.now()),
      };

      client = await getClientForSigner(signer3);

      const { signature, recovery } = await getSignatureForMessage(message, 2);

      const treasuryBalBefore = await client.getBalance(
        account4.address,
        "uxion"
      );

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          mint_with_claim: {
            message: message,
            signature: signature,
            recovery_byte: recovery,
          },
        },
        "auto",
        "",
        [coin(1000, "uxion")]
      );

      const treasuryBalAfter = await client.getBalance(
        account4.address,
        "uxion"
      );

      expect(+treasuryBalAfter.amount - +treasuryBalBefore.amount).equal(1000);
    });

    it("Accepts mint fees in a different denom", async () => {
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
        to: account4.address,
        token_uri: "TEST-URI",
        fee: {
          denom: "utest",
          amount: "1000",
        },
        verifying_contract: membershipInstance.contractAddress,
        chain_id: "xion-testnet-1",
        bech32_hre: "xion",
        timestamp: String(+Date.now()),
      };

      // Account4 is loaded with utest tokens in the build script
      client = await getClientForSigner(signer4);

      try {
        await client.queryContractSmart(membershipInstance.contractAddress, {
          get_active_token_id: {
            address: account4.address,
          },
        });

        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("No tokens"));
      }

      const { signature, recovery } = await getSignatureForMessage(message, 2);

      const treasuryBalBefore = await client.getBalance(
        account5.address,
        "utest"
      );

      await client.execute(
        account4.address,
        instance.contractAddress,
        {
          mint_with_claim: {
            message: message,
            signature: signature,
            recovery_byte: recovery,
          },
        },
        "auto",
        "",
        [coin(1000, "utest")]
      );

      const afterClaim = await client.queryContractSmart(
        membershipInstance.contractAddress,
        {
          get_active_token_id: {
            address: account4.address,
          },
        }
      );

      const treasuryBalAfter = await client.getBalance(
        account5.address,
        "utest"
      );

      expect(+afterClaim.value).equal(1);
      expect(+treasuryBalAfter.amount - +treasuryBalBefore.amount).equal(1000);
    });
  });
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
