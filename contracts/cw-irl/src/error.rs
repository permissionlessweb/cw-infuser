use std::fmt;

use cosmwasm_std::{CheckedMultiplyRatioError, Coin, Instantiate2AddressError, StdError};
use cw_controllers::AdminError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Admin(#[from] AdminError),

    #[error("Sold out")]
    SoldOut {},
 
}

 