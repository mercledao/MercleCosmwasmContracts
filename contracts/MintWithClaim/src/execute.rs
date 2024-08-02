use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MemberhsipExecute, MembershipMintMsg, Message};
use crate::state::{MintWithClaimContract, Role};
use cosmwasm_std::{
    to_json_binary, Addr, BankMsg, Binary, CosmosMsg, CustomMsg, DepsMut, Empty, Env, MessageInfo,
    Response, StdResult, WasmMsg,
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
            ExecuteMsg::MintWithClaim {
                message,
                signature,
                recovery_byte,
            } => self.mint_with_claim(deps, info, message, signature, recovery_byte),
        }
    }
}
impl<'a, C> MintWithClaimContract<'a, C>
where
    C: CustomMsg,
{
    fn mint_with_claim(
        &self,
        deps: DepsMut,
        _info: MessageInfo,
        message: Message,
        signature: Binary,
        recovery_byte: u8,
    ) -> Result<Response<C>, ContractError> {
        let (is_duplicate, is_sign_valid, has_role) = self
            .validate_claim(
                deps.as_ref(),
                message.to_owned(),
                signature.to_owned(),
                recovery_byte,
            )
            .map_err(|e| ContractError::ValidationError { msg: e.to_string() })?;

        let is_valid = !is_duplicate && is_sign_valid && has_role;

        if !is_valid {
            return Err(ContractError::VerificationFailure {
                has_role,
                is_duplicate,
                is_sign_valid,
            });
        }

        if message.to != _info.sender {
            return Err(ContractError::NotReceiver {});
        }

        let treasury = self.treasury.may_load(deps.storage).unwrap().unwrap();

        let mint_msg = MemberhsipExecute::Mint(MembershipMintMsg::<Empty> {
            owner: message.to.into_string(),
            token_uri: Some(message.token_uri.to_string()),
            extension: Empty {},
        });

        let wasm_msg = WasmMsg::Execute {
            contract_addr: message.verifying_contract.into_string(),
            msg: to_json_binary(&mint_msg).unwrap(),
            funds: vec![],
        };

        let fund_transfer_msg = BankMsg::Send {
            to_address: treasury.into_string(),
            amount: vec![message.fee],
        };

        self.claim_map
            .save(deps.storage, &signature, &true)
            .unwrap();

        Ok(Response::new()
            .add_message(CosmosMsg::Wasm(wasm_msg))
            .add_message(fund_transfer_msg))
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
