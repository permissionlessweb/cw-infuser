use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OwnershipError(#[from] OwnershipError),

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("{0}")]
    Base(#[from] cw721::error::Cw721ContractError),

    #[error("Minting is paused")]
    MintingPaused {},

    #[error("Minting has not started yet")]
    MintingNotStarted {},

    #[error("Cannot mint more than total supply")]
    CannotMintMoreThanTotal {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("SVG template exceeds max size of {max} bytes (got {got})")]
    SvgTemplateTooLarge { max: usize, got: usize },

    #[error("Total supply {got} exceeds maximum of {max}")]
    TotalSupplyTooHigh { max: u64, got: u64 },

    #[error("Minting period has ended")]
    MintingEnded {},

    #[error("Incorrect payment: expected {expected}")]
    IncorrectPayment { expected: String },

    #[error("Proof hashes required for whitelist verification")]
    MissingProofHashes {},

    #[error("Address {addr} is not whitelisted")]
    NotWhitelisted { addr: String },

    #[error("No whitelist contract configured")]
    NoWhitelistConfigured {},

    #[error("Whitelist per-address mint limit exceeded")]
    MaxPerAddressLimitExceeded {},

    #[error("Invalid variable definition: {reason}")]
    InvalidVariableDef { reason: String },

    #[error("Invalid template placeholder: {reason}")]
    InvalidTemplatePlaceholder { reason: String },
}
