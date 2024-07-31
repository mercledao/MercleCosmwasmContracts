import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { fromHex } from "@cosmjs/encoding";
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { expect } from "chai";
import { readFileSync } from "fs";

const rpc = "http://localhost:26657";

describe("MembershipNFTV", async () => {
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

  const defaultParams = {
    name: "TEST TOKEN",
    symbol: "TEST",
    minter: account1.address,
    claim_issuer: account1.address,

    is_open_mint: false,
    is_single_mint: true,
    is_tradable: false,
  };

  describe("Initialization", async () => {
    it("Should correctly initialize the contract", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
        minter: account3.address,
      });

      const client = await getClientForSigner(signer2);

      const [
        contractInfo,
        creatorInfo,
        claimIssuerHasRole,
        creatorHasClaimIssuerRole,
        creatorHasDefaultAdminRole,
        creatorHasMinterRole,
        minterHasRole,
        isOpenMint,
        isSingleMint,
        isTradable,
      ] = await Promise.all([
        client.queryContractSmart(instance.contractAddress, {
          contract_info: {},
        }),
        client.queryContractSmart(instance.contractAddress, {
          creator: {},
        }),

        client.queryContractSmart(instance.contractAddress, {
          has_role: {
            address: account2.address,
            role: "ClaimIssuer",
          },
        }),
        client.queryContractSmart(instance.contractAddress, {
          has_role: {
            address: account1.address,
            role: "ClaimIssuer",
          },
        }),
        client.queryContractSmart(instance.contractAddress, {
          has_role: {
            address: account1.address,
            role: "DefaultAdmin",
          },
        }),
        client.queryContractSmart(instance.contractAddress, {
          has_role: {
            address: account1.address,
            role: "Minter",
          },
        }),
        client.queryContractSmart(instance.contractAddress, {
          has_role: {
            address: account3.address,
            role: "Minter",
          },
        }),
        client.queryContractSmart(instance.contractAddress, {
          is_open_mint: {},
        }),
        client.queryContractSmart(instance.contractAddress, {
          is_single_mint: {},
        }),
        client.queryContractSmart(instance.contractAddress, {
          is_tradable: {},
        }),
      ]);

      expect(contractInfo.name).equal(defaultParams.name);
      expect(contractInfo.symbol).equal(defaultParams.symbol);
      expect(creatorInfo.creator).equal(account1.address);
      expect(claimIssuerHasRole.value).equal(true);
      expect(creatorHasClaimIssuerRole.value).equal(true);
      expect(creatorHasDefaultAdminRole.value).equal(true);
      expect(creatorHasMinterRole.value).equal(true);
      expect(minterHasRole.value).equal(true);
      expect(isOpenMint.value).equal(defaultParams.is_open_mint);
      expect(isSingleMint.value).equal(defaultParams.is_single_mint);
      expect(isTradable.value).equal(defaultParams.is_tradable);
    });
  });

  describe("Basic NFT Functionalities", async () => {
    it("Allows blacklisting of accounts", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
        is_single_mint: false,
        is_tradable: true,
      });

      let client = await getClientForSigner(signer1);

      const isBlacklisted = await client.queryContractSmart(
        instance.contractAddress,
        {
          has_role: {
            address: account3.address,
            role: "Blacklisted",
          },
        }
      );

      expect(isBlacklisted.value).equal(false);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          grant_role: {
            address: account3.address,
            role: "Blacklisted",
          },
        },
        "auto"
      );

      try {
        await client.execute(
          account1.address,
          instance.contractAddress,
          {
            mint: {
              owner: account3.address,
              token_uri: "TESTURI",
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Blacklisted address"));
      }

      client = await getClientForSigner(signer3);

      try {
        await client.execute(
          account3.address,
          instance.contractAddress,
          {
            revoke_role: {
              address: account3.address,
              role: "Blacklisted",
            },
          },
          "auto"
        );
      } catch (e: any) {
        expect(e.message.includes("Unauthorized"));
      }

      client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          revoke_role: {
            address: account3.address,
            role: "Blacklisted",
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account3.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account3.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          grant_role: {
            address: account3.address,
            role: "Blacklisted",
          },
        },
        "auto"
      );

      client = await getClientForSigner(signer3);

      try {
        await client.execute(
          account3.address,
          instance.contractAddress,
          {
            transfer_nft: {
              recipient: account4.address,
              token_id: "1",
            },
          },
          "auto"
        );
      } catch (e: any) {
        expect(e.message.includes("Blacklisted address"));
      }

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          burn: {
            token_id: "1",
          },
        },
        "auto"
      );

      client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          revoke_role: {
            address: account3.address,
            role: "Blacklisted",
          },
        },
        "auto"
      );

      client = await getClientForSigner(signer3);

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          transfer_nft: {
            recipient: account4.address,
            token_id: "2",
          },
        },
        "auto"
      );
    });
  });

  describe("Functionality Switches", async () => {
    it("setIsOpenMint : Works as expected", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
        is_single_mint: false,
      });

      let client = await getClientForSigner(signer3);

      const isOpenMint = await client.queryContractSmart(
        instance.contractAddress,
        {
          is_open_mint: {},
        }
      );

      expect(isOpenMint.value).equal(false);

      try {
        await client.execute(
          account3.address,
          instance.contractAddress,
          {
            mint: {
              owner: account3.address,
              token_uri: "TESTURI",
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Unauthorized"));
      }

      try {
        await client.execute(
          account3.address,
          instance.contractAddress,
          {
            set_is_open_mint: {
              value: true,
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Unauthorized"));
      }

      client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          set_is_open_mint: {
            value: true,
          },
        },
        "auto"
      );

      client = await getClientForSigner(signer3);

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          mint: {
            owner: account3.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );
    });

    it("setIsTradable : Works as expected", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
      });

      let client = await getClientForSigner(signer1);

      const isTradable = await client.queryContractSmart(
        instance.contractAddress,
        {
          is_tradable: {},
        }
      );

      expect(isTradable.value).equal(false);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      try {
        await client.execute(
          account1.address,
          instance.contractAddress,
          {
            transfer_nft: {
              recipient: account2.address,
              token_id: "1",
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Soulbound Tokens"));
      }

      client = await getClientForSigner(signer3);

      try {
        await client.execute(
          account3.address,
          instance.contractAddress,
          {
            set_is_tradable: {
              value: true,
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Unauthorized"));
      }

      client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          set_is_tradable: {
            value: true,
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          transfer_nft: {
            recipient: account2.address,
            token_id: "1",
          },
        },
        "auto"
      );
    });

    it("setHasMinted : Works as expected", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
        is_open_mint: true,
      });

      let client = await getClientForSigner(signer1);

      const hasMinted = await client.queryContractSmart(
        instance.contractAddress,
        {
          has_minted: {
            address: account3.address,
          },
        }
      );

      expect(hasMinted.value).equal(false);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account3.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      const hasMintedAfterMint = await client.queryContractSmart(
        instance.contractAddress,
        {
          has_minted: {
            address: account3.address,
          },
        }
      );

      expect(hasMintedAfterMint.value).equal(true);

      try {
        await client.execute(
          account1.address,
          instance.contractAddress,
          {
            mint: {
              owner: account3.address,
              token_uri: "TESTURI",
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Already claimed"));
      }

      client = await getClientForSigner(signer3);
      try {
        await client.execute(
          account3.address,
          instance.contractAddress,
          {
            set_has_minted: {
              address: account3.address,
              value: false,
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Unauthorized"));
      }

      client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          set_has_minted: {
            address: account3.address,
            value: false,
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account3.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );
    });
  });

  describe("Minting functionalities", async () => {
    it("Allows creator to mint nft", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
      });

      const client = await getClientForSigner(signer1);

      const tokensBefore = await client.queryContractSmart(
        instance.contractAddress,
        {
          get_tokens_for_owner: {
            address: account1.address,
          },
        }
      );

      expect(tokensBefore.tokens.length).equal(0);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account2.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      const [tokensAfter, tokensForNonMinter] = await Promise.all([
        client.queryContractSmart(instance.contractAddress, {
          get_tokens_for_owner: {
            address: account2.address,
          },
        }),

        client.queryContractSmart(instance.contractAddress, {
          get_tokens_for_owner: {
            address: account1.address,
          },
        }),
      ]);

      expect(tokensAfter.tokens.length).equal(1);
      expect(tokensForNonMinter.tokens.length).equal(0);
    });

    it("Allows minter role to mint nft", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        minter: account2.address,
      });

      const client = await getClientForSigner(signer2);

      await client.execute(
        account2.address,
        instance.contractAddress,
        {
          mint: {
            owner: account2.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      const tokens = await client.queryContractSmart(instance.contractAddress, {
        get_tokens_for_owner: {
          address: account2.address,
        },
      });

      expect(tokens.tokens.length).equal(1);
    });

    it("Allows non-minter roles to mint when open mint is enabled and reverts if not", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
      });

      let client = await getClientForSigner(signer3);

      try {
        await client.execute(
          account3.address,
          instance.contractAddress,
          {
            mint: {
              owner: account3.address,
              token_uri: "TESTURI",
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Unauthorized"));
      }

      client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          set_is_open_mint: {
            value: true,
          },
        },
        "auto"
      );

      client = await getClientForSigner(signer3);

      await client.execute(
        account3.address,
        instance.contractAddress,
        {
          mint: {
            owner: account3.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );
    });

    it("Doesnt allow multiple mints when single mint is enabled and allows if not", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
      });

      let client = await getClientForSigner(signer1);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      try {
        await client.execute(
          account1.address,
          instance.contractAddress,
          {
            mint: {
              owner: account1.address,
              token_uri: "TESTURI",
            },
          },
          "auto"
        );
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("Already claimed")).equal(true);
      }

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          set_is_single_mint: {
            value: false,
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );
    });
  });

  describe("Read only helper functions", async () => {
    it("getActiveTokenId : Returns the latest id", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
        is_single_mint: false,
        is_tradable: true,
      });

      const client = await getClientForSigner(signer1);

      try {
        const res = await client.queryContractSmart(instance.contractAddress, {
          get_active_token_id: {
            address: account1.address,
          },
        });

        console.log(res);
        expect(true).equal(false);
      } catch (e: any) {
        expect(e.message.includes("No tokens"));
      }

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      const res = await client.queryContractSmart(instance.contractAddress, {
        get_active_token_id: {
          address: account1.address,
        },
      });

      expect(+res.value).equal(2);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          transfer_nft: {
            recipient: account2.address,
            token_id: "2",
          },
        },
        "auto"
      );

      const afterTtransfer = await client.queryContractSmart(
        instance.contractAddress,
        {
          get_active_token_id: {
            address: account1.address,
          },
        }
      );

      expect(+afterTtransfer.value).equal(1);
    });

    it("getTokenIdsForOwner : Returns all token ids owned by owner", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
        is_single_mint: false,
        is_tradable: true,
      });

      const client = await getClientForSigner(signer1);

      const res = await client.queryContractSmart(instance.contractAddress, {
        get_tokens_for_owner: {
          address: account1.address,
        },
      });

      expect(res.tokens.length).equal(0);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      const afterMint = await client.queryContractSmart(
        instance.contractAddress,
        {
          get_tokens_for_owner: {
            address: account1.address,
          },
        }
      );

      expect(afterMint.tokens.length).equal(2);
      expect(JSON.stringify(afterMint.tokens)).equal(
        JSON.stringify(["1", "2"])
      );
      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          transfer_nft: {
            recipient: account2.address,
            token_id: "2",
          },
        },
        "auto"
      );

      const afterTransfer = await client.queryContractSmart(
        instance.contractAddress,
        {
          get_tokens_for_owner: {
            address: account1.address,
          },
        }
      );

      expect(JSON.stringify(afterTransfer.tokens)).equal(JSON.stringify(["1"]));
    });

    it("getTokenDetailsBulk : Returns all token ids requested", async () => {
      const instance = await getContract(signer1, {
        ...defaultParams,
        claim_issuer: account2.address,
        is_single_mint: false,
        is_tradable: true,
      });

      const client = await getClientForSigner(signer1);

      const res = await client.queryContractSmart(instance.contractAddress, {
        get_token_details_bulk: {
          limit: 10,
        },
      });

      expect(res.tokens.length).equal(0);

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      await client.execute(
        account1.address,
        instance.contractAddress,
        {
          mint: {
            owner: account1.address,
            token_uri: "TESTURI",
          },
        },
        "auto"
      );

      const afterMint = await client.queryContractSmart(
        instance.contractAddress,
        {
          get_token_details_bulk: {
            limit: 10,
          },
        }
      );

      expect(afterMint.tokens.length).equal(2);
      expect(afterMint.tokens[0][1].owner).equal(account1.address);
      expect(afterMint.tokens[0][1].token_uri).equal("TESTURI");
    });
  });

  async function getSigner(num: number): Promise<DirectSecp256k1Wallet> {
    const key = readFileSync(`keys/account${num}.key`).toString().trim();
    return DirectSecp256k1Wallet.fromKey(fromHex(key), "xion");
  }
});

async function getContract(deployer: DirectSecp256k1Wallet, params: any) {
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

async function getClientForSigner(
  signer: DirectSecp256k1Wallet
): Promise<SigningCosmWasmClient> {
  const client = await SigningCosmWasmClient.connectWithSigner(rpc, signer, {
    gasPrice: GasPrice.fromString("4uxion"),
  });

  return client;
}
