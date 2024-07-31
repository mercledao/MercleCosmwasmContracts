use crate::msg::{QueryMsg, TestResponse, VerifyClaimResponse};
use crate::state::MintWithClaimContract;
use cosmwasm_std::{to_json_binary, Binary, CustomMsg, Deps, Env, StdError, StdResult};
use sha2::Digest;
use sha3::Keccak256;

impl<'a, C, E, Q> MintWithClaimContract<'a, C, E, Q>
where
    E: CustomMsg,
    Q: CustomMsg,
{
    pub fn query(&self, _deps: Deps, _env: Env, msg: QueryMsg<Q>) -> StdResult<Binary> {
        match msg {
            QueryMsg::VerifySign { message, signature } => {
                to_json_binary(&self.verify_claim(_deps, message, signature)?)
            }
            QueryMsg::Test {} => to_json_binary(&self.test()?),
            QueryMsg::Extension { msg: _ } => Ok(Binary::default()),
        }
    }
}

impl<'a, C, E, Q> MintWithClaimContract<'a, C, E, Q>
where
    E: CustomMsg,
    Q: CustomMsg,
{
    fn verify_claim(
        &self,
        deps: Deps,
        message: String,
        signature: Binary,
    ) -> StdResult<VerifyClaimResponse> {
        let message_hash = keccak_256(message.as_bytes());
        let pub_key_result = deps
            .api
            .secp256k1_recover_pubkey(&message_hash, &signature, 0);

        if let Err(_) = pub_key_result {
            return Err(StdError::generic_err("Verfication failed"));
        }
        Ok(VerifyClaimResponse {
            value: pub_key_result?,
        })
    }

    fn test(&self) -> StdResult<TestResponse> {
        Ok(TestResponse { value: true })
    }
}

pub fn keccak_256(data: &[u8]) -> [u8; 32] {
    Keccak256::digest(data).into()
}
