use crate::msg::{
    CreatorResponse, GetActiveTokenIdResponse, GetTokenDetailsBulkResponse,
    GetTokensForOwnerResponse, HasMintedResponse, HasRoleResponse, IsOpenMintResponse,
    IsSingleMintResponse, IsTradableResponse, QueryMsg,
};
use crate::state::{Approval, Cw721Contract, Role, TokenInfo};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, BlockInfo, CustomMsg, Deps, Env, Order, StdError, StdResult,
};
use cw721::{
    AllNftInfoResponse, ApprovalResponse, ApprovalsResponse, ContractInfoResponse, Cw721Query,
    Expiration, NftInfoResponse, NumTokensResponse, OperatorResponse, OperatorsResponse,
    OwnerOfResponse, TokensResponse,
};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;
use serde::de::DeserializeOwned;
use serde::Serialize;

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 100;

impl<'a, T, C, E, Q> Cw721Query<T> for Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn contract_info(&self, deps: Deps) -> StdResult<ContractInfoResponse> {
        self.contract_info.load(deps.storage)
    }

    fn num_tokens(&self, deps: Deps) -> StdResult<NumTokensResponse> {
        let count = self.token_count(deps.storage)?;
        Ok(NumTokensResponse { count })
    }

    fn nft_info(&self, deps: Deps, token_id: String) -> StdResult<NftInfoResponse<T>> {
        let info = self.tokens.load(deps.storage, &token_id)?;
        Ok(NftInfoResponse {
            token_uri: info.token_uri,
            extension: info.extension,
        })
    }

    fn owner_of(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        include_expired: bool,
    ) -> StdResult<OwnerOfResponse> {
        let info = self.tokens.load(deps.storage, &token_id)?;
        Ok(OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info, include_expired),
        })
    }

    fn operator(
        &self,
        deps: Deps,
        env: Env,
        owner: String,
        operator: String,
        include_expired: bool,
    ) -> StdResult<OperatorResponse> {
        let owner_addr = deps.api.addr_validate(&owner)?;
        let operator_addr = deps.api.addr_validate(&operator)?;

        let info = self
            .operators
            .may_load(deps.storage, (&owner_addr, &operator_addr))?;

        if let Some(expires) = info {
            if !include_expired && expires.is_expired(&env.block) {
                return Err(StdError::not_found("Approval not found"));
            }

            return Ok(OperatorResponse {
                approval: cw721::Approval {
                    spender: operator,
                    expires,
                },
            });
        }

        Err(StdError::not_found("Approval not found"))
    }

    fn operators(
        &self,
        deps: Deps,
        env: Env,
        owner: String,
        include_expired: bool,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<OperatorsResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_addr = maybe_addr(deps.api, start_after)?;
        let start = start_addr.as_ref().map(Bound::exclusive);

        let owner_addr = deps.api.addr_validate(&owner)?;
        let res: StdResult<Vec<_>> = self
            .operators
            .prefix(&owner_addr)
            .range(deps.storage, start, None, Order::Ascending)
            .filter(|r| {
                include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block)
            })
            .take(limit)
            .map(parse_approval)
            .collect();
        Ok(OperatorsResponse { operators: res? })
    }

    fn approval(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        spender: String,
        include_expired: bool,
    ) -> StdResult<ApprovalResponse> {
        let token = self.tokens.load(deps.storage, &token_id)?;

        if token.owner == spender {
            let approval = cw721::Approval {
                spender: token.owner.to_string(),
                expires: Expiration::Never {},
            };
            return Ok(ApprovalResponse { approval });
        }

        let filtered: Vec<_> = token
            .approvals
            .into_iter()
            .filter(|t| t.spender == spender)
            .filter(|t| include_expired || !t.is_expired(&env.block))
            .map(|a| cw721::Approval {
                spender: a.spender.into_string(),
                expires: a.expires,
            })
            .collect();

        if filtered.is_empty() {
            return Err(StdError::not_found("Approval not found"));
        }
        let approval = filtered[0].clone();

        Ok(ApprovalResponse { approval })
    }

    fn approvals(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        include_expired: bool,
    ) -> StdResult<ApprovalsResponse> {
        let token = self.tokens.load(deps.storage, &token_id)?;
        let approvals: Vec<_> = token
            .approvals
            .into_iter()
            .filter(|t| include_expired || !t.is_expired(&env.block))
            .map(|a| cw721::Approval {
                spender: a.spender.into_string(),
                expires: a.expires,
            })
            .collect();

        Ok(ApprovalsResponse { approvals })
    }

    fn tokens(
        &self,
        deps: Deps,
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let owner_addr = deps.api.addr_validate(&owner)?;
        let tokens: Vec<String> = self
            .tokens
            .idx
            .owner
            .prefix(owner_addr)
            .keys(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect::<StdResult<Vec<_>>>()?;

        Ok(TokensResponse { tokens })
    }

    fn all_tokens(
        &self,
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let tokens: StdResult<Vec<String>> = self
            .tokens
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(k, _)| k))
            .collect();

        Ok(TokensResponse { tokens: tokens? })
    }

    fn all_nft_info(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        include_expired: bool,
    ) -> StdResult<AllNftInfoResponse<T>> {
        let info = self.tokens.load(deps.storage, &token_id)?;
        Ok(AllNftInfoResponse {
            access: OwnerOfResponse {
                owner: info.owner.to_string(),
                approvals: humanize_approvals(&env.block, &info, include_expired),
            },
            info: NftInfoResponse {
                token_uri: info.token_uri,
                extension: info.extension,
            },
        })
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn is_open_mint(&self, deps: Deps) -> StdResult<IsOpenMintResponse> {
        let value = self._is_open_mint(deps.storage)?;
        Ok(IsOpenMintResponse { value })
    }

    fn is_single_mint(&self, deps: Deps) -> StdResult<IsSingleMintResponse> {
        let value = self._is_single_mint(deps.storage)?;
        Ok(IsSingleMintResponse { value })
    }

    fn is_tradable(&self, deps: Deps) -> StdResult<IsTradableResponse> {
        let value = self._is_tradable(deps.storage)?;
        Ok(IsTradableResponse { value })
    }

    fn get_creator(&self, deps: Deps) -> StdResult<CreatorResponse> {
        let creator = self.creator.may_load(deps.storage)?.unwrap();
        Ok(CreatorResponse { creator })
    }

    fn has_minted(&self, deps: Deps, address: Addr) -> StdResult<HasMintedResponse> {
        let value = self
            .claim_map
            .load(deps.storage, address)
            .unwrap_or_default();

        Ok(HasMintedResponse { value })
    }

    fn get_active_token_id(
        &self,
        deps: Deps,
        address: Addr,
    ) -> StdResult<GetActiveTokenIdResponse> {
        let mut tokens = self
            .tokens
            .range(deps.storage, None, None, Order::Descending);

        let token = tokens.find(|result| {
            if let Ok((_, token_info)) = result {
                token_info.owner == address
            } else {
                false
            }
        });

        if token.is_none() {
            return Err(StdError::generic_err("No tokens"));
        }

        Ok(GetActiveTokenIdResponse {
            value: token.unwrap().unwrap().0,
        })
    }

    fn get_token_details_bulk(
        &self,
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<GetTokenDetailsBulkResponse<T>> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

        let tokens: StdResult<Vec<(String, TokenInfo<T>)>> = self
            .tokens
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(k, v)| (k, v)))
            .collect();

        Ok(GetTokenDetailsBulkResponse {
            tokens: tokens.unwrap(),
        })
    }

    fn get_tokens_for_owner(
        &self,
        deps: Deps,
        address: Addr,
    ) -> StdResult<GetTokensForOwnerResponse> {
        let tokens = self
            .tokens
            .range(deps.storage, None, None, Order::Ascending);

        let result: Vec<String> = tokens
            .filter_map(|result| {
                if let Ok((token_id, token_info)) = result {
                    if token_info.owner == address {
                        Some(token_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(GetTokensForOwnerResponse { tokens: result })
    }

    fn address_has_role(
        &self,
        deps: Deps,
        address: Addr,
        role: Role,
    ) -> StdResult<HasRoleResponse> {
        let value = self.has_role(deps.storage, &address, role)?;
        Ok(HasRoleResponse { value })
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn query(&self, deps: Deps, env: Env, msg: QueryMsg<Q>) -> StdResult<Binary> {
        match msg {
            QueryMsg::ContractInfo {} => to_json_binary(&self.contract_info(deps)?),
            QueryMsg::NftInfo { token_id } => to_json_binary(&self.nft_info(deps, token_id)?),
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => to_json_binary(&self.owner_of(
                deps,
                env,
                token_id,
                include_expired.unwrap_or(false),
            )?),
            QueryMsg::IsOpenMint {} => to_json_binary(&self.is_open_mint(deps)?),
            QueryMsg::IsTradable {} => to_json_binary(&self.is_tradable(deps)?),
            QueryMsg::IsSingleMint {} => to_json_binary(&self.is_single_mint(deps)?),
            QueryMsg::Creator {} => to_json_binary(&self.get_creator(deps)?),
            QueryMsg::HasMinted { address } => to_json_binary(&self.has_minted(deps, address)?),
            QueryMsg::GetActiveTokenId { address } => {
                to_json_binary(&self.get_active_token_id(deps, address)?)
            }
            QueryMsg::GetTokenDetailsBulk { start_after, limit } => {
                to_json_binary(&self.get_token_details_bulk(deps, start_after, limit)?)
            }
            QueryMsg::GetTokensForOwner { address } => {
                to_json_binary(&self.get_tokens_for_owner(deps, address)?)
            }
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => to_json_binary(&self.all_nft_info(
                deps,
                env,
                token_id,
                include_expired.unwrap_or(false),
            )?),
            QueryMsg::Operator {
                owner,
                operator,
                include_expired,
            } => to_json_binary(&self.operator(
                deps,
                env,
                owner,
                operator,
                include_expired.unwrap_or(false),
            )?),
            QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => to_json_binary(&self.operators(
                deps,
                env,
                owner,
                include_expired.unwrap_or(false),
                start_after,
                limit,
            )?),
            QueryMsg::NumTokens {} => to_json_binary(&self.num_tokens(deps)?),
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => to_json_binary(&self.tokens(deps, owner, start_after, limit)?),
            QueryMsg::AllTokens { start_after, limit } => {
                to_json_binary(&self.all_tokens(deps, start_after, limit)?)
            }
            QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => to_json_binary(&self.approval(
                deps,
                env,
                token_id,
                spender,
                include_expired.unwrap_or(false),
            )?),
            QueryMsg::Approvals {
                token_id,
                include_expired,
            } => to_json_binary(&self.approvals(
                deps,
                env,
                token_id,
                include_expired.unwrap_or(false),
            )?),
            QueryMsg::HasRole { address, role } => {
                to_json_binary(&self.address_has_role(deps, address, role)?)
            }
            QueryMsg::Extension { msg: _ } => Ok(Binary::default()),
        }
    }
}

fn parse_approval(item: StdResult<(Addr, Expiration)>) -> StdResult<cw721::Approval> {
    item.map(|(spender, expires)| cw721::Approval {
        spender: spender.to_string(),
        expires,
    })
}

fn humanize_approvals<T>(
    block: &BlockInfo,
    info: &TokenInfo<T>,
    include_expired: bool,
) -> Vec<cw721::Approval> {
    info.approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}
