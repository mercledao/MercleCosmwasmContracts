mod error;
mod execute;
pub mod helpers;
pub mod msg;
mod query;
pub mod state;

pub use crate::error::ContractError;
pub use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
pub use crate::state::MintWithClaimContract;

pub const CONTRACT_NAME: &str = "MERCLE_MINT_WITH_CLAIM";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod entry {
    use super::*;

    use cosmwasm_std::{
        entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
    };

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let tract = MintWithClaimContract::<Empty>::default();
        tract.instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        let tract = MintWithClaimContract::<Empty>::default();
        tract.execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        let tract = MintWithClaimContract::<Empty>::default();
        tract.query(deps, env, msg)
    }
}
