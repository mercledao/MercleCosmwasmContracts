use crate::helpers::get_key_for_role;
use cosmwasm_std::{Addr, BlockInfo, CustomMsg, StdResult, Storage};
use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum Role {
    DefaultAdmin,
    ClaimIssuer,
    Minter,
    Blacklisted,
}

pub struct Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    Q: CustomMsg,
    E: CustomMsg,
{
    pub contract_info: Item<'a, ContractInfoResponse>,
    pub token_count: Item<'a, u64>,

    pub creator: Item<'a, Addr>,
    pub single_mint: Item<'a, bool>,
    pub open_mint: Item<'a, bool>,
    pub tradable: Item<'a, bool>,

    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,
    pub role_map: Map<'a, (&'a Addr, &'a str), bool>,
    pub claim_map: Map<'a, Addr, bool>,

    pub(crate) _custom_response: PhantomData<C>,
    pub(crate) _custom_query: PhantomData<Q>,
    pub(crate) _custom_execute: PhantomData<E>,
}

impl<T, C, E, Q> Default for Cw721Contract<'static, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "num_tokens",
            "operators",
            "tokens",
            "tokens__owner",
            "role_map",
            "creator",
            "is_single_mint",
            "is_open_mint",
            "is_tradable",
            "has_claimed",
        )
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn new(
        contract_key: &'a str,
        token_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
        role_map_key: &'a str,
        creator: &'a str,
        is_single_mint_key: &'a str,
        is_open_mint_key: &'a str,
        is_tradable_key: &'a str,
        has_claimed_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            token_count: Item::new(token_count_key),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            role_map: Map::new(role_map_key),
            creator: Item::new(creator),
            open_mint: Item::new(is_open_mint_key),
            single_mint: Item::new(is_single_mint_key),
            tradable: Item::new(is_tradable_key),
            claim_map: Map::new(has_claimed_key),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
        }
    }

    pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? + 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
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

    pub fn _is_open_mint(&self, storage: &dyn Storage) -> StdResult<bool> {
        Ok(self.open_mint.may_load(storage)?.unwrap_or_default())
    }

    pub fn _set_is_open_mint(&self, storage: &mut dyn Storage, value: bool) -> StdResult<()> {
        self.open_mint.save(storage, &value)
    }

    pub fn _is_single_mint(&self, storage: &dyn Storage) -> StdResult<bool> {
        Ok(self.single_mint.may_load(storage)?.unwrap_or_default())
    }

    pub fn _set_is_single_mint(&self, storage: &mut dyn Storage, value: bool) -> StdResult<()> {
        self.single_mint.save(storage, &value)
    }

    pub fn _is_tradable(&self, storage: &dyn Storage) -> StdResult<bool> {
        Ok(self.tradable.may_load(storage)?.unwrap_or_default())
    }

    pub fn _set_is_tradable(&self, storage: &mut dyn Storage, value: bool) -> StdResult<()> {
        self.tradable.save(storage, &value)
    }

    pub fn _has_claimed(&self, storage: &dyn Storage, address: Addr) -> StdResult<bool> {
        Ok(self.claim_map.load(storage, address).unwrap_or_default())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<T> {
    pub owner: Addr,
    pub approvals: Vec<Approval>,

    pub token_uri: Option<String>,

    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    pub spender: Addr,
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub owner: MultiIndex<'a, Addr, TokenInfo<T>, String>,
}

impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<T>(_pk: &[u8], d: &TokenInfo<T>) -> Addr {
    d.owner.clone()
}
