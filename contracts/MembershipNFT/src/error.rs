use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Version(#[from] cw2::VersionError),

    #[error("Already claimed")]
    Claimed {},

    #[error("No Tokens")]
    NoTokens {},

    #[error("Not owner")]
    NotOwner {},

    #[error("Soulbound Token!")]
    Souldbound {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Blacklisted address")]
    Blacklisted {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },
}
