use crate::state::{Role, TokenInfo};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Empty};
use cw721::Expiration;
use schemars::JsonSchema;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,

    pub minter: Addr,
    pub claim_issuer: Addr,

    pub is_open_mint: bool,
    pub is_single_mint: bool,
    pub is_tradable: bool,
}

#[cw_serde]
pub enum ExecuteMsg<T, E> {
    TransferNft {
        recipient: String,
        token_id: String,
    },
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    Revoke {
        spender: String,
        token_id: String,
    },
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    RevokeAll {
        operator: String,
    },

    Mint {
        owner: String,
        token_uri: Option<String>,
        extension: T,
    },

    SetIsTradable {
        value: bool,
    },

    SetIsSingleMint {
        value: bool,
    },

    GrantRole {
        role: Role,
        address: Addr,
    },

    RevokeRole {
        role: Role,
        address: Addr,
    },

    SetIsOpenMint {
        value: bool,
    },

    SetHasMinted {
        address: Addr,
        value: bool,
    },

    Burn {
        token_id: String,
    },

    Extension {
        msg: E,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg<Q: JsonSchema> {
    #[returns(cw721::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(cw721::ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    #[returns(cw721::ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    #[returns(cw721::OperatorResponse)]
    Operator {
        owner: String,
        operator: String,
        include_expired: Option<bool>,
    },
    #[returns(cw721::OperatorsResponse)]
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(cw721::NumTokensResponse)]
    NumTokens {},

    #[returns(GetActiveTokenIdResponse)]
    GetActiveTokenId { address: Addr },

    #[returns(GetTokensForOwnerResponse)]
    GetTokensForOwner { address: Addr },

    #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},

    #[returns(cw721::NftInfoResponse<Q>)]
    NftInfo { token_id: String },
    #[returns(cw721::AllNftInfoResponse<Q>)]
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },

    #[returns(cw721::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(cw721::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns[GetTokenDetailsBulkResponse<Empty>]]
    GetTokenDetailsBulk {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(IsOpenMintResponse)]
    IsOpenMint {},

    #[returns(IsSingleMintResponse)]
    IsSingleMint {},

    #[returns(IsTradableResponse)]
    IsTradable {},

    #[returns(CreatorResponse)]
    Creator {},

    #[returns(HasMintedResponse)]
    HasMinted { address: Addr },

    #[returns(HasRoleResponse)]
    HasRole { address: Addr, role: Role },

    #[returns(())]
    Extension { msg: Q },
}

#[cw_serde]
pub struct MinterResponse {
    pub minter: Option<String>,
}

#[cw_serde]
pub struct IsOpenMintResponse {
    pub value: bool,
}

#[cw_serde]
pub struct IsSingleMintResponse {
    pub value: bool,
}

#[cw_serde]
pub struct IsTradableResponse {
    pub value: bool,
}

#[cw_serde]
pub struct CreatorResponse {
    pub creator: Addr,
}

#[cw_serde]
pub struct HasMintedResponse {
    pub value: bool,
}

#[cw_serde]
pub struct HasRoleResponse {
    pub value: bool,
}

#[cw_serde]
pub struct GetActiveTokenIdResponse {
    pub value: String,
}

#[cw_serde]
pub struct GetTokensForOwnerResponse {
    pub tokens: Vec<String>,
}

#[cw_serde]
pub struct GetTokenDetailsBulkResponse<T> {
    pub tokens: Vec<(String, TokenInfo<T>)>,
}
