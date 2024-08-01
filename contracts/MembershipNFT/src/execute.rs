use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Approval, Cw721Contract, Role, TokenInfo};
use cosmwasm_std::{Addr, Binary, CustomMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw721::{ContractInfoResponse, Cw721Execute, Cw721ReceiveMsg, Expiration};
use serde::de::DeserializeOwned;
use serde::Serialize;

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
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
        let info = ContractInfoResponse {
            name: msg.name,
            symbol: msg.symbol,
        };
        self.contract_info.save(deps.storage, &info)?;
        self.creator.save(deps.storage, &_info.sender)?;

        self.update_role(deps.storage, &msg.claim_issuer, Role::ClaimIssuer, true)?;
        self.update_role(deps.storage, &_info.sender, Role::ClaimIssuer, true)?;
        self.update_role(deps.storage, &_info.sender, Role::DefaultAdmin, true)?;

        self.update_role(deps.storage, &_info.sender, Role::Minter, true)?;
        self.update_role(deps.storage, &msg.minter, Role::Minter, true)?;

        self.open_mint.save(deps.storage, &msg.is_open_mint)?;
        self.single_mint.save(deps.storage, &msg.is_single_mint)?;
        self.tradable.save(deps.storage, &msg.is_tradable)?;

        Ok(Response::default())
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg<T, E>,
    ) -> Result<Response<C>, ContractError> {
        match msg {
            ExecuteMsg::Mint {
                owner,
                token_uri,
                extension,
            } => self.mint(deps, info, owner, token_uri, extension),
            ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self.approve(deps, env, info, spender, token_id, expires),
            ExecuteMsg::Revoke { spender, token_id } => {
                self.revoke(deps, env, info, spender, token_id)
            }
            ExecuteMsg::ApproveAll { operator, expires } => {
                self.approve_all(deps, env, info, operator, expires)
            }
            ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
            ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self.transfer_nft(deps, env, info, recipient, token_id),
            ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self.send_nft(deps, env, info, contract, token_id, msg),
            ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
            ExecuteMsg::GrantRole { role, address } => self.grant_role(deps, info, address, role),
            ExecuteMsg::RevokeRole { role, address } => self.revoke_role(deps, info, address, role),
            ExecuteMsg::SetIsOpenMint { value } => self.set_open_mint(deps, info, value),
            ExecuteMsg::SetIsTradable { value } => self.set_is_tradable(deps, info, value),
            ExecuteMsg::SetIsSingleMint { value } => self.set_single_mint(deps, info, value),
            ExecuteMsg::SetHasMinted { address, value } => {
                self.set_has_minted(deps, info, address, value)
            }
            ExecuteMsg::Extension { msg: _ } => Ok(Response::default()),
        }
    }
}

impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn mint(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        owner: String,
        token_uri: Option<String>,
        extension: T,
    ) -> Result<Response<C>, ContractError> {
        let address = deps.api.addr_validate(&owner)?;

        if self.has_role(deps.storage, &address, Role::Blacklisted)?
            || self.has_role(deps.storage, &info.sender, Role::Blacklisted)?
        {
            return Err(ContractError::Blacklisted {});
        }

        if !self.has_role(deps.storage, &info.sender, Role::Minter)?
            && !self._is_open_mint(deps.storage)?
        {
            return Err(ContractError::Unauthorized {});
        }

        if self._is_single_mint(deps.storage)?
            && self._has_claimed(deps.storage, address.to_owned())?
        {
            return Err(ContractError::Claimed {});
        }

        let token = TokenInfo {
            owner: address.to_owned(),
            approvals: vec![],
            token_uri,
            extension,
        };

        self.increment_tokens(deps.storage)?;
        let current = self.token_count(deps.storage)?.to_string();

        self.tokens
            .update(deps.storage, &current, |old| match old {
                Some(_) => Err(ContractError::Claimed {}),
                None => Ok(token),
            })?;

        self.claim_map.save(deps.storage, address, &true)?;

        Ok(Response::new()
            .add_attribute("action", "mint")
            .add_attribute("minter", info.sender)
            .add_attribute("owner", owner)
            .add_attribute("token_id", current))
    }

    pub fn grant_role(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
        role: Role,
    ) -> Result<Response<C>, ContractError> {
        if !self.has_role(deps.storage, &info.sender, Role::DefaultAdmin)? {
            return Err(ContractError::Unauthorized {});
        }
        self.update_role(deps.storage, &address, role, true)?;
        Ok(Response::new())
    }

    pub fn revoke_role(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
        role: Role,
    ) -> Result<Response<C>, ContractError> {
        if !self.has_role(deps.storage, &info.sender, Role::DefaultAdmin)? {
            return Err(ContractError::Unauthorized {});
        }
        self.update_role(deps.storage, &address, role, false)?;
        Ok(Response::new())
    }

    pub fn set_open_mint(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        value: bool,
    ) -> Result<Response<C>, ContractError> {
        if !self.has_role(deps.storage, &info.sender, Role::DefaultAdmin)? {
            return Err(ContractError::Unauthorized {});
        }
        self._set_is_open_mint(deps.storage, value)?;
        Ok(Response::new())
    }

    pub fn set_single_mint(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        value: bool,
    ) -> Result<Response<C>, ContractError> {
        if !self.has_role(deps.storage, &info.sender, Role::DefaultAdmin)? {
            return Err(ContractError::Unauthorized {});
        }
        self._set_is_single_mint(deps.storage, value)?;
        Ok(Response::new())
    }

    pub fn set_is_tradable(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        value: bool,
    ) -> Result<Response<C>, ContractError> {
        if !self.has_role(deps.storage, &info.sender, Role::DefaultAdmin)? {
            return Err(ContractError::Unauthorized {});
        }
        self._set_is_tradable(deps.storage, value)?;
        Ok(Response::new())
    }

    pub fn set_has_minted(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        address: Addr,
        value: bool,
    ) -> Result<Response<C>, ContractError> {
        if !self.has_role(deps.storage, &info.sender, Role::DefaultAdmin)? {
            return Err(ContractError::Unauthorized {});
        }
        self.claim_map.save(deps.storage, address, &value)?;
        Ok(Response::new())
    }
}

