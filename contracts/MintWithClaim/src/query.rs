use crate::msg::{HasRoleResponse, QueryMsg, TreasuryResponse, VerifyClaimResponse};
use crate::state::{MintWithClaimContract, Role};
use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, Env, StdResult};
use sha2::Digest;
use sha2::Sha256;

impl<'a, C> MintWithClaimContract<'a, C> {
    pub fn query(&self, deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::VerifySign {
                message,
                signature,
                recovery_byte,
            } => to_json_binary(&self.verify_claim(
                deps,
                message.as_slice(),
                signature.as_slice(),
                recovery_byte,
            )?),
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
        message: &[u8],
        signature: &[u8],
        recovery_byte: u8,
    ) -> StdResult<VerifyClaimResponse> {
        let hash = Sha256::digest(message);

        // Verification
        let result = deps
            .api
            .secp256k1_recover_pubkey(hash.as_ref(), signature, recovery_byte);
        match result {
            Ok(pub_key) => Ok(VerifyClaimResponse { value: pub_key }),
            Err(err) => Err(err.into()),
        }
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
