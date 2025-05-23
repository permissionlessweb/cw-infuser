use clap::{arg, command, Parser};
use cw_infuser::msg::ExecuteMsgFns;
use cw_infusions::bundles::BundleType;
use cw_infusions::nfts::InfusedCollection;
use cw_infusions::state::{EligibleNFTCollection, Infusion, InfusionParamState};
use cw_orch::prelude::*;
use scripts::infuser::CwInfuser;
use scripts::ELGAFAR_1;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// collections eligble. stars1..,stars2...,stars...
    #[arg(long)]
    col_addrs_eligible: String,
    /// min token required. Sort in order collections are defined
    #[arg(long)]
    col_min_required: String,
    /// infused collection name
    #[arg(long)]
    infuse_col_admin: Option<String>,
    /// infused collection name
    #[arg(long)]
    infuse_col_name: String,
    /// infused collection symbol
    #[arg(long)]
    infuse_col_symbol: String,
    /// infused collection title image
    #[arg(long)]
    infuse_col_image: String,
    /// infused collection description
    #[arg(long)]
    infuse_col_description: String,
    /// infused collection base uri
    #[arg(long)]
    infuse_col_base_uri: String,
    /// min num of tokens in collection bundle for all infusions of this contract
    #[arg(long)]
    infuse_col_num_tokens: String,
    /// min num of tokens in collection bundle for all infusions of this contract
    #[arg(long)]
    config_min_per_bundle: String,
    /// optional recipient of infusions. defaults to sender
    #[arg(long)]
    treasury: Option<String>,
}

// cargo run --bin create -- --col_addrs_eligible  --col_min_required --infuse_col_admin --infuse_col_name --infuse_col_symbol --infuse_col_base_uri --config_min_per_bundle
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let args = Args::parse();
    // grab count and each collection addr from args
    let bundle_collections = args.col_addrs_eligible;
    let mint_nft_per_bundle = args.col_min_required;
    let mut infusions: Vec<EligibleNFTCollection> = vec![];
    // create infusion msg type from args
    let collections: Vec<String> = bundle_collections
        .split(',')
        .map(|s| s.to_string())
        .collect();
    let min_required: Vec<String> = mint_nft_per_bundle
        .split(',')
        .map(|s| s.to_string())
        .collect();

    for (collection, min) in collections.iter().zip(min_required.iter()) {
        let addr = Addr::unchecked(collection);
        let min_req: u64 = min.parse().unwrap_or(0);
        let infusion = EligibleNFTCollection {
            addr,
            min_req,
            max_req: None,
            payment_substitute: None,
        };
        infusions.push(infusion);
    }

    let infusion_params = InfusionParamState {
        mint_fee: None,
        params: None,
        bundle_type: BundleType::AllOf {},
        wavs_enabled: true,
    };

    let infused_collection = InfusedCollection {
        addr: None,
        admin: args.infuse_col_admin,
        name: args.infuse_col_name,
        symbol: args.infuse_col_symbol,
        base_uri: args.infuse_col_base_uri,
        num_tokens: args.infuse_col_num_tokens.parse().unwrap(),
        sg: true,
        royalty_info: None,
        start_trading_time: None,
        explicit_content: None,
        external_link: None,
        image: args.infuse_col_image,
        description: "eret".into(),
    };

    // pass infusions to orchestrator
    let chain = Daemon::builder(ELGAFAR_1).build()?;
    let infuser = CwInfuser::new(chain.clone());
    // create infusion
    infuser.create_infusion(vec![Infusion {
        collections: infusions,
        infused_collection,
        infusion_params,
        payment_recipient: Some(chain.sender_addr()),
        owner: None,
        description: Some("todo!()".to_string()),
    }])?;

    Ok(())
}
