use cosmwasm_std::{Addr, CustomMsg, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::helpers::get_key_for_role;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum Role {
    DefaultAdmin,
    ClaimIssuer,
    Minter,
    Blacklisted,
}

pub struct MintWithClaimContract<'a, C, E, Q>
where
    Q: CustomMsg,
    E: CustomMsg,
{
    pub treasury: Item<'a, Addr>,
    pub claim_map: Map<'a, String, bool>,
    pub role_map: Map<'a, (&'a Addr, &'a str), bool>,

    pub(crate) _custom_response: PhantomData<C>,
    pub(crate) _custom_query: PhantomData<Q>,
    pub(crate) _custom_execute: PhantomData<E>,
}

impl<C, E, Q> Default for MintWithClaimContract<'static, C, E, Q>
where
    E: CustomMsg,
    Q: CustomMsg,
{
    fn default() -> Self {
        Self::new("treasury", "claim_map", "role_map")
    }
}

impl<'a, C, E, Q> MintWithClaimContract<'a, C, E, Q>
where
    E: CustomMsg,
    Q: CustomMsg,
{
    fn new(treasury_key: &'a str, claimed_map_key: &'a str, role_map_key: &'a str) -> Self {
        Self {
            treasury: Item::new(treasury_key),
            claim_map: Map::new(claimed_map_key),
            role_map: Map::new(role_map_key),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
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
}
