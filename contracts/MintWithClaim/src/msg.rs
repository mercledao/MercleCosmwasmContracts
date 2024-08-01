use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

use crate::state::Role;

#[cw_serde]
pub enum MemberhsipExecute<T> {
    Mint(MembershipMintMsg<T>),
}

#[cw_serde]
pub struct MembershipHasRoleMsg {
    pub address: Addr,
    pub role: Role,
}

#[cw_serde]
pub struct MembershipHasRoleResponse {
    pub value: bool,
}

#[cw_serde]
pub struct MembershipMintMsg<T> {
    pub owner: String,
    pub token_uri: Option<String>,
    pub extension: T,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub treasury: Addr,
}

#[cw_serde]
pub struct Message {
    pub receiver: Addr,
    pub token_uri: String,
    pub fee: Coin,
    pub verifying_contract: Addr,
    pub chain_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetTreasury {
        address: Addr,
    },
    GrantRole {
        role: Role,
        address: Addr,
    },
    RevokeRole {
        role: Role,
        address: Addr,
    },
    MintWithClaim {
        message: Message,
        signature: Binary,
        recovery_byte: u8,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VerifyClaimResponse)]
    VerifySign {
        message: Message,
        signature: Binary,
        recovery_byte: u8,
    },

    #[returns(TreasuryResponse)]
    GetTreasury {},

    #[returns(HasRoleResponse)]
    HasRole { address: Addr, role: Role },
}

#[cw_serde]
pub struct VerifyClaimResponse {
    pub value: Vec<u8>,
    pub hash: String,
}

#[cw_serde]
pub struct HasRoleResponse {
    pub value: bool,
}

#[cw_serde]
pub struct TreasuryResponse {
    pub value: Option<Addr>,
}
