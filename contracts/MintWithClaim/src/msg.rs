use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary};
use schemars::JsonSchema;

#[cw_serde]
pub struct InstantiateMsg {
    pub treasury: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg<Q: JsonSchema> {
    #[returns(VerifyClaimResponse)]
    VerifySign { message: String, signature: Binary },

    #[returns(TestResponse)]
    Test {},
    /// Extension query
    #[returns(())]
    Extension { msg: Q },
}

#[cw_serde]
pub struct VerifyClaimResponse {
    pub value: bool,
}

#[cw_serde]
pub struct TestResponse {
    pub value: bool,
}
