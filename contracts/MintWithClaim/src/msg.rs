use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};

use crate::state::Role;

#[cw_serde]
pub struct InstantiateMsg {
    pub treasury: Addr,
}

#[cw_serde]
pub struct Message {
    pub nft: Addr,
}

#[cw_serde]
pub enum MemberhsipExecute<T> {
    Mint(MembershipMintMsg<T>),
}

#[cw_serde]
pub struct MembershipMintMsg<T> {
    pub owner: String,
    pub token_uri: Option<String>,
    pub extension: T,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetTreasury { address: Addr },
    GrantRole { role: Role, address: Addr },
    RevokeRole { role: Role, address: Addr },
    MintWithClaim { message: Message },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VerifyClaimResponse)]
    VerifySign {
        message: Binary,
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
}

#[cw_serde]
pub struct HasRoleResponse {
    pub value: bool,
}

#[cw_serde]
pub struct TreasuryResponse {
    pub value: Option<Addr>,
}
