use cosmwasm_std::Storage;

use crate::{
    state::{
        BundleType, InfusedCollection, InfusionParamState, InfusionState, NFTCollection, INFUSION, MINT_COUNT,
    },
    ContractError,
};

/// Migrate all bundles to default BundleType::AllOf
pub fn patch_mint_count_v040(storage: &mut dyn Storage) -> Result<(), ContractError> {
    MINT_COUNT.save(storage, &0)?;
    Ok(())
}
