use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{MintWithClaimContract, Role};
use cosmwasm_std::{CustomMsg, DepsMut, Env, MessageInfo, Response, StdResult};

impl<'a, C, E, Q> MintWithClaimContract<'a, C, E, Q>
where
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response<C>> {
        self.treasury.save(deps.storage, &msg.treasury)?;
        self.update_role(deps.storage, &_info.sender, Role::DefaultAdmin, true)?;
        Ok(Response::default())
    }

    pub fn execute(
        &self,
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response<C>, ContractError> {
        match msg {}
    }
}
