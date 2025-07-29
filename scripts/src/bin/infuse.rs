use std::str::FromStr;

use clap::{arg, command, Parser};
use cosmwasm_std::Uint128;
use cw721::ApprovalResponse;
// use cw_infusion_minter::msg::{ExecuteMsgFns, QueryMsgFns};

use cw_infuser_scripts::{CwInfuser, ELGAFAR_1};
use cw_infusions::bundles::Bundle;

use cw_infusions::nfts::NFT;
use cw_orch::prelude::*;

/// Simple program to greet a person
// #[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
// struct Args {
//     /// create infusion json message
//     #[arg(short, long)]
//     id: u64,
//     /// collections
//     #[arg(long)]
//     collections: String,
//     /// collection_ids
//     #[arg(long)]
//     collection_ids: String,
// }

// cargo run --bin infuse -- --id 1 --collections stars18vng693zqjgwd08p3ypzy26h8f7d7yjweahn5hxq2xnuu837emuslfzn5w,stars1pxcrcl2kt30qdjny8ek6fpkffye4xstvypqdgmh5ssr4yrfu8sgs7450ql --collection-ids 91-90-89-88,86-58
pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    // // // create infusion msg type from args
    // let args = Args::parse();
    // let bundle_collections = args.collections;
    // let collection_ids = args.collection_ids;
    // let collections: Vec<String> = bundle_collections
    //     .split(',')
    //     .map(|s| s.to_string())
    //     .collect();
    // let token_id: Vec<Vec<String>> = collection_ids
    //     .split(',')
    //     .map(|s| s.split('-').map(|x| x.to_string()).collect())
    //     .collect();

    // let chain = Daemon::builder(ELGAFAR_1).build()?;
    // let infuser = CwInfuser::new(chain.clone());

    // let mut msgs = vec![];
    // let mut bundle = Bundle { nfts: vec![] };
    // // approve transfer for each nft being infused
    // for (contract_address, id) in collections.iter().zip(token_id.iter()) {
    //     for token in id {
    //         let res: Result<ApprovalResponse, _> = chain.wasm_querier().smart_query(
    //             &Addr::unchecked(contract_address.clone()),
    //             &sg721_base::QueryMsg::Approval {
    //                 token_id: token.to_string(),
    //                 spender: infuser.addr_str()?,
    //                 include_expired: None,
    //             },
    //         );
    //         if res.is_err() {
    //             println!("Approval query failed, doesnt exists, creating approval for infuser");
    //             let am: sg721::ExecuteMsg<Empty, Empty> = sg721::ExecuteMsg::Approve {
    //                 spender: infuser.addr_str()?,
    //                 token_id: token.to_string(),
    //                 expires: None,
    //             };

    //             let approve = chain.execute(&am, &[], &Addr::unchecked(contract_address));
    //             msgs.push(approve);
    //         }

    //         let nfts = NFT {
    //             addr: Addr::unchecked(contract_address),
    //             token_id: Uint128::from_str(token)?.u128() as u64,
    //         };
    //         bundle.nfts.push(nfts);
    //     }
    // }

    // // create infuse msg
    // let infuse = infuser.infuse(vec![bundle], args.id)?;
    // let infusion = infuser.infusion_by_id(1)?;
    // println!("{:#?}", infuse);
    // println!("{:#?}", infusion);

    Ok(())
}
