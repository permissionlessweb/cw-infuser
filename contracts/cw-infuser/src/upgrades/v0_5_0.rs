use cosmwasm_std::{coin, Addr, Env, Storage};
use cw_infusions::{
    nfts::{InfusedCollection, RoyaltyInfoResponse},
    state::{EligibleNFTCollection, InfusionParamState, InfusionState},
};

use crate::{state::INFUSION, ContractError};

/// Prep To Add Cosmic Wavs
pub fn save_patch_upgrade(
    storage: &mut dyn Storage,
    env: Env,
    items: Vec<((Addr, u64), InfusionState)>,
) -> Result<(), ContractError> {
    for item in items {
        INFUSION.save(storage, item.0, &item.1)?;
    }
    Ok(())
}
/// Prep To Add Cosmic Wavs
pub fn v050_patch_upgrade(
    storage: &mut dyn Storage,
    env: Env,
) -> Result<Vec<((Addr, u64), InfusionState)>, ContractError> {
    let infusions =
        cw_infuser_v050::state::INFUSION.keys(storage, None, None, cosmwasm_std::Order::Descending);
    let mut keys: Vec<((Addr, u64), InfusionState)> = vec![];

    for infusion in infusions {
        let key = infusion?;
        let v040 = cw_infuser_v050::state::INFUSION.load(storage, key.clone())?;
        keys.push((
            key,
            InfusionState {
                payment_recipient: Addr::unchecked(v040.payment_recipient),
                enabled: v040.enabled,
                owner: v040.owner,
                collections: v040
                    .collections
                    .iter()
                    .map(|col| EligibleNFTCollection {
                        addr: col.addr.clone(),
                        min_req: col.min_req,
                        max_req: col.max_req,
                        payment_substitute: col.payment_substitute.clone(),
                    })
                    .collect(),
                infused_collection: InfusedCollection {
                    sg: v040.infused_collection.sg,
                    admin: v040.infused_collection.admin,
                    name: v040.infused_collection.name,
                    description: v040.infused_collection.description,
                    symbol: v040.infused_collection.symbol,
                    base_uri: v040.infused_collection.base_uri,
                    image: v040.infused_collection.image,
                    num_tokens: v040.infused_collection.num_tokens,
                    royalty_info: match v040.infused_collection.royalty_info {
                        Some(ri) => Some(RoyaltyInfoResponse {
                            payment_address: ri.payment_address,
                            share: ri.share,
                        }),
                        None => None,
                    },
                    start_trading_time: v040.infused_collection.start_trading_time,
                    explicit_content: v040.infused_collection.explicit_content,
                    external_link: v040.infused_collection.external_link,
                    addr: v040.infused_collection.addr,
                },
                infusion_params: InfusionParamState {
                    bundle_type: match v040.infusion_params.bundle_type {
                        cw_infusions_v050::bundles::BundleType::AllOf {} => {
                            cw_infusions::bundles::BundleType::AllOf {}
                        }
                        cw_infusions_v050::bundles::BundleType::AnyOf { addrs } => {
                            cw_infusions::bundles::BundleType::AnyOf {
                                addrs: addrs
                                    .iter()
                                    .map(|addr| Addr::unchecked(addr.to_string()))
                                    .collect(),
                            }
                        }
                        _ => panic!("none exists"),
                    },
                    mint_fee: match v040.infusion_params.mint_fee {
                        Some(c) => Some(coin(c.amount.u128(), c.denom)),
                        None => None,
                    },
                    params: None,
                    wavs_enabled: false,
                },
            },
        ));
    }
    Ok(keys)
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

        // super::v050(mockdeps.storage, mockenv.clone())?;

        Ok(())
    }
}
