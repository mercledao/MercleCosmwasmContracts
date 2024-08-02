use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Signature already used!")]
    Claimed {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("failure {msg}")]
    ValidationError { msg: String },

    #[error("Verification failed, {is_duplicate} {is_sign_valid} {has_role}")]
    VerificationFailure {
        is_duplicate: bool,
        is_sign_valid: bool,
        has_role: bool,
    },
}
