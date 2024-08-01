use crate::msg::Message;
use crate::state::Role;
use bech32::{encode, ToBase32};
use cosmwasm_std::{Addr, Binary, Deps, StdError, StdResult};
use ripemd160::Digest as OtherDigest;
use ripemd160::Ripemd160;
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

pub fn recover_signer(
    deps: Deps,
    message: Message,
    signature: Binary,
    recovery_byte: u8,
) -> StdResult<Addr> {
    let message_hash = serde_json::to_string(&message).unwrap();
    let hash = Sha256::digest(message_hash.clone());

    let result = deps
        .api
        .secp256k1_recover_pubkey(hash.as_ref(), &signature, recovery_byte);
    match result {
        Ok(pub_key_uncompressed) => {
            let pub_key = compress_pubkey(&pub_key_uncompressed)?;

            let sha256_hash = Sha256::digest(&pub_key);
            let ripemd160_hash = Ripemd160::digest(&sha256_hash);

            let address_bytes = ripemd160_hash.to_base32();
            let bech32_addr = encode(&message.bech32_hre, address_bytes)
                .map_err(|err| StdError::generic_err(format!("Bech32 encoding failed: {}", err)))?;
            let address = Addr::unchecked(bech32_addr);
            Ok(address)
        }
        Err(err) => Err(err.into()),
    }
}

// Function to compress a public key
fn compress_pubkey(pub_key: &[u8]) -> StdResult<Vec<u8>> {
    if pub_key.len() != 65 || pub_key[0] != 0x04 {
        return Err(StdError::generic_err(
            "Invalid uncompressed public key format",
        ));
    }

    let mut compressed = vec![0u8; 33];
    compressed[0] = if pub_key[64] % 2 == 0 { 0x02 } else { 0x03 };
    compressed[1..33].copy_from_slice(&pub_key[1..33]);
    Ok(compressed)
}
