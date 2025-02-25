use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    
    #[error("User is already on EAP.")]
    AlreadyWhitelisted {},

    #[error("Payment failed. Make sure you sent 1 INJ to the contract.")]
    PaymentError {},

    #[error("Payment failed.")]
    PaymentFailed {},

    #[error("Invalid ref code, please use a valid one or leave it blank.")]
    InvalidRefCode {},
}
