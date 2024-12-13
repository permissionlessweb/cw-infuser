use cosmwasm_std::{Instantiate2AddressError, StdError};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Asset(#[from] cw_asset::AssetError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Fee payment not accepted. Ensure you are sending the correct amount for the fee payment.")]
    FeeNotAccepted,

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("The Message sender has to be the owner of the NFT to prevent hacks")]
    SenderNotOwner {},

    #[error("Bundle Not Accepted.")]
    BundleNotAccepted,

    #[error("Invalid base token URI (must be an IPFS URI)")]
    InvalidBaseTokenURI {},

    #[error("Token id: {token_id} already sold")]
    TokenIdAlreadySold { token_id: u32 },

    #[error("Sold out")]
    SoldOut {},
    
    #[error("Not enough bundles in nft.  Have: {have}. Min: {min}, Max: {max}")]
    NotEnoughNFTsInBundle { have: u64, min: u64, max: u64 },

    #[error("Too many infusions specified. Have: {have}. Min: {min}, Max: {max}")]
    BadBundle { have: u64, min: u64, max: u64 },

    #[error("Too many infusions specified.")]
    TooManyInfusions,

    #[error("The max_bundles being set is greater than possible. Current hard-coded at 5.")]
    MaxBundleError,

    #[error("You are attempting to create more bundles than possible. Current hard-coded limit set at 2")]
    MaxInfusionErrror,

    #[error("Error setting the fee for your infusion.")]
    CreateInfusionFeeError,

    #[error("Unauthorized.")]
    Unauthorized,
}
