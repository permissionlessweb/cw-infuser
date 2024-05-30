use cosmwasm_std::{Instantiate2AddressError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized.")]
    Unauthorized,
    #[error("Bundle Not Accepted.")]
    BundleNotAccepted,

    #[error("The Message sender has to be the owner of the NFT to prevent hacks")]
    SenderNotOwner {},

    #[error("Contract got an unexpected Reply")]
    UnexpectedReply(),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

