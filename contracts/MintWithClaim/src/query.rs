use crate::msg::{Message, QueryMsg};
use crate::state::MintWithClaimContract;
use cosmwasm_std::{to_json_binary, Binary, CustomMsg, Deps, Env, StdError, StdResult};
use sha2::Digest;

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
            QueryMsg::Extension { msg: _ } => Ok(Binary::default()),
        }
    }
}

impl<'a, C, E, Q> MintWithClaimContract<'a, C, E, Q>
where
    E: CustomMsg,
    Q: CustomMsg,
{
    fn verify_claim(&self, deps: Deps, message: Message, signature: Binary) -> StdResult<bool> {
        let claim_json = serde_json::to_string(&message)
            .map_err(|_| StdError::generic_err("Serialization errro!"))?;

        let mut hasher = sha2::Sha256::new();
        hasher.update(claim_json.as_bytes());
        let message_hash = hasher.finalize();

        let is_valid = deps.api.secp256k1_verify(
            &message_hash,
            &signature,
            &message.from.as_bytes().to_vec(),
        )?;

        if !is_valid {
            return Err(StdError::generic_err("Invalid signature"));
        }

        Ok(true)
    }
}
