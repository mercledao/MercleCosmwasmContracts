use crate::{
    helpers::{get_key_for_role, recover_signer},
    msg::{HasRoleResponse, MemberhsipQuery, Message},
};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, QueryRequest, StdResult, Storage, WasmQuery,
};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum Role {
    DefaultAdmin,
    ClaimIssuer,
    Minter,
    Blacklisted,
}

pub struct MintWithClaimContract<'a, C> {
    pub treasury: Item<'a, Addr>,
    pub claim_map: Map<'a, &'a [u8], bool>,
    pub role_map: Map<'a, (&'a Addr, &'a str), bool>,

    pub(crate) _custom_response: PhantomData<C>,
}

impl<C> Default for MintWithClaimContract<'static, C> {
    fn default() -> Self {
        Self::new("treasury", "claim_map", "role_map")
    }
}

impl<'a, C> MintWithClaimContract<'a, C> {
    fn new(treasury_key: &'a str, claimed_map_key: &'a str, role_map_key: &'a str) -> Self {
        Self {
            treasury: Item::new(treasury_key),
            claim_map: Map::new(claimed_map_key),
            role_map: Map::new(role_map_key),
            _custom_response: PhantomData,
        }
    }

    pub fn update_role(
        &self,
        storage: &mut dyn Storage,
        address: &Addr,
        role: Role,
        value: bool,
    ) -> StdResult<()> {
        let key = get_key_for_role(role);
        self.role_map.save(storage, (address, key), &value)?;
        Ok(())
    }

    pub fn has_role(&self, storage: &dyn Storage, address: &Addr, role: Role) -> StdResult<bool> {
        let key = get_key_for_role(role);
        let val = self
            .role_map
            .load(storage, (address, key))
            .unwrap_or_default();

        Ok(val)
    }

    pub fn validate_claim(
        &self,
        deps: Deps,
        message: Message,
        signature: Binary,
        recovery_byte: u8,
    ) -> StdResult<(bool, bool, bool)> {
        let addr = recover_signer(
            deps,
            message.to_owned(),
            signature.to_owned(),
            recovery_byte,
        )?;

        let has_claim_issuer_role_msg: MemberhsipQuery = MemberhsipQuery::HasRole {
            address: addr.to_owned(),
            role: Role::ClaimIssuer,
        };

        let query_response: HasRoleResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: message.verifying_contract.to_string(),
                msg: to_json_binary(&has_claim_issuer_role_msg)?,
            }))?;

        let has_role = query_response.value;

        let is_sign_valid = &message.from == addr;

        let is_duplicate = self
            .claim_map
            .may_load(deps.storage, &signature)?
            .unwrap_or_default();
        deps.api.debug(&format!("Is duplicate: {}", is_duplicate));

        Ok((is_duplicate, is_sign_valid, has_role))
    }
}
