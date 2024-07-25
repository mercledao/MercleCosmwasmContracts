use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::COUNT;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    COUNT.save(deps.storage, &msg.count)?;
    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Current {} => to_json_binary(&query::current(deps)?),
    }
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::Decrement {} => exec::decrement(deps),
        ExecuteMsg::Increment {} => exec::increment(deps),
    }
}

mod exec {
    use super::*;
    use crate::state::COUNT;

    pub fn increment(deps: DepsMut) -> Result<Response, StdError> {
        let count = COUNT.load(deps.storage)?;
        COUNT.save(deps.storage, &(count + 1))?;
        Ok(Response::new())
    }

    pub fn decrement(deps: DepsMut) -> Result<Response, StdError> {
        let count = COUNT.load(deps.storage)?;
        COUNT.save(deps.storage, &(count - 1))?;
        Ok(Response::new())
    }
}

mod query {
    use crate::{msg::CounterRes, state::COUNT};

    use super::*;

    pub fn current(deps: Deps) -> StdResult<CounterRes> {
        let count = COUNT.load(deps.storage)?;
        let resp = CounterRes { message: count };
        Ok(resp)
    }
}
