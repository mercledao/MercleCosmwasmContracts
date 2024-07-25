use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub enum ExecuteMsg {
    Increment,
    Decrement,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub count: u128,
}

#[cw_serde]
pub struct CounterRes {
    pub message: u128,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CounterRes)]
    Current {},
}
