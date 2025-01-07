use cosmwasm_std::{Coin, Instantiate2AddressError, StdError};
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

    #[error(
        "Fee payment not accepted. Ensure you are sending the correct amount for the fee payment."
    )]
    FeeNotAccepted,

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),

    #[error("The Message sender has to be the owner of the NFT to prevent hacks")]
    SenderNotOwner {},

    #[error("Cannot specify the same contract address more than once")]
    DuplicateCollectionInInfusion,

    #[error("CollectionNotEligible")]
    CollectionNotEligible,

    #[error("Bundle Not Accepted. Have:{have}. Want: {want}")]
    BundleNotAccepted{ have: u64, want: u64},

    #[error("Bundle cannot be empty.")]
    EmptyBundle,

    #[error("Invalid base token URI (must be an IPFS URI)")]
    InvalidBaseTokenURI {},

    #[error("Token id: {token_id} already sold")]
    TokenIdAlreadySold { token_id: u32 },

    #[error("Sold out")]
    SoldOut {},

    #[error("Too many NFT collections being set for infusion. Have: {have}.  Max: {max}")]
    TooManyCollectionsInInfusion { have: u64, max: u64 },

    #[error("Not enough bundles in nft. Collection: {col} Have: {have}. Min: {min}, Max: {max}")]
    NotEnoughNFTsInBundle {
        col: String,
        have: u64,
        min: u64,
        max: u64,
    },

    #[error("Too many infusions specified. Have: {have}. Min: {min}, Max: {max}")]
    BadBundle { have: u64, min: u64, max: u64 },

    #[error("Too many infusions specified.")]
    TooManyInfusions,

    #[error("The max_bundles being set is greater than possible. Current hard-coded at 5.")]
    MaxBundleError,

    #[error("You are attempting to create more bundles than possible. Current hard-coded limit set at 2")]
    MaxInfusionErrror,

    #[error("New infusion require a minimum fee of {min} to be created.")]
    InfusionFeeLessThanMinimumRequired { min: Coin },

    #[error("RequirednfusionFeeError: New infusion fee required not sent. Retry infusion creation with correct funds.")]
    RequirednfusionFeeError,

    #[error("You cannot set the infusion fee as 0. Omit this value from the create_infsuion message to disable infusion fee requirements.")]
    InfusionFeeCannotbeZero,

    #[error("Cannot set infused collection as sg.")]
    UnauthorizedSg,

    #[error("Unauthorized.")]
    Unauthorized,
}
