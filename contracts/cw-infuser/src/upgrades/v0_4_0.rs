use cosmwasm_std::{Env, Storage};

use crate::{
    state::MINT_COUNT,
    ContractError,
};

/// Migrate all bundles to default BundleType::AllOf
pub fn patch_mint_count_v040(storage: &mut dyn Storage) -> Result<(), ContractError> {
    MINT_COUNT.save(storage, &0)?;
    Ok(())
}

///Patch Storage of mint maps
pub fn v0410_remove_mint_count_store(
    storage: &mut dyn Storage,
) -> Result<Vec<Vec<(u32, u32)>>, ContractError> {
    let inf1_token_id = ["187", "332", "477", "594", "88"];
    let inf2_token_id = ["487"];

    // iterate through all token positions.
    // let mtp = v020infuse::state::MINTABLE_TOKEN_POSITIONS.range(
    //     storage,
    //     None,
    //     None,
    //     cosmwasm_std::Order::Ascending,
    // );

    // 1. reset all keys, taking note of keys already minted
    // 2. retain position of used key to prevent double use
    // 3. remove old map from store
    // 4. initialize new maps using prefixes

    let inf1_found = vec![];
    let inf2_found = vec![];

    let count = 0;
    // for kvkey in mtp {
    //     let key = kvkey?;
    //     if inf1_token_id.contains(&key.1.to_string().as_str()) {
    //         inf1_found.push(key);
    //         // we found a key that was used for the first infusion.
    //         // lets reference the map position to save this back to the position it was in.
    //     } else if inf2_token_id.contains(&key.1.to_string().as_str()) {
    //         inf2_found.push(key);
    //     }
    //     count += 1;
    // }

    Ok(vec![inf1_found, inf2_found])
}
///Patch Storage of mint maps
pub fn v0410_add_mint_count_store(
    storage: &mut dyn Storage,
    env: Env,
    position_data: Vec<Vec<(u32, u32)>>,
) -> Result<(), ContractError> {
    // for id in 1..3 {
    //     let infuser = INFUSION_ID.load(storage, id)?;
    //     let infusion = INFUSION.load(storage, infuser)?;
    //     let token_ids = random_token_list(
    //         &env,
    //         Addr::unchecked(infusion.owner),
    //         (1..=infusion.infused_collection.num_tokens).collect::<Vec<u32>>(),
    //     )?;
    //     // omit token id's already minted

    //     let mut vec = MINTABLE_TOKEN_VECTORS.load(storage, id).unwrap_or_default();
    //     for token_id in token_ids {
    //         let mut used: bool = false;
    //         let current_pos = vec.len() as u32;

    //         if id == 1 {
    //             for a in position_data[0].clone() {
    //                 //  if we are at a position in old map we know was used, inject existing value
    //                 if a.0 == current_pos as u32 {
    //                     // println!(
    //                     //     "infusion: {:#?}: setting previous current position value : {:#?} with token-id: {:#?}",
    //                     //   id,  current_pos, a.1
    //                     // );
    //                     if a.1 == 88 && a.0 != 420 {
    //                         if vec.contains(&token_id) {
    //                             panic!("should not exist already")
    //                         }
    //                         vec.push(token_id);
    //                         used = true;
    //                         break;
    //                     }
    //                     // save the known token id to the position it was at
    //                     vec.push(a.1);
    //                     used = true;
    //                     break;
    //                 } else if a.1 == token_id {
    //                     //dont save a used token generate to map. will save if known position is reached
    //                     used = true;
    //                     break;
    //                 }
    //             }
    //             if !used {
    //                 vec.push(token_id);
    //             }
    //         } else if id == 2 {
    //             for a in position_data[1].clone() {
    //                 if a.0 == current_pos as u32 {
    //                     // println!(
    //                     //     "infusion: {:#?}: setting previous current position value : {:#?} with token-id: {:#?}",
    //                     //   id,  current_pos, a.1
    //                     // );
    //                     // save the known token id to the position it was at
    //                     vec.push(a.1);

    //                     used = true;
    //                     break;
    //                 } else if a.1 == token_id {
    //                     used = true;
    //                     break;
    //                 }
    //             }
    //             if !used {
    //                 vec.push(token_id);
    //             }
    //         }
    //     }

    //     MINTABLE_TOKEN_VECTORS.save(storage, id, &vec)?;
    // }
    Ok(())
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw_orch::anyhow;

    use crate::contract::random_token_list;

    #[test]
    fn test_migration() -> anyhow::Result<()> {
        let inf1_token_id = ["187", "332", "477", "594", "88"];
        let inf2_token_id = ["487"];
        let mut binding = mock_dependencies();
        let mockdeps = binding.as_mut();
        let mut mockenv = mock_env();
        // let infcoladdr1 = Addr::unchecked("cosmos1infuse1");
        // let infcoladdr2 = Addr::unchecked("cosmos1infus2");
        let info = mock_info("sender", &[]);

        // INFUSION_ID.save(mockdeps.storage, 1, &(infcoladdr1.clone(), 1))?;
        // INFUSION_ID.save(mockdeps.storage, 2, &(infcoladdr2.clone(), 2))?;

        // let mut infcol1 = InfusedCollection::default();
        // infcol1.num_tokens = 666;
        // let mut infcol2 = InfusedCollection::default();
        // infcol2.num_tokens = 100;
        // INFUSION.save(
        //     mockdeps.storage,
        //     (infcoladdr1, 1),
        //     &InfusionState {
        //         enabled: true,
        //         owner: Addr::unchecked("cosmos1owner"),
        //         collections: vec![],
        //         infused_collection: infcol1,
        //         infusion_params: InfusionParamState::default(),
        //         payment_recipient: Addr::unchecked("cosmos1owner"),
        //     },
        // )?;

        // INFUSION.save(
        //     mockdeps.storage,
        //     (infcoladdr2, 2),
        //     &InfusionState {
        //         enabled: true,
        //         owner: Addr::unchecked("cosmos1owner"),
        //         collections: vec![],
        //         infused_collection: infcol2,
        //         infusion_params: InfusionParamState::default(),
        //         payment_recipient: Addr::unchecked("cosmos1owner"),
        //     },
        // )?;

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
        // for token_id in token_ids1 {
        //     if inf1_token_id.contains(&token_id.to_string().as_str()) {
        //         inf_found1.push((position, token_id));
        //     }
        //     v020infuse::state::MINTABLE_TOKEN_POSITIONS.save(
        //         mockdeps.storage,
        //         position,
        //         &token_id,
        //     )?;
        //     position += 1;
        // }

        mockenv.block.height += 1;
        let token_ids2 = random_token_list(
            &mockenv.clone(),
            info.sender.clone(),
            (1..=100).collect::<Vec<u32>>(),
        )?;

        // save keys with their maps, incorrectly
        position = 1;
        // for token_id in token_ids2 {
        //     if inf2_token_id.contains(&token_id.to_string().as_str()) {
        //         inf_found2.push((position, token_id));
        //     }
        //     v020infuse::state::MINTABLE_TOKEN_POSITIONS.save(
        //         mockdeps.storage,
        //         position,
        //         &token_id,
        //     )?;
        //     position += 1;
        // }

        // println!("inf_found1: {:#?}", inf_found1);
        // println!("inf_found2: {:#?}", inf_found2);

        // run migrations
        let store_data = super::v0410_remove_mint_count_store(mockdeps.storage)?;
        super::v0410_add_mint_count_store(mockdeps.storage, mockenv.clone(), store_data)?;

        // let vecs = MINTABLE_TOKEN_VECTORS.load(mockdeps.storage, 1)?;
        // confirm infusion toke maps retained existing mints, and created new ones

        // println!("~~~~~~~~~~~~~~~~~~~ infusion 1 ~~~~~~~~~~~~~~~~~~~~~~~~");
        // for found in &inf_found1 {
        //     for found in &vecs {
        //         if found > &666 {
        //             panic!("found is not greater than total supply")
        //         } else {
        //             // println!("token ids are not greater than total supply!");
        //         }
        //     }
        //     if !vecs[found.0 as usize] == found.1 {
        //         panic!("not matching")
        //     } else {
        //         println!("token id: {:#?}", found.1);
        //         println!("patched position: {:#?}", found.0);
        //     }
        // }
        // println!("infusion id 1 updated!");

        // // let vecs = MINTABLE_TOKEN_VECTORS.load(mockdeps.storage, 2)?;
        // println!("~~~~~~~~~~~~~~~~~~~ infusion 2 ~~~~~~~~~~~~~~~~~~~~~~~~");
        // if vecs.len() != 100 {
        //     panic!("second infusion must have 100 entries ")
        // } else {
        //     println!("token id map size confirmed!");
        // }
        // for found in &vecs {
        //     if found > &100 {
        //         panic!("token ids are greater than total supply")
        //     } else {
        //         // println!("token ids are not greater than total supply!");
        //     }
        // }
        // println!("infusion id 2 updated!");
        Ok(())
    }
}
