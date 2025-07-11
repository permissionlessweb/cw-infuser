use cosmwasm_std::{Env, Storage};
use cw_infusions::state::{Infusion, InfusionParamState, InfusionState};

use crate::{
    state::{INFUSION, MINT_COUNT},
    ContractError,
};

/// Downgrade
pub fn v050_patch_downgrade(storage: &mut dyn Storage, env: Env) -> Result<(), ContractError> {
    let infusions = cw_infuser_last::state::INFUSION.range(
        storage,
        None,
        None,
        cosmwasm_std::Order::Descending,
    );

    for infusion in infusions {
        let v040 = infusion?;

        let _updating = InfusionState {
            payment_recipient: v040.1.payment_recipient,
            enabled: todo!(),
            owner: todo!(),
            collections: todo!(),
            infused_collection: todo!(),
            infusion_params: InfusionParamState {
                bundle_type: v0,
                mint_fee:v040.1.infusion_params.mint_fee,
                params: v040.1.infusion_params.params,
                wavs_enabled: false,
            },
        };
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw_orch::anyhow;

    use crate::contract::random_token_list;

    #[test]
    fn test_migration() -> anyhow::Result<()> {
        let mut binding = mock_dependencies();
        let mockdeps = binding.as_mut();
        let mut mockenv = mock_env();
        // let infcoladdr1 = Addr::unchecked("cosmos1infuse1");
        // let infcoladdr2 = Addr::unchecked("cosmos1infus2");
        let info = mock_info("sender", &[]);

        // INFUSION
        let token_ids1 = random_token_list(
            &mockenv.clone(),
            info.sender.clone(),
            (1..=666).collect::<Vec<u32>>(),
        )?;
        //  find the existing tokens we have and save them to map
        //find token positions for minted tokens
        // let mut inf_found1 = vec![];
        // let mut inf_found2 = vec![];

        let mut position = 1;

        mockenv.block.height += 1;
        let token_ids2 = random_token_list(
            &mockenv.clone(),
            info.sender.clone(),
            (1..=100).collect::<Vec<u32>>(),
        )?;

        // save keys with their maps, incorrectly
        position = 1;

        super::v050_patch_downgrade(mockdeps.storage, mockenv.clone())?;

        Ok(())
    }
}
