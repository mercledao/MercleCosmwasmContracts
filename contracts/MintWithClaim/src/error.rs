use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("Signature already used!")]
    Claimed {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Verification failed")]
    VerificationFailure {},
}
