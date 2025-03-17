use cosmwasm_std::{Decimal, Env, Storage};
use cw_storage_plus::Item;

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

    for (key, state) in infusions {
        INFUSION.save(
            storage,
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
                    description: state.infused_collection.description.to_string(),
                    symbol: state.infused_collection.symbol,
                    base_uri: state.infused_collection.base_uri.clone(),
                    image: state.infused_collection.image,
                    num_tokens: state.infused_collection.num_tokens,
                    royalty_info: state.infused_collection.royalty_info,
                    start_trading_time: state.infused_collection.start_trading_time,
                    explicit_content: state.infused_collection.explicit_content,
                    external_link: state.infused_collection.external_link,
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

    let newcfg = &Config {
        latest_infusion_id: v020cfg.latest_infusion_id,
        contract_owner: v020cfg.contract_owner,
        owner_fee: Decimal::percent(v020cfg.owner_fee), // turn existing fee into decimal.  100u64 = 100%
        min_creation_fee: v020cfg.min_creation_fee,
        min_infusion_fee: v020cfg.min_infusion_fee,
        max_infusions: v020cfg.max_infusions,
        min_per_bundle: v020cfg.min_per_bundle,
        max_per_bundle: v020cfg.max_per_bundle,
        max_bundles: v020cfg.max_bundles,
        code_id: v020cfg.code_id,
        code_hash: v020cfg.code_hash,
    };

    let v030cfg = CONFIG.may_load(storage)?;
    match v030cfg {
        Some(_) => {
            // do nothing if exists
        }
        None => {
            // save new item to map
            let cfg: Item<Config> = Item::new("cfg");
            cfg.save(storage, &newcfg)?;
        }
    }

    Ok(())
}
