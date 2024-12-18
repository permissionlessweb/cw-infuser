use std::{error::Error, str::FromStr};

use abstract_cw_multi_test::{Contract, IntoAddr};
use cosmwasm_std::{coin, coins, HexBinary, Uint128};
use cw_infuser::{
    msg::{ExecuteMsg, ExecuteMsgFns, InstantiateMsg, QueryMsgFns},
    state::{Bundle, Config, InfusedCollection, Infusion, InfusionParams, NFTCollection, NFT},
    ContractError,
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
    // setsup the infuser suite by storing, instantiating, and configuring nft collections & the cw-infuser
    fn setup(init_msg: Option<InstantiateMsg>) -> anyhow::Result<InfuserSuite<MockBech32>> {
        let mock = MockBech32::new("mock");
        let sender = mock.sender_addr();
        mock.add_balance(&sender, coins(100000000, "ubtsg"))?;
        mock.add_balance(&sender, coins(100000000, "ustars"))?;
        let treasury = mock.addr_make("treasury");
        let infuser = CwInfuser::new(mock.clone());

        // store cw-infuser
        infuser.upload()?;

        // store cw721
        let cw721 = cw721_contract();
        let cw721_code_id = mock.upload_custom("cw721", cw721)?.uploaded_code_id()?;

        let mut nft_collection_addrs = vec![];

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
            println!("test nft collection: {:#?},{:#?}", i, cw721_a.to_string());
            nft_collection_addrs.push(cw721_a);
        }

        let default_init = InstantiateMsg {
            admin: Some(sender.to_string()),
            max_bundles: None,
            max_infusions: None,
            max_per_bundle: None,
            min_per_bundle: None,
            cw721_code_id,
            admin_fee: 0,
            min_creation_fee: None,
            min_infusion_fee: None,
        };
        // create cw-infsion app
        infuser.instantiate(&init_msg.clone().unwrap_or(default_init), None, None)?;

        for i in nft_collection_addrs.clone() {
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

        for i in nft_collection_addrs.clone() {
            let mut fee: Vec<Coin> = vec![];
            if let Some(ref i) = init_msg {
                if let Some(mcf) = &i.min_creation_fee {
                    fee.push(mcf.clone());
                }
            }
            // create infusion
            infuser.execute(
                &ExecuteMsg::CreateInfusion {
                    collections: vec![Infusion {
                        collections: vec![NFTCollection {
                            addr: i.clone(),
                            min_req: 2,
                        }],
                        infused_collection: InfusedCollection {
                            addr: None,
                            admin: None,
                            name: "test-".to_string() + &i.to_string().to_owned(),
                            symbol: "TEST".to_string(),
                            base_uri:
                                "ipfs://bafybeidhcxcxolehykzlmmfxzcu5tr2bi4p5yaz7a2s6vsdyqkr25ykkku"
                                    .to_string(),
                            num_tokens: 100,
                            sg: false,
                        },
                        infusion_params: InfusionParams {
                            params: None,
                            mint_fee: None,
                            min_per_bundle: Some(1),
                        },
                        payment_recipient: Some(treasury.clone()),
                    }],
                    payment_recipient: Some(treasury.clone()),
                },
                Some(fee.to_vec().as_slice()),
            )?;
            mock.next_block()?;
        }
        Ok(InfuserSuite {
            chain: mock,
            infuser,
            nfts: nft_collection_addrs,
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup(None)?;
    let app = env.infuser;

    let config = app.config()?;
    assert_eq!(
        config,
        Config {
            latest_infusion_id: 3,
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
    let env = InfuserSuite::<MockBech32>::setup(None)?;
    let app = env.infuser;

    // infuse
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
    // println!("{:#?}", res);

    // confirm infused collection mint
    let res = app.infusion_by_id(1)?;
    assert!(
        res.infused_collection.addr.is_some(),
        "infusion collection not set!"
    );

    // error if too few nfts provided in bundle
    let err = app.infuse(vec![], 1).unwrap_err();

    assert_eq!(err.source().unwrap().to_string(), "Bundle cannot be empty.");

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
    let env = InfuserSuite::<MockBech32>::setup(None)?;
    let app = env.infuser;

    let bad_nfts = vec![
        NFTCollection {
            addr: env.nfts[0].clone(),
            min_req: 2,
        },
        NFTCollection {
            addr: env.nfts[0].clone(),
            min_req: 4,
        },
    ];
    let good_nfts = vec![
        NFTCollection {
            addr: env.nfts[0].clone(),
            min_req: 2,
        },
        NFTCollection {
            addr: env.nfts[1].clone(),
            min_req: 4,
        },
    ];
    let good_infused = InfusedCollection {
        addr: None,
        admin: None,
        name: "test-".to_string(),
        symbol: "TEST".to_string(),
        base_uri: "ipfs://bafybeidhcxcxolehykzlmmfxzcu5tr2bi4p5yaz7a2s6vsdyqkr25ykkku".to_string(),
        num_tokens: 100,
        sg: false,
    };
    let good_infusion_params = InfusionParams {
        params: None,
        mint_fee: None,
        min_per_bundle: Some(1),
    };

    let mut infusion = Infusion {
        collections: good_nfts.clone(),
        infused_collection: good_infused.clone(),
        infusion_params: good_infusion_params,
        payment_recipient: Some(env.chain.sender),
    };

    // cannot provide same nft collection twice
    infusion.collections = bad_nfts;
    let err = app
        .create_infusion(vec![infusion.clone()], None)
        .unwrap_err();
    assert_eq!(
        err.downcast::<ContractError>().unwrap().to_string(),
        ContractError::DuplicateCollectionInInfusion.to_string()
    );
    infusion.collections = good_nfts;
    // create infusion accepting nft collection 1 & 2
    let res = app.create_infusion(vec![infusion.clone()], None)?;
    let infusion_id = Uint128::from_str(&res.event_attr_value("wasm", "infusion-id")?)
        .unwrap()
        .u128() as u64;
    // assert bundle with collection 1 & 3 errors
    // infuse
    assert_eq!(
        app.infuse(
            vec![Bundle {
                nfts: vec![
                    NFT {
                        addr: env.nfts[0].clone(),
                        token_id: 2,
                    },
                    NFT {
                        addr: env.nfts[0].clone(),
                        token_id: 3,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 1,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 2,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 3,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 4,
                    },
                ],
            }],
            infusion_id.clone(),
        )
        .unwrap_err()
        .downcast::<ContractError>()?
        .to_string(),
        ContractError::CollectionNotEligible.to_string()
    );
    // assert bundle with collection 2 & 3 errors
    let err = app
        .infuse(
            vec![Bundle {
                nfts: vec![
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 2,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 8,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 9,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 7,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 6,
                    },
                ],
            }],
            infusion_id.clone(),
        )
        .unwrap_err();
    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::CollectionNotEligible.to_string()
    );
    // assert right number of collections for each bundle

    // assert bundle with collection 2 & 3 errors
    let err = app
        .infuse(
            vec![Bundle {
                nfts: vec![
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 1,
                    },
                    NFT {
                        addr: env.nfts[2].clone(),
                        token_id: 2,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 1,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 2,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 3,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 4,
                    },
                ],
            }],
            infusion_id.clone(),
        )
        .unwrap_err();
    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::CollectionNotEligible.to_string()
    );
    // good infusion
    app.infuse(
        vec![Bundle {
            nfts: vec![
                NFT {
                    addr: env.nfts[0].clone(),
                    token_id: 1,
                },
                NFT {
                    addr: env.nfts[0].clone(),
                    token_id: 2,
                },
                NFT {
                    addr: env.nfts[1].clone(),
                    token_id: 1,
                },
                NFT {
                    addr: env.nfts[1].clone(),
                    token_id: 2,
                },
                NFT {
                    addr: env.nfts[1].clone(),
                    token_id: 3,
                },
                NFT {
                    addr: env.nfts[1].clone(),
                    token_id: 4,
                },
            ],
        }],
        infusion_id.clone(),
    )?;
    Ok(())
}

// Correct Trait Requirement Logic

// Correct Fees & Destination
#[test]
fn correct_fees() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup(Some(InstantiateMsg {
        admin: None,
        admin_fee: 10u64,
        min_creation_fee: Some(coin(500u128, "ubtsg")),
        min_infusion_fee: Some(coin(100u128, "ubtsg")),
        min_per_bundle: None,
        max_per_bundle: None,
        max_bundles: None,
        max_infusions: None,
        cw721_code_id: 2u64,
    }))?;
    let app = env.infuser;
    let admin_addr = env.chain.addr_make("admin");
    let nft1 = env.nfts[0].clone();
    let good_nfts = vec![NFTCollection {
        addr: nft1.clone(),
        min_req: 1,
    }];

    let mut infusion_params = InfusionParams {
        params: None,
        mint_fee: None,
        min_per_bundle: Some(1),
    };

    let good_infused = InfusedCollection {
        addr: None,
        admin: None,
        name: "test-".to_string(),
        symbol: "TEST".to_string(),
        base_uri: "ipfs://bafybeidhcxcxolehykzlmmfxzcu5tr2bi4p5yaz7a2s6vsdyqkr25ykkku".to_string(),
        num_tokens: 100,
        sg: false,
    };
    // ensure fee set is within contract level bounds
    infusion_params.mint_fee = Some(coin(100, "ubtsg"));
    let payment_recipient = env
        .chain
        .addr_make_with_balance("payment-recipient", vec![])?;

    let mut infusion = Infusion {
        collections: good_nfts.clone(),
        infused_collection: good_infused.clone(),
        infusion_params: infusion_params,
        payment_recipient: Some(payment_recipient.clone()),
    };

    // err when sending less fees than required
    assert_eq!(
        app.execute(
            &ExecuteMsg::CreateInfusion {
                collections: vec![infusion.clone()],
                payment_recipient: None,
            },
            Some(&[coin(499, "ubtsg")]),
        )
        .unwrap_err()
        .downcast::<ContractError>()
        .unwrap()
        .to_string(),
        ContractError::RequirednfusionFeeError.to_string()
    );

    // err on more than required mint fee set
    assert_eq!(
        app.execute(
            &ExecuteMsg::CreateInfusion {
                collections: vec![infusion.clone()],
                payment_recipient: None,
            },
            Some(&[coin(501, "ubtsg")]),
        )
        .unwrap_err()
        .downcast::<ContractError>()
        .unwrap()
        .to_string(),
        ContractError::RequirednfusionFeeError.to_string()
    );

    // good infusion creation
    let res = app.execute(
        &ExecuteMsg::CreateInfusion {
            collections: vec![infusion.clone()],
            payment_recipient: None,
        },
        Some(&[coin(500, "ubtsg")]),
    )?;

    // ensure fee gets set to config
    let infusion_id = Uint128::from_str(&res.event_attr_value("wasm", "infusion-id")?)
        .unwrap()
        .u128() as u64;
    let mut bundle = Bundle {
        nfts: vec![NFT {
            addr: nft1.clone(),
            token_id: 1,
        }],
    };

    // ensure fee is required when infusing
    // wrong amount
    let infuse = app
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(1, "ubtsg")]),
        )
        .unwrap_err();
    assert_eq!(
        infuse.downcast::<ContractError>()?.to_string(),
        ContractError::FeeNotAccepted.to_string()
    );

    // wrong token
    let infuse = env
        .chain
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            &[coin(1, "ustars")],
            &app.address()?,
        )
        .unwrap_err();
    assert_eq!(
        infuse.downcast::<ContractError>()?.to_string(),
        ContractError::FeeNotAccepted.to_string()
    );

    // ensure fee is required when infusing
    let infuse = app.execute(
        &ExecuteMsg::Infuse {
            infusion_id: infusion_id.clone(),
            bundle: vec![bundle.clone()],
        },
        Some(&[coin(100, "ubtsg")]),
    );
    assert!(infuse.is_ok());

    bundle.nfts[0].token_id = 2;
    env.chain.wait_blocks(1)?;

    // ensure fees goes to the correct place
    app.execute(
        &ExecuteMsg::Infuse {
            infusion_id: infusion_id.clone(),
            bundle: vec![bundle.clone()],
        },
        Some(&[coin(100, "ubtsg")]),
    )?;
    env.chain.wait_blocks(1)?;

    Ok(())
}

// burn during cw721 send
// confirm random token mints
// confirm funds go to destination
