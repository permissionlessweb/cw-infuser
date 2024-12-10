use std::str::FromStr;

use clap::{arg, command, Parser};
use cosmwasm_std::Uint128;
use cw_infuser::msg::{ExecuteMsgFns, QueryMsgFns};
use cw_infuser::state::{Bundle, NFT};
use cw_orch::core::serde_json;
use cw_orch::daemon::TxSender as _;
use cw_orch::prelude::*;
use scripts::infuser::CwInfuser;
use scripts::ELGAFAR_1;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// create infusion json message
    #[arg(short, long)]
    id: u64,
    /// collections
    #[arg(long)]
    collections: String,
    /// collection_ids
    #[arg(long)]
    collection_ids: String,
}

// cargo run --bin infuse -- --id 1 --collections stars18vng693zqjgwd08p3ypzy26h8f7d7yjweahn5hxq2xnuu837emuslfzn5w,stars1pxcrcl2kt30qdjny8ek6fpkffye4xstvypqdgmh5ssr4yrfu8sgs7450ql --collection_ids 4,2
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    // // create infusion msg type from args
    let args = Args::parse();
    let bundle_collections = args.collections;
    let collection_ids = args.collection_ids;
    let collections: Vec<String> = bundle_collections
        .split(',')
        .map(|s| s.to_string())
        .collect();
    let token_id: Vec<Vec<String>> = collection_ids
        .split(',')
        .map(|s| s.split('-').map(|x| x.to_string()).collect())
        .collect();

    let chain = Daemon::builder(ELGAFAR_1).build()?;
    let infuser = CwInfuser::new(chain.clone());

    let mut msgs = vec![];
    let mut bundle = Bundle { nfts: vec![] };
    // approve transfer for each nft being infused
    for (contract_address, id) in collections.iter().zip(token_id.iter()) {
        for token in id {
            let am = sg721::ExecuteMsg::<Empty, Empty>::Approve {
                spender: infuser.addr_str()?,
                token_id: token.to_string(),
                expires: None,
            };

            let approve = chain.execute(&am, &[], &Addr::unchecked(contract_address));
            msgs.push(approve);

            let nfts = NFT {
                addr: Addr::unchecked(contract_address),
                token_id: Uint128::from_str(token)?.u128() as u64,
            };
            bundle.nfts.push(nfts);
        }
    }

    // create infuse msg
    let infuse = infuser.infuse(vec![bundle], args.id)?;
    let infusion = infuser.infusion_by_id(1)?;
    println!("{:#?}", infuse);
    println!("{:#?}", infusion);

    Ok(())
}
