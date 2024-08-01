use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MemberhsipExecute, MembershipMintMsg, Message};
use crate::state::{MintWithClaimContract, Role};
use cosmwasm_std::{
    to_json_binary, Addr, CosmosMsg, CustomMsg, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};

impl<'a, C> MintWithClaimContract<'a, C>
where
    C: CustomMsg,
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
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response<C>, ContractError> {
        match msg {
            ExecuteMsg::SetTreasury { address } => self.set_treasury(deps, info, address),
            ExecuteMsg::GrantRole { role, address } => self.grant_role(deps, info, address, role),
            ExecuteMsg::RevokeRole { role, address } => self.revoke_role(deps, info, address, role),
            ExecuteMsg::MintWithClaim { message } => self.mint_with_claim(deps, info, message),
        }
    }
}
impl<'a, C> MintWithClaimContract<'a, C>
where
    C: CustomMsg,
{
    fn mint_with_claim(
        &self,
        _deps: DepsMut,
        _info: MessageInfo,
        message: Message,
    ) -> Result<Response<C>, ContractError> {
        // let treasury = self.treasury.may_load(deps.storage).unwrap().unwrap();

        let mint_msg = MemberhsipExecute::Mint(MembershipMintMsg::<Empty> {
            owner: _info.sender.into_string(),
            token_uri: Some("TEST".to_string()),
            extension: Empty {},
        });

        let wasm_msg = WasmMsg::Execute {
            contract_addr: message.nft.to_string(),
            msg: to_json_binary(&mint_msg).unwrap(),
            funds: vec![],
        };

        Ok(Response::new().add_message(CosmosMsg::Wasm(wasm_msg)))
    }

    fn set_treasury(
        &self,
        _deps: DepsMut,
        _info: MessageInfo,
        address: Addr,
    ) -> Result<Response<C>, ContractError> {
        if !self
            .has_role(_deps.storage, &_info.sender, Role::DefaultAdmin)
            .unwrap_or_default()
        {
            return Err(ContractError::Unauthorized {});
        }
        self.treasury.save(_deps.storage, &address).unwrap();
        Ok(Response::default())
    }

    pub fn grant_role(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
        role: Role,
    ) -> Result<Response<C>, ContractError> {
        if !self
            .has_role(deps.storage, &info.sender, Role::DefaultAdmin)
            .unwrap_or_default()
        {
            return Err(ContractError::Unauthorized {});
        }
        self.update_role(deps.storage, &address, role, true)
            .unwrap();

        Ok(Response::new())
    }

    pub fn revoke_role(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
        role: Role,
    ) -> Result<Response<C>, ContractError> {
        if !self
            .has_role(deps.storage, &info.sender, Role::DefaultAdmin)
            .unwrap_or_default()
        {
            return Err(ContractError::Unauthorized {});
        }
        self.update_role(deps.storage, &address, role, false)
            .unwrap();

        Ok(Response::new())
    }
}
