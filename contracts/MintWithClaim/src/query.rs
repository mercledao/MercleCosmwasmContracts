use crate::msg::{HasRoleResponse, Message, QueryMsg, TreasuryResponse, VerifyClaimResponse};
use crate::state::{MintWithClaimContract, Role};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult};

impl<'a, C> MintWithClaimContract<'a, C> {
    pub fn query(&self, deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::VerifySign {
                message,
                recovery_byte,
                signature,
            } => to_json_binary(&self.verify_claim(deps, message, signature, recovery_byte)?),
            QueryMsg::GetTreasury {} => to_json_binary(&self.get_treasury(deps)?),
            QueryMsg::HasRole { address, role } => {
                to_json_binary(&self.address_has_role(deps, address, role)?)
            }
        }
    }
}

impl<'a, C> MintWithClaimContract<'a, C> {
    fn verify_claim(
        &self,
        deps: Deps,
        message: Message,
        signature: Binary,
        recovery_byte: u8,
    ) -> StdResult<VerifyClaimResponse> {
        let (is_duplicate, is_sign_valid, has_role) = self
            .validate_claim(
                deps,
                message.to_owned(),
                signature.to_owned(),
                recovery_byte,
            )
            .unwrap_or_default();

        let value = !is_duplicate && is_sign_valid && has_role;

        Ok(VerifyClaimResponse { value })
    }

    fn get_treasury(&self, deps: Deps) -> StdResult<TreasuryResponse> {
        let value = self.treasury.may_load(deps.storage)?;
        Ok(TreasuryResponse { value })
    }

    fn address_has_role(
        &self,
        deps: Deps,
        address: Addr,
        role: Role,
    ) -> StdResult<HasRoleResponse> {
        let value = self.has_role(deps.storage, &address, role)?;
        Ok(HasRoleResponse { value })
    }
}