impl<'a, T, C, E, Q> Cw721Execute<T, C> for Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    type Err = ContractError;

    fn transfer_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

        Ok(Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id))
    }

    fn send_nft(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response<C>, ContractError> {
        // Transfer token
        self._transfer_nft(deps, &env, &info, &contract, &token_id)?;

        let send = Cw721ReceiveMsg {
            sender: info.sender.to_string(),
            token_id: token_id.clone(),
            msg,
        };

        // Send message
        Ok(Response::new()
            .add_message(send.into_cosmos_msg(contract.clone())?)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", info.sender)
            .add_attribute("recipient", contract)
            .add_attribute("token_id", token_id))
    }

    fn approve(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn revoke(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        self._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

        Ok(Response::new()
            .add_attribute("action", "revoke")
            .add_attribute("sender", info.sender)
            .add_attribute("spender", spender)
            .add_attribute("token_id", token_id))
    }

    fn approve_all(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response<C>, ContractError> {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }

        // set the operator for us
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

        Ok(Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn revoke_all(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        operator: String,
    ) -> Result<Response<C>, ContractError> {
        let operator_addr = deps.api.addr_validate(&operator)?;
        self.operators
            .remove(deps.storage, (&info.sender, &operator_addr));

        Ok(Response::new()
            .add_attribute("action", "revoke_all")
            .add_attribute("sender", info.sender)
            .add_attribute("operator", operator))
    }

    fn burn(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: String,
    ) -> Result<Response<C>, ContractError> {
        let token = self.tokens.load(deps.storage, &token_id)?;
        self.check_can_send(deps.as_ref(), &env, &info, &token, true)?;

        self.tokens.remove(deps.storage, &token_id)?;

        Ok(Response::new()
            .add_attribute("action", "burn")
            .add_attribute("sender", info.sender)
            .add_attribute("token_id", token_id))
    }
}

// helpers
impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn _transfer_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: &str,
        token_id: &str,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        // ensure we have permissions
        self.check_can_send(deps.as_ref(), env, info, &token, false)?;
        // set owner and remove existing approvals
        token.owner = deps.api.addr_validate(recipient)?;
        token.approvals = vec![];
        self.tokens.save(deps.storage, token_id, &token)?;
        Ok(token)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn _update_approvals(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: &str,
        token_id: &str,
        add: bool,
        expires: Option<Expiration>,
    ) -> Result<TokenInfo<T>, ContractError> {
        let mut token = self.tokens.load(deps.storage, token_id)?;
        self.check_can_approve(deps.as_ref(), env, info, &token)?;

        let spender_addr = deps.api.addr_validate(spender)?;
        token.approvals.retain(|apr| apr.spender != spender_addr);

        if add {
            let expires = expires.unwrap_or_default();
            if expires.is_expired(&env.block) {
                return Err(ContractError::Expired {});
            }
            let approval = Approval {
                spender: spender_addr,
                expires,
            };
            token.approvals.push(approval);
        }

        self.tokens.save(deps.storage, token_id, &token)?;

        Ok(token)
    }

    pub fn check_can_approve(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
    ) -> Result<(), ContractError> {
        if token.owner == info.sender {
            return Ok(());
        }
        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::NotOwner {})
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::NotOwner {}),
        }
    }

    pub fn check_can_send(
        &self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
        token: &TokenInfo<T>,
        is_burn: bool,
    ) -> Result<(), ContractError> {
        if !is_burn {
            if self.has_role(deps.storage, &info.sender, Role::Blacklisted)?
                || self.has_role(deps.storage, &token.owner, Role::Blacklisted)?
            {
                return Err(ContractError::Blacklisted {});
            }
        }

        if !self._is_tradable(deps.storage)? {
            return Err(ContractError::Souldbound {});
        }
        if token.owner == info.sender {
            return Ok(());
        }

        if token
            .approvals
            .iter()
            .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
        {
            return Ok(());
        }

        let op = self
            .operators
            .may_load(deps.storage, (&token.owner, &info.sender))?;
        match op {
            Some(ex) => {
                if ex.is_expired(&env.block) {
                    Err(ContractError::NotOwner {})
                } else {
                    Ok(())
                }
            }
            None => Err(ContractError::NotOwner {}),
        }
    }
}
