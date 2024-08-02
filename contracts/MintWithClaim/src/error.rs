use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Signature already used!")]
    Claimed {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not receiver!")]
    NotReceiver {},

    #[error("failure {msg}")]
    ValidationError { msg: String },

    #[error("Verification failed, Duplicate : {is_duplicate} , SignValid :  {is_sign_valid} , Has_Role : {has_role}")]
    VerificationFailure {
        is_duplicate: bool,
        is_sign_valid: bool,
        has_role: bool,
    },
}
