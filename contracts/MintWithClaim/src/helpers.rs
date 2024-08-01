use crate::msg::Message;
use crate::{msg::VerifyClaimResponse, state::Role};
use cosmwasm_std::{Binary, Deps, StdResult};
use sha2::Digest;
use sha2::Sha256;

pub fn get_key_for_role<'a>(role: Role) -> &'a str {
    match role {
        Role::DefaultAdmin => "1",
        Role::ClaimIssuer => "2",
        Role::Minter => "3",
        Role::Blacklisted => "4",
    }
}

pub fn verify_claim(
    deps: Deps,
    message: Message,
    signature: Binary,
    recovery_byte: u8,
) -> StdResult<VerifyClaimResponse> {
    let message_hash = serde_json::to_string(&message).unwrap();
    let hash = Sha256::digest(message_hash.clone());

    let result = deps
        .api
        .secp256k1_recover_pubkey(hash.as_ref(), &signature, recovery_byte);
    match result {
        Ok(pub_key) => Ok(VerifyClaimResponse {
            value: pub_key,
            hash: message_hash,
        }),
        Err(err) => Err(err.into()),
    }
}
