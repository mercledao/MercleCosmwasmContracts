import {
  BasicBackendApi,
  BasicKVIterStorage,
  BasicQuerier,
  IBackend,
  VMInstance,
} from "@terran-one/cosmwasm-vm-js";
import { readFileSync } from "fs";


const wasmBytecode = readFileSync(
  "./contracts/target/wasm32-unknown-unknown/release/mercle_cw_contracts.wasm"
);

const backend: IBackend = {
  backend_api: new BasicBackendApi("xion"),
  storage: new BasicKVIterStorage(),
  querier: new BasicQuerier(),
};

const vm = new VMInstance(backend);
const mockEnv = {
  block: {
    height: 1337,
    time: "2000000000",
    chain_id: "xion-testnet-1",
  },
  contract: {
    address: "terra14z56l0fp2lsf86zy3hty2z47ezkhnthtr9yq76",
  },
};

const mockInfo = {
  sender: "terra1337xewwfv3jdjuz8e0nea9vd8dpugc0k2dcyt3",
  funds: [],
};

describe("CosmWasm", async () => {
  it("Works", async () => {
    await vm.build(wasmBytecode);

    const region = vm.instantiate(mockEnv, mockInfo, {});

    console.log(region.json);
  });
});
