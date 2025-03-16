use cosmwasm_std::{Decimal, Env, Storage};

use crate::{
    msg::MigrateMsg,
    state::{
        BundleType, Config, InfusedCollection, InfusionParamState, InfusionState, NFTCollection,
        CONFIG, INFUSION,
    },
    ContractError,
};

/// Migrate all bundles to default BundleType::AllOf
pub fn migrate_infusions_bundle_type(storage: &mut dyn Storage) -> Result<(), ContractError> {
    // let range = INFUSION_ID
    //     .range(storage, None, None, cosmwasm_std::Order::Descending)
    //     .map(|k| {
    //         let (_, b) = k?;
    //         Ok::<(Addr, u64), ContractError>(b)
    //     })
    //     .collect::<Vec<_>>();
    // println!("range: {:#?}", range);

    // Collect all infusion key-value pairs into a vector first
    let infusions: Vec<_> = v020infuse::state::INFUSION
        .range(storage, None, None, cosmwasm_std::Order::Descending)
        .map(|i| i.unwrap())
        .collect();

    // Now iterate over the collected infusions and save new states
    for (key, state) in infusions {
        INFUSION.save(
            storage, // Note: storage is mutable here
            key,
            &InfusionState {
                enabled: true,
                owner: state.payment_recipient.clone(),
                collections: state
                    .collections
                    .into_iter()
                    .map(|c| NFTCollection {
                        addr: c.addr,
                        min_req: c.min_req.clone(),
                        max_req: None,
                        payment_substitute: None,
                    })
                    .collect(),
                infused_collection: InfusedCollection {
                    sg: state.infused_collection.sg,
                    admin: state.infused_collection.admin,
                    name: state.infused_collection.name,
                    description: "".to_string(),
                    symbol: state.infused_collection.symbol,
                    base_uri: state.infused_collection.base_uri.clone(),
                    image: state.infused_collection.base_uri,
                    num_tokens: state.infused_collection.num_tokens,
                    royalty_info: state.infused_collection.extension,
                    start_trading_time: None,
                    explicit_content: None,
                    external_link: None,
                    addr: state.infused_collection.addr,
                },
                infusion_params: InfusionParamState {
                    bundle_type: BundleType::AllOf {},
                    mint_fee: state.infusion_params.mint_fee,
                    params: None,
                },
                payment_recipient: state.payment_recipient,
            },
        )?;
    }

    Ok(())
}

pub fn migrate_contract_owner_fee_type(
    storage: &mut dyn Storage,
    _env: &Env,
    _msg: &MigrateMsg,
) -> Result<(), ContractError> {
    // match with new config type, replacing the owner admin u64 with Decimals
    let v020cfg = v020infuse::state::CONFIG.load(storage)?;
 
    CONFIG.save(
        storage,
        &Config {
            latest_infusion_id: v020cfg.latest_infusion_id,
            contract_owner: v020cfg.admin,
            owner_fee: Decimal::percent(v020cfg.admin_fee), // turn existing fee into decimal.  100u64 = 100%
            min_creation_fee: v020cfg.min_creation_fee,
            min_infusion_fee: v020cfg.min_infusion_fee,
            max_infusions: v020cfg.max_infusions,
            min_per_bundle: v020cfg.min_per_bundle,
            max_per_bundle: v020cfg.max_per_bundle,
            max_bundles: v020cfg.max_bundles,
            code_id: v020cfg.code_id,
            code_hash: v020cfg.code_hash,
        },
    )?;

    Ok(())
}
