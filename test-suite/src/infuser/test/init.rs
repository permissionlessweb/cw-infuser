use std::error::Error;

use abstract_cw_multi_test::{Contract, IntoAddr};
use cosmwasm_std::{coin, coins, HexBinary};
use cw_infuser::{
    msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns},
    state::{Bundle, Config, InfusedCollection, Infusion, InfusionParams, NFTCollection, NFT},
};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};
use scripts::CwInfuser;

fn cw721_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}
// minimal infuser
pub struct InfuserSuite<Chain> {
    pub chain: MockBech32,
    pub infuser: CwInfuser<Chain>,
    pub nfts: Vec<Addr>,
}

impl<Chain: CwEnv> InfuserSuite<Chain> {
    fn setup() -> anyhow::Result<InfuserSuite<MockBech32>> {
        let mock = MockBech32::new("mock");
        let sender = mock.sender_addr();
        mock.add_balance(&sender, coins(100, "ubtsg"))?;
        let treasury = mock.addr_make("treasury");
        let infuser = CwInfuser::new(mock.clone());

        // store cw-infuser
        infuser.upload()?;

        // store cw721
        let cw721 = cw721_contract();
        let cw721_code_id = mock.upload_custom("cw721", cw721)?.uploaded_code_id()?;

        let mut addrs = vec![];
        // create 3 collections
        for i in 0..3 {
            let msg_a = mock.instantiate(
                cw721_code_id,
                &cw721_base::InstantiateMsg {
                    name: "good-chronic".to_string(),
                    symbol: "CHRONIC-".to_owned() + i.to_string().as_str(),
                    minter: Some(sender.to_string()),
                    withdraw_address: Some(treasury.to_string()),
                },
                Some("cw721-base-good-chronic"),
                None,
                &[],
            )?;
            let cw721_a = msg_a.instantiated_contract_address()?;
            addrs.push(cw721_a);
        }

        for i in addrs.clone() {
            // mint 11 nfts?
            for n in 0..10 {
                mock.execute(
                    &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint {
                        token_id: n.to_string(),
                        owner: sender.to_string(),
                        token_uri: None,
                        extension: None,
                    },
                    &[],
                    &i.clone(),
                )?;
            }
        }

        // create cw-infsion app
        infuser.instantiate(
            &InstantiateMsg {
                admin: Some(sender.to_string()),
                max_bundles: None,
                max_infusions: None,
                max_per_bundle: None,
                min_per_bundle: None,
                cw721_code_id,
                admin_fee: 10,
                min_creation_fee: None,
                min_infusion_fee: None,
            },
            None,
            None,
        )?;

        for i in addrs.clone() {
            for n in 0..10 {
                // approve infuser for nft
                mock.execute(
                    &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Approve {
                        spender: infuser.address()?.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    },
                    &[],
                    &i.clone(),
                )?;
            }
        }
        for i in addrs.clone() {
            // create infusion
            infuser.create_infusion(
                vec![Infusion {
                    collections: vec![NFTCollection {
                        addr: i.clone(),
                        min_req: 2,
                    }],
                    infused_collection: InfusedCollection {
                        addr: None,
                        admin: None,
                        name: "test".to_string(),
                        symbol: "TEST".to_string(),
                        base_uri: "ipfs".to_string(),
                    },
                    infusion_params: InfusionParams {
                        params: None,
                        mint_fee: None,
                        min_per_bundle: 1,
                    },
                    payment_recipient: treasury.clone(),
                }],
                None,
            )?;
        }
        Ok(InfuserSuite {
            chain: mock,
            infuser,
            nfts: addrs,
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup()?;
    let app = env.infuser;

    let config = app.config()?;
    assert_eq!(
        config,
        Config {
            latest_infusion_id: 1,
            admin: env.chain.sender_addr(),
            max_infusions: 2u64,
            min_per_bundle: 1u64,
            max_per_bundle: 10u64,
            max_bundles: 5u64,
            code_id: 2,
            code_hash: HexBinary::from_hex(
                "7e961e9369f7a3619b102834beec5bc2463f9008b40de972c91c45e3b300a805"
            )?,
            admin_fee: 10u64,
            min_creation_fee: None,
            min_infusion_fee: None,
        }
    );
    Ok(())
}

#[test]
fn successful_infusion() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup()?;
    let app = env.infuser;

    // create first infusion.
    app.infuse(
        vec![Bundle {
            nfts: vec![
                NFT {
                    addr: env.nfts[0].clone(),
                    token_id: 1,
                },
                NFT {
                    addr: env.nfts[0].clone(),
                    token_id: 3,
                },
            ],
        }],
        1,
    )?;

    // confirm infused collection mint
    let res = app.infusion_by_id(1)?;
    assert_eq!(
        res.infused_collection.addr.unwrap().as_str(),
        "mock1h7fqqvv9enn34w36qselazjvs7exkcw90cl8unnj9zgngkshaktslpljlk"
    );

    // error if too few nfts provided in bundle
    let err = app.infuse(vec![], 1).unwrap_err();

    assert_eq!(err.source().unwrap().to_string(), "Bundle Not Accepted.");

    // error if too many nfts provided in bundle
    let err = app
        .infuse(
            vec![Bundle {
                nfts: vec![
                    NFT {
                        addr: env.nfts[0].clone(),
                        token_id: 2,
                    },
                    NFT {
                        addr: env.nfts[0].clone(),
                        token_id: 4,
                    },
                    NFT {
                        addr: env.nfts[0].clone(),
                        token_id: 6,
                    },
                ],
            }],
            1,
        )
        .unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Not enough bundles in nft.  Have: 3. Min: 2, Max: 2"
    );

    // assert queries
    let res = app.infusion_by_id(1)?;
    assert_eq!(res.collections.len(), 1);
    // assert_eq!(res.collections[0].min_wanted, 2);
    assert!(app.is_in_bundle(env.nfts[0].clone(), 1)?);
    assert!(!app.is_in_bundle(
        "mock1oqklo6g7ca7euusre35wuqj4el3hyj8jty84kwln7du5stwwxyns6h6h3f".into_addr(),
        1
    )?);
    Ok(())
}

// Multiple Collections In Bundle
#[test]
fn multiple_collections_in_bundle() -> anyhow::Result<()> {
    Ok(())
}

// Correct Trait Requirement Logic

// Correct Fees & Destination
#[test]
fn correct_feed() -> anyhow::Result<()> {
    Ok(())
}

// burn during cw721 send
