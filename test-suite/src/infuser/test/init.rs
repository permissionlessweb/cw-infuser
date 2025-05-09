use abstract_cw_multi_test::{Contract, IntoAddr};
use cosmwasm_std::{coin, coins, Decimal, Event, HexBinary, Uint128};
use cw_infuser::{
    msg::{ExecuteMsg, ExecuteMsgFns, InstantiateMsg, QueryMsg, QueryMsgFns},
    state::{
        Bundle, BundleType, Config, InfusedCollection, Infusion, InfusionParamState, InfusionState,
        NFTCollection, NFT,
    },
    AnyOfErr, ContractError,
};
use std::{error::Error, str::FromStr};
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

fn v020_infusion() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        v020infuse::contract::execute,
        v020infuse::contract::instantiate,
        v020infuse::contract::query,
    )
    .with_reply(v020infuse::contract::reply)
    .with_migrate(v020infuse::contract::migrate);
    Box::new(contract)
}

// minimal infuser
pub struct InfuserSuite<Chain> {
    pub chain: MockBech32,
    pub infuser: CwInfuser<Chain>,
    pub nfts: Vec<Addr>,
    pub infusion: Infusion,
    pub admin: Addr,
    pub payment_recipient: Addr,
}

impl<Chain: CwEnv> InfuserSuite<Chain> {
    fn default_infused_collection() -> anyhow::Result<InfusedCollection> {
        Ok(InfusedCollection {
            addr: None,
            admin: None,
            description: "test-description".to_string(),
            name: "test-name".to_string(),
            symbol: "TEST".to_string(),
            base_uri: "ipfs://bafybeidhcxcxolehykzlmmfxzcu5tr2bi4p5yaz7a2s6vsdyqkr25ykkku"
                .to_string(),
            num_tokens: 100,
            sg: false,
            royalty_info: None,
            start_trading_time: None,
            explicit_content: None,
            external_link: None,
            image: "ipfs://bafybeidhcxcxolehykzlmmfxzcu5tr2bi4p5yaz7a2s6vsdyqkr25ykkku".to_string(),
        })
    }

    fn default_good_nfts(&self, nfts: &Vec<Addr>) -> anyhow::Result<Vec<NFTCollection>> {
        let good_nfts = vec![
            NFTCollection {
                addr: nfts[0].clone(),
                min_req: 1,
                max_req: None,
                payment_substitute: None,
            },
            NFTCollection {
                addr: nfts[1].clone(),
                min_req: 1,
                max_req: None,
                payment_substitute: None,
            },
        ];
        Ok(good_nfts)
    }

    fn mint_and_approve_helper(
        chain: MockBech32,
        infuser: Addr,
        admin: Addr,
        nft_collection_addrs: Vec<Addr>,
        nft_count: u64,
    ) -> anyhow::Result<()> {
        for i in nft_collection_addrs.clone() {
            // mint 11 nfts?
            for n in 0..nft_count {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Mint {
                        token_id: n.to_string(),
                        owner: chain.sender.to_string(),
                        token_uri: None,
                        extension: None,
                    };
                chain.execute(msg, &[], &i.clone())?;
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: infuser.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                chain.execute(msg, &[], &i.clone())?;
            }
            for n in 11..21 {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Mint {
                        token_id: n.to_string(),
                        owner: admin.to_string(),
                        token_uri: None,
                        extension: None,
                    };
                chain.execute(msg, &[], &i.clone())?;
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: infuser.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                chain.call_as(&admin).execute(msg, &[], &i.clone())?;
            }
        }
        Ok(())
    }

    fn default_nft_instantiate(
        chain: MockBech32,
        cw721_code: u64,
        minter: Addr,
        withdraw: Addr,
    ) -> anyhow::Result<Vec<Addr>> {
        let mut nft_collection_addrs = vec![];

        for i in 0..3 {
            let msg_a = chain.instantiate(
                cw721_code,
                &cw721_base::InstantiateMsg {
                    name: "good-chronic".to_string(),
                    symbol: "CHRONIC-".to_owned() + i.to_string().as_str(),
                    minter: Some(minter.to_string()),
                    withdraw_address: Some(withdraw.to_string()),
                },
                Some("cw721-base-good-chronic"),
                None,
                &[],
            )?;
            let cw721_a = msg_a.instantiated_contract_address()?;
            println!("test nft collection: {:#?},{:#?}", i, cw721_a.to_string());
            nft_collection_addrs.push(cw721_a);
        }
        Ok(nft_collection_addrs)
    }

    fn default_nft_approvals(
        chain: MockBech32,
        nfts: Vec<Addr>,
        infuser: Addr,
        admin: Addr,
    ) -> anyhow::Result<()> {
        // approve nfts for new infusion
        for i in nfts.clone() {
            for n in 0..10 {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: infuser.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                chain.execute(msg, &[], &i.clone())?;
            }
            // mint nfts to contract owner
            for n in 11..21 {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: infuser.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                chain.call_as(&admin).execute(msg, &[], &i.clone())?;
            }
        }
        Ok(())
    }

    fn setup_fee_suite(bundle_type: BundleType) -> anyhow::Result<InfuserSuite<MockBech32>> {
        // setup infuser with admin fees
        let env = InfuserSuite::<MockBech32>::setup()?;
        let payment_recipient = env
            .chain
            .addr_make_with_balance("payment-recipient", vec![])?;

        // create new infusion contract
        env.infuser.instantiate(
            &InstantiateMsg {
                contract_owner: Some(env.admin.to_string()),
                owner_fee: Decimal::from_str("0.1")?,
                min_creation_fee: Some(coin(500u128, "ustars")),
                min_infusion_fee: Some(coin(100u128, "ustars")),
                min_per_bundle: None,
                max_per_bundle: None,
                max_bundles: None,
                max_infusions: None,
                cw721_code_id: 2u64,
            },
            Some(&env.admin.clone()),
            None,
        )?;
        let nft1 = env.nfts[0].clone();
        let nft2 = env.nfts[1].clone();

        let good_nfts = env.default_good_nfts(&vec![nft1, nft2])?;
        InfuserSuite::<MockBech32>::default_nft_approvals(
            env.chain.clone(),
            env.nfts.clone(),
            env.infuser.address()?,
            env.admin.clone(),
        )?;

        let mut infusion_params = InfusionParamState {
            params: None,
            mint_fee: None,
            bundle_type,
        };

        let good_infused = InfuserSuite::<MockBech32>::default_infused_collection()?;
        // ensure fee set is within contract level bounds
        infusion_params.mint_fee = Some(coin(100, "ustars"));

        let infusion = Infusion {
            collections: good_nfts.clone(),
            infused_collection: good_infused.clone(),
            infusion_params: infusion_params,
            payment_recipient: Some(payment_recipient.clone()),
            owner: None,
            description: Some("testewates".to_string()),
        };
        Ok(InfuserSuite {
            chain: env.chain,
            infuser: env.infuser,
            nfts: env.nfts,
            infusion,
            admin: env.admin,
            payment_recipient,
        })
    }

    // setsup the infuser suite by storing, instantiating, and configuring nft collections & the cw-infuser
    fn setup() -> anyhow::Result<InfuserSuite<MockBech32>> {
        let mock = MockBech32::new("mock");
        let sender = mock.sender_addr();
        let admin = mock.addr_make_with_balance("admin", coins(1000000000, "ustars"))?;
        mock.add_balance(&sender, coins(100000000, "ubtsg"))?;
        mock.add_balance(&sender, coins(100000000, "ustars"))?;
        let treasury = mock.addr_make("treasury");

        // latest infusion version
        let infuser = CwInfuser::new(mock.clone());

        // store cw-infuser
        infuser.upload()?;

        // store cw721
        let cw721 = cw721_contract();
        let cw721_code_id = mock.upload_custom("cw721", cw721)?.uploaded_code_id()?;
        let nft_collection_addrs = InfuserSuite::<MockBech32>::default_nft_instantiate(
            mock.clone(),
            cw721_code_id,
            sender.clone(),
            treasury.clone(),
        )?;

        let default_init = InstantiateMsg {
            contract_owner: Some(admin.to_string()),
            max_bundles: None,
            max_infusions: None,
            max_per_bundle: None,
            min_per_bundle: None,
            cw721_code_id,
            owner_fee: Decimal::zero(),
            min_creation_fee: None,
            min_infusion_fee: None,
        };

        // create cw-infsion app
        infuser.instantiate(&default_init, None, None)?;

        // mint
        InfuserSuite::<MockBech32>::mint_and_approve_helper(
            mock.clone(),
            infuser.address()?.clone(),
            admin.clone(),
            nft_collection_addrs.clone(),
            10,
        )?;
        let mut infusion = Infusion {
            collections: vec![],
            infused_collection: InfuserSuite::<MockBech32>::default_infused_collection()?,
            infusion_params: InfusionParamState {
                params: None,
                mint_fee: None,
                bundle_type: BundleType::AllOf {},
            },
            payment_recipient: Some(treasury.clone()),
            owner: Some(admin.clone()),
            description: Some("testewates".to_string()),
        };

        for i in nft_collection_addrs.clone() {
            let nft_collection = NFTCollection {
                addr: i.clone(),
                min_req: 2,
                max_req: None,
                payment_substitute: None,
            };
            infusion.infused_collection.name =
                infusion.infused_collection.name + &i.to_string().to_owned();
            infusion.collections = [nft_collection].to_vec();
            // create infusion
            infuser.execute(
                &ExecuteMsg::CreateInfusion {
                    infusions: vec![infusion.clone()],
                },
                None,
            )?;
            mock.next_block()?;
        }
        Ok(InfuserSuite {
            chain: mock,
            infuser,
            nfts: nft_collection_addrs,
            infusion,
            admin,
            payment_recipient: treasury,
        })
    }
}

#[test]
fn test_successful_install() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup()?;
    let app = env.infuser;

    let config = app.config()?;
    assert_eq!(
        config,
        Config {
            latest_infusion_id: 3,
            contract_owner: env.admin,
            max_infusions: 2u64,
            min_per_bundle: 1u64,
            max_per_bundle: 10u64,
            max_bundles: 5u64,
            code_id: 2,
            code_hash: HexBinary::from_hex(
                "7e961e9369f7a3619b102834beec5bc2463f9008b40de972c91c45e3b300a805"
            )?,
            owner_fee: Decimal::zero(),
            min_creation_fee: None,
            min_infusion_fee: None,
        }
    );
    Ok(())
}

#[test]
fn test_successful_infusion() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup()?;
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
    env.chain.wait_blocks(1)?;

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
    println!("{:#?}", err);
    assert_eq!(
        err.source().unwrap().to_string(),
        ContractError::BundleNotAccepted { have: 3, want: 2 }.to_string()
    );

    // assert queries
    let res = app.infusion_by_id(1)?;
    assert_eq!(res.collections.len(), 1);
    assert_eq!(res.collections[0].min_req, 2);
    assert!(app.is_in_bundle(env.nfts[0].clone(), 1)?);
    assert!(!app.is_in_bundle(
        "mock1oqklo6g7ca7euusre35wuqj4el3hyj8jty84kwln7du5stwwxyns6h6h3f".into_addr(),
        1
    )?);
    Ok(())
}

// Multiple Collections In Bundle
#[test]
fn test_all_of_infuse_multiple_collections_in_bundle() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup()?;
    let app = env.infuser.clone();

    let bad_nfts = vec![
        NFTCollection {
            addr: env.nfts[0].clone(),
            min_req: 2,
            max_req: None,
            payment_substitute: None,
        },
        NFTCollection {
            addr: env.nfts[0].clone(),
            min_req: 4,
            max_req: None,
            payment_substitute: None,
        },
    ];

    let good_nfts = vec![
        NFTCollection {
            addr: env.nfts[0].clone(),
            min_req: 2,
            max_req: None,
            payment_substitute: None,
        },
        NFTCollection {
            addr: env.nfts[1].clone(),
            min_req: 4,
            max_req: None,
            payment_substitute: None,
        },
    ];

    let good_infused = InfuserSuite::<MockBech32>::default_infused_collection()?;
    let good_infusion_params = InfusionParamState {
        mint_fee: None,
        params: None,
        bundle_type: BundleType::AllOf {},
    };

    let mut infusion = Infusion {
        collections: good_nfts.clone(),
        infused_collection: good_infused.clone(),
        infusion_params: good_infusion_params,
        payment_recipient: Some(env.chain.sender),
        owner: None,
        description: Some("testewates".to_string()),
    };

    // cannot provide same nft collection twice
    infusion.collections = bad_nfts;
    let err = app.create_infusion(vec![infusion.clone()]).unwrap_err();
    assert_eq!(
        err.downcast::<ContractError>().unwrap().to_string(),
        ContractError::DuplicateCollectionInInfusion.to_string()
    );
    infusion.collections = good_nfts;

    // create infusion accepting nft collection 1 & 2
    let res = app.create_infusion(vec![infusion.clone()])?;
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
        ContractError::BundleCollectionNotEligilbe {
            col: env.nfts[1].to_string(),
            bun_type: 1,
        }
        .to_string()
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
        ContractError::BundleCollectionNotEligilbe {
            col: env.nfts[0].to_string(),
            bun_type: 1,
        }
        .to_string()
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
        ContractError::BundleCollectionNotEligilbe {
            col: env.nfts[0].to_string(),
            bun_type: 1,
        }
        .to_string()
    );
    // good infusion
    let res = app.infuse(
        vec![
            Bundle {
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
            },
            Bundle {
                nfts: vec![
                    NFT {
                        addr: env.nfts[0].clone(),
                        token_id: 3,
                    },
                    NFT {
                        addr: env.nfts[0].clone(),
                        token_id: 4,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 5,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 6,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 7,
                    },
                    NFT {
                        addr: env.nfts[1].clone(),
                        token_id: 8,
                    },
                ],
            },
        ],
        infusion_id.clone(),
    )?;


    // good infusion
    // app.infuse(vec![], infusion_id.clone())?;

    //try to infuse again immediately
    Ok(())
}

// Correct Trait Requirement Logic

#[test]
fn test_all_of_eligible_nft_collections() -> anyhow::Result<()> {
    // setup infuser with admin fees
    let env = InfuserSuite::<MockBech32>::setup()?;
    let app = env.infuser.clone();
    let not_nft = env.chain.addr_make("mock-nft");

    let bad_nfts = vec![
        NFTCollection {
            addr: env.nfts[0].clone(),
            min_req: 2,
            max_req: None,
            payment_substitute: None,
        },
        NFTCollection {
            addr: not_nft.clone(),
            min_req: 4,
            max_req: None,
            payment_substitute: None,
        },
    ];
    let good_infused = InfuserSuite::<MockBech32>::default_infused_collection()?;
    let good_infusion_params = InfusionParamState {
        mint_fee: None,
        params: None,
        bundle_type: BundleType::AllOf {},
    };

    let infusion = Infusion {
        collections: bad_nfts.clone(),
        infused_collection: good_infused.clone(),
        infusion_params: good_infusion_params,
        payment_recipient: Some(env.chain.sender),
        owner: None,
        description: Some("testewates".to_string()),
    };

    let res = app.create_infusion(vec![infusion.clone()]).unwrap_err();
    assert_eq!(
        res.root().to_string(),
        ContractError::AddrIsNotNFTCol {
            addr: not_nft.to_string()
        }
        .to_string()
    );
    Ok(())
}

// Correct Fees & Destination
#[test]
fn test_correct_fees() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup_fee_suite(BundleType::AllOf {})?;
    let app = env.infuser;

    let nft1 = env.nfts[0].clone();
    let nft2 = env.nfts[1].clone();

    // err when sending less fees than required
    assert_eq!(
        app.execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(499, "ustars")]),
        )
        .unwrap_err()
        .downcast::<ContractError>()
        .unwrap()
        .to_string(),
        ContractError::RequirednfusionFeeError {
            fee: coin(500, "ustars")
        }
        .to_string()
    );

    // err on more than required mint fee set
    assert_eq!(
        app.execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(501, "ustars")]),
        )
        .unwrap_err()
        .downcast::<ContractError>()
        .unwrap()
        .to_string(),
        ContractError::RequirednfusionFeeError {
            fee: coin(500, "ustars")
        }
        .to_string()
    );

    // good infusion creation
    let res = app.execute(
        &ExecuteMsg::CreateInfusion {
            infusions: vec![env.infusion.clone()],
        },
        Some(&[coin(500, "ustars")]),
    )?;

    let infusion_id = Uint128::from_str(&res.event_attr_value("wasm", "infusion-id")?)
        .unwrap()
        .u128() as u64;

    let mut bundle = Bundle {
        nfts: vec![
            NFT {
                addr: nft1.clone(),
                token_id: 1,
            },
            NFT {
                addr: nft2.clone(),
                token_id: 1,
            },
        ],
    };

    // ensure fee is required when infusing
    // wrong amount
    let infuse = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(1, "ustars")]),
        )
        .unwrap_err();
    assert_eq!(
        infuse.downcast::<ContractError>()?.to_string(),
        ContractError::FeeNotAccepted.to_string()
    );

    // wrong token
    let infuse = env
        .chain
        .call_as(&env.admin)
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
    bundle.nfts[0].token_id = 12;
    bundle.nfts[1].token_id = 12;
    let infuse = app.call_as(&env.admin).execute(
        &ExecuteMsg::Infuse {
            infusion_id: infusion_id.clone(),
            bundle: vec![bundle.clone()],
        },
        Some(&[coin(100, "ustars")]),
    );
    assert!(infuse.is_ok());

    bundle.nfts[0].token_id = 13;
    bundle.nfts[1].token_id = 13;
    env.chain.wait_blocks(1)?;

    // filter events to ensure fees goes to the correct place
    let res = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(100, "ustars")]),
        )?
        .events;

    let fee_transfers = res
        .iter()
        .filter(|e| e.ty == "transfer")
        .collect::<Vec<&Event>>();
    println!("fee_transfers: {:#?}", fee_transfers);
    assert_eq!(fee_transfers.len(), 2);

    // for attribute with env.sender as recipient key value
    let admin_payment = fee_transfers
        .clone()
        .into_iter()
        .find(|e| {
            e.attributes
                .iter()
                .any(|a| a.key == "recipient" && a.value == env.admin.to_string())
        })
        .expect("infusion creation fees were not sent to the correct destination. Should have gone to the contract owner")
        .attributes
        .iter()
        .find(|a| a.key == "amount")
        .expect("incorrect amount of tokens were sent to the contract owner for infusion creation fees.");
    assert_eq!(admin_payment.value, "10ustars".to_string());

    // for attribute with env.sender as recipient key value
    let infusion_owner_payment = fee_transfers
        .into_iter()
        .find(|e| {
            e.attributes
                .iter()
                .any(|a| a.key == "recipient" && a.value == env.payment_recipient.to_string())
        })
        .expect("No admin event found")
        .attributes
        .iter()
        .find(|a| a.key == "amount")
        .expect("No amount attribute found");

    assert_eq!(infusion_owner_payment.value, "90ustars".to_string());
    env.chain.wait_blocks(1)?;

    // ensure admin is ommitted from creation fee validation
    app.call_as(&env.admin).execute(
        &ExecuteMsg::CreateInfusion {
            infusions: vec![env.infusion.clone()],
        },
        None,
    )?;

    Ok(())
}

#[test]
fn test_all_of_infusion_fee() -> anyhow::Result<()> {
    // setup infuser with admin fees
    let mut env = InfuserSuite::<MockBech32>::setup_fee_suite(BundleType::AllOf {})?;
    let app = env.infuser;
    let nft1 = env.nfts[0].clone();
    let nft2 = env.nfts[1].clone();
    let nft3 = env.nfts[2].clone();

    // update infusion bundle type

    env.infusion.collections[0].min_req = 2;
    // update substitute payment for infusion being created to 200ustars
    env.infusion.collections[1].payment_substitute = Some(coin(200u128, "ustars"));

    // good infusion creation
    let infusion_id = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )?
        .event_attr_value("wasm", "infusion-id")?;

    let infusion_id = Uint128::from_str(&infusion_id)?.u128() as u64;

    // test bundle setups
    let mut bundle = Bundle { nfts: vec![] };

    // error on incorrect fee payment substitute for anyOf collection.
    bundle.nfts = vec![
        NFT {
            addr: nft1.clone(),
            token_id: 11,
        },
        NFT {
            addr: nft1.clone(),
            token_id: 12,
        },
    ];

    // error on incorrect nft collections
    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(100, "ustars")]),
        )
        .unwrap_err();
    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::PaymentSubstituteNotProvided {
            col: env.nfts[1].to_string(),
            haved: "ustars".into(),
            havea: "0".into(),
            wantd: "ustars".into(),
            wanta: "200".into(),
        }
        .to_string()
    );
    bundle.nfts.push(NFT {
        addr: env.nfts[2].clone(),
        token_id: 11,
    });
    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::NftIsNotEligible {
            col: env.nfts[2].to_string(),
        }
        .to_string()
    );
    bundle.nfts.pop();

    let res = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )?
        .event_attr_values("wasm", "action");
    // println!("event attribute values for infusion: {:#?}", res);

    assert_eq!(res.len(), 3);
    assert_eq!(res[0], "burn");
    assert_eq!(res[1], "burn");
    assert_eq!(res[2], "mint");
    bundle.nfts = vec![
        NFT {
            addr: env.nfts[0].clone(),
            token_id: 13,
        },
        NFT {
            addr: env.nfts[0].clone(),
            token_id: 14,
        },
    ];

    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(50, "ustars")]),
        )
        .unwrap_err();
    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::FeeNotAccepted {}.to_string()
    );

    let res = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )?
        .event_attr_values("wasm", "action");
    // println!("event attribute values for infusion: {:#?}", res);
    assert_eq!(res.len(), 3);
    assert_eq!(res[0], "burn");
    assert_eq!(res[1], "burn");
    assert_eq!(res[2], "mint");

    env.chain.wait_blocks(1)?;
    // good infusion creation
    env.infusion.collections[1].payment_substitute = None;
    let infusion_id = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )?
        .event_attr_value("wasm", "infusion-id")?;

    let infusion_id = Uint128::from_str(&infusion_id)?.u128() as u64;

    let mut bundle = Bundle { nfts: vec![] };

    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::BundleCollectionNotEligilbe {
            bun_type: 1,
            col: env.nfts[0].to_string(),
        }
        .to_string()
    );

    bundle.nfts = vec![
        NFT {
            addr: env.nfts[0].clone(),
            token_id: 15,
        },
        NFT {
            addr: env.nfts[0].clone(),
            token_id: 16,
        },
    ];

    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::BundleCollectionNotEligilbe {
            bun_type: 1,
            col: env.nfts[1].to_string(),
        }
        .to_string()
    );
    bundle.nfts.extend(vec![
        NFT {
            addr: env.nfts[1].clone(),
            token_id: 15,
        },
        NFT {
            addr: env.nfts[2].clone(),
            token_id: 11,
        },
    ]);
    bundle.nfts.pop();
    let res = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )?
        .event_attr_values("wasm", "action");
    // println!("event attribute values for infusion: {:#?}", res);
    assert_eq!(res.len(), 4);
    assert_eq!(res[0], "burn");
    assert_eq!(res[1], "burn");
    assert_eq!(res[2], "burn");
    assert_eq!(res[3], "mint");

    // error on incorrect static mint fee
    Ok(())
}

#[test]
fn test_infusion_fee_any_of() -> anyhow::Result<()> {
    let mut bundle_type = BundleType::AnyOf { addrs: vec![] };
    // setup infuser with admin fees
    let mut env = InfuserSuite::<MockBech32>::setup_fee_suite(bundle_type)?;
    let app = env.infuser;
    let nft1 = env.nfts[0].clone();
    let nft2 = env.nfts[1].clone();
    let nft3 = env.nfts[2].clone();

    // update infusion bundle type
    env.infusion.collections[0].min_req = 2;

    // cannot create w/ empty anyof
    // good infusion creation
    let err = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::AnyOfConfigError {
            err: AnyOfErr::Empty
        }
        .to_string()
    );

    //cannot use not uneligible collection
    bundle_type = BundleType::AnyOf {
        addrs: vec![nft1.clone(), nft3.clone()],
    };
    env.infusion.infusion_params.bundle_type = bundle_type;

    let mut err = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::AnyOfConfigError {
            err: AnyOfErr::Uneligible
        }
        .to_string()
    );

    // using correct one works
    bundle_type = BundleType::AnyOf {
        addrs: vec![nft1.clone(), nft2.clone()],
    };
    env.infusion.infusion_params.bundle_type = bundle_type;

    // set fee substitute for only one of them
    env.infusion.collections[1].payment_substitute = Some(coin(200u128, "ustars"));

    // good infusion creation
    let infusion_id = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )?
        .event_attr_value("wasm", "infusion-id")?;

    let infusion_id = Uint128::from_str(&infusion_id)?.u128() as u64;
    assert_eq!(infusion_id, 1);
    // test bundle setups
    let mut bundle = Bundle { nfts: vec![] };

    // error on incorrect fee payment substitute for anyOf collection.
    bundle.nfts = vec![NFT {
        addr: nft1.clone(),
        token_id: 11,
    }];

    err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(100, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::PaymentSubstituteNotProvided {
            col: nft2.to_string(),
            haved: "ustars".into(),
            havea: "0".into(),
            wantd: "ustars".into(),
            wanta: "200".into()
        }
        .to_string()
    );
    bundle.nfts.extend(vec![NFT {
        addr: nft1.clone(),
        token_id: 14,
    }]);
    // erro on correct feesub for anyOf, incorrect amount for another anyOf
    err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(150, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::PaymentSubstituteNotProvided {
            col: nft2.to_string(),
            haved: "ustars".into(),
            havea: "50".into(),
            wantd: "ustars".into(),
            wanta: "200".into()
        }
        .to_string()
    );
    // ensure only 1 nft minted with anyOf satisfied by fee payment
    bundle.nfts = vec![];

    let res = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )?
        .event_attr_values("wasm", "action");
    // println!("event attribute values for infusion: {:#?}", res);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], "mint");
    // ensure nft is not burnt

    //  ensure 2 nfts minted with anyOf satisfied twice (no-fee-substitute)
    bundle.nfts = vec![
        NFT {
            addr: nft1.clone(),
            token_id: 11,
        },
        NFT {
            addr: nft1.clone(),
            token_id: 13,
        },
    ];
    let res = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )?
        .event_attr_values("wasm", "action");
    // println!("{:#?}", res);
    assert_eq!(res.len(), 4);
    assert_eq!(res[2], "mint");
    assert_eq!(res[3], "mint");

    //  ensure 1 nft minted if
    bundle.nfts = vec![];
    let res = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )?
        .event_attr_values("wasm", "action");

    println!("{:#?}", res);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0], "mint");
    assert_eq!(res[1], "mint");
    Ok(())
}

#[test]
fn test_infusion_fee_any_of_blend() -> anyhow::Result<()> {
    Ok(())
}
#[test]
fn test_all_of_payment_substitute() -> anyhow::Result<()> {
    // setup infuser with admin fees
    let mut env = InfuserSuite::<MockBech32>::setup_fee_suite(BundleType::AllOf {})?;
    let app = env.infuser;
    let nft1 = env.nfts[0].clone();
    let nft2 = env.nfts[1].clone();

    env.chain
        .add_balance(&env.admin, vec![coin(2000, "ubtsg")])?;

    // update substitute payment for infusion being created to 200ustars
    env.infusion.collections[1].payment_substitute = Some(coin(200u128, "ustars"));

    // good infusion creation
    let infusion_id = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )?
        .event_attr_value("wasm", "infusion-id")?;

    let infusion_id = Uint128::from_str(&infusion_id)?.u128() as u64;

    let mut bundle = Bundle { nfts: vec![] };

    // 0 bundles
    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![],
            },
            None,
        )
        .unwrap_err();
    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::EmptyBundle {}.to_string()
    );

    // empty, single bundle with no fee payment, and no eligible collection
    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            None,
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::BundleCollectionNotEligilbe {
            col: nft1.to_string(),
            bun_type: 1,
        }
        .to_string()
    );

    //  empty bundle with correct # of non-subpayment col, incorrect fees for subpayment col
    bundle.nfts = vec![NFT {
        addr: nft1.clone(),
        token_id: 11,
    }];

    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(200, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::PaymentSubstituteNotProvided {
            col: nft2.to_string(),
            haved: "ustars".into(),
            havea: 100u64.to_string(),
            wantd: "ustars".into(),
            wanta: 200u64.to_string(),
        }
        .to_string()
    );

    // ensure no token and no replacement errors
    let infuse = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(100, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        infuse.downcast::<ContractError>()?.to_string(),
        ContractError::PaymentSubstituteNotProvided {
            col: nft2.to_string(),
            haved: "ustars".into(),
            havea: "0".into(), // global infusion fee was subtracted
            wantd: "ustars".into(),
            wanta: "200".into()
        }
        .to_string()
    );
    bundle.nfts.extend([NFT {
        addr: nft1.clone(),
        token_id: 12,
    }]);
    // too many non paymentsub collection
    let err = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(200, "ustars")]),
        )
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::BundleNotAccepted { have: 2, want: 1 }.to_string()
    );

    // ensure correct payment substitute but incorrect bundle
    bundle = Bundle { nfts: vec![] };
    let infuse = app
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )
        .unwrap_err();
    assert_eq!(
        infuse.downcast::<ContractError>()?.to_string(),
        ContractError::BundleCollectionNotEligilbe {
            col: nft1.to_string(),
            bun_type: 1,
        }
        .to_string()
    );

    // ensure correct payment substitute but incorrect bundle
    bundle = Bundle {
        nfts: vec![NFT {
            addr: nft1.clone(),
            token_id: 11,
        }],
    };
    let infuse = app
        .call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(100, "ustars"), coin(200, "ubtsg")]),
        )
        .unwrap_err();
    assert_eq!(
        infuse.downcast::<ContractError>()?.to_string(),
        ContractError::PaymentSubstituteNotProvided {
            col: nft2.to_string(),
            haved: "ustars".into(),
            havea: "0".into(), // global infusion fee was subtracted
            wantd: "ustars".into(),
            wanta: "200".into()
        }
        .to_string()
    );

    app.call_as(&env.admin)
        .execute(
            &ExecuteMsg::Infuse {
                infusion_id: infusion_id.clone(),
                bundle: vec![bundle.clone()],
            },
            Some(&[coin(300, "ustars")]),
        )
        .unwrap();

    // check with limits

    Ok(())
}

#[test]
fn test_updating_infusion_bundle_type() -> anyhow::Result<()> {
    // setup infuser with admin fees
    let mut env = InfuserSuite::<MockBech32>::setup_fee_suite(BundleType::AllOf {})?;
    let app = env.infuser;

    // good infusion creation
    let infusion_id = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )?
        .event_attr_value("wasm", "infusion-id")?;
    let infusion_id = Uint128::from_str(&infusion_id)?.u128() as u64;

    //  cannot update bundle type to anyOf with incorrect addr

    let mut bundle_type = BundleType::AnyOf {
        addrs: vec![env.nfts[2].clone(), env.nfts[1].clone()],
    };

    let err = app
        .update_infusion_bundle_type(bundle_type, infusion_id)
        .unwrap_err();

    assert_eq!(
        err.downcast::<ContractError>()?.to_string(),
        ContractError::AnyOfConfigError {
            err: AnyOfErr::Uneligible,
        }
        .to_string()
    );

    bundle_type = BundleType::AnyOf {
        addrs: vec![env.nfts[1].clone()],
    };

    app.update_infusion_bundle_type(bundle_type, infusion_id)?;

    Ok(())
}
#[test]
fn test_updating_infusion_eligible_collections() -> anyhow::Result<()> {
    // setup infuser with admin fees
    let mut env = InfuserSuite::<MockBech32>::setup_fee_suite(BundleType::AllOf {})?;
    let app = env.infuser;

    // update infusion collect params to undesired state
    env.infusion.collections[1].payment_substitute = Some(coin(1u128, "ustars"));

    // good infusion creation
    let infusion_id = app
        .execute(
            &ExecuteMsg::CreateInfusion {
                infusions: vec![env.infusion.clone()],
            },
            Some(&[coin(500, "ustars")]),
        )?
        .event_attr_value("wasm", "infusion-id")?;
    let infusion_id = Uint128::from_str(&infusion_id)?.u128() as u64;
    // update infusion accepted params
    env.infusion.collections[1].max_req = Some(3);
    env.infusion.collections[1].min_req = 2;
    env.infusion.collections[1].payment_substitute = Some(coin(100, "ubtsg"));
    app.update_infusions_eligible_collections(
        infusion_id,
        vec![env.infusion.collections[1].clone()],
        vec![env.infusion.collections[0].clone()],
    )?;

    let infusion = app.infusion_by_id(infusion_id)?;
    assert_eq!(infusion.collections.len(), 1);
    assert_eq!(infusion.collections[0].max_req, Some(3));
    assert_eq!(infusion.collections[0].min_req, 2);
    assert_eq!(
        infusion.collections[0].payment_substitute,
        Some(coin(100, "ubtsg"))
    );

    Ok(())
}

// #[test]
// fn test_migration_v041() -> anyhow::Result<()> {
//     let inf1_token_id = vec!["187", "332", "477", "594", "88"];
//     let inf2_token_id = vec!["487"];

//     // setup nft & infusion simulating current infusions
//     let env = InfuserSuite::<MockBech32>::setup_fee_suite()?;

//     let v020_infusion = v020_infusion();
//     let v020_infusion_code_id = env
//         .chain
//         .upload_custom("v020infusion", v020_infusion)?
//         .uploaded_code_id()?;

//     // instantiate 2 infusion contracts from same code-id
//     let mut infuse_addrs = vec![];
//     for _ in 0..2 {
//         let ia = env
//             .chain
//             .instantiate(
//                 v020_infusion_code_id,
//                 &v020infuse::msg::InstantiateMsg {
//                     contract_owner: Some(env.admin.to_string()),
//                     owner_fee: 10,
//                     min_creation_fee: Some(coin(1, "ustars")),
//                     min_infusion_fee: Some(coin(2, "ustars")),
//                     min_per_bundle: None,
//                     max_per_bundle: None,
//                     max_bundles: None,
//                     max_infusions: None,
//                     cw721_code_id: 2u64,
//                 },
//                 Some("infusion being migrated"),
//                 Some(&env.admin),
//                 &[],
//             )?
//             .instantiated_contract_address()?;

//         infuse_addrs.push(ia);
//     }

//     // configure cw721

//     let good_nfts = env.default_good_nfts(&vec![env.nfts[0].clone(), env.nfts[1].clone()])?;
//     let infused_col = InfuserSuite::<MockBech32>::default_infused_collection()?;
//     // create infusion for both
//     for ia in 0..infuse_addrs.len() {
//         let mut total_supply = 666u64;
//         if ia == 2 {
//             total_supply = 100;
//         }
//         env.chain.execute(
//             &v020infuse::msg::ExecuteMsg::CreateInfusion {
//                 infusions: vec![v020infuse::state::Infusion {
//                     collections: good_nfts
//                         .clone()
//                         .into_iter()
//                         .map(|n| v020infuse::state::NFTCollection {
//                             addr: n.addr,
//                             min_req: n.min_req,
//                             max_req: n.max_req,
//                             payment_substitute: n.payment_substitute,
//                         })
//                         .collect(),
//                     infused_collection: v020infuse::state::InfusedCollection {
//                         sg: infused_col.sg,
//                         addr: None,
//                         admin: infused_col.admin.clone(),
//                         name: infused_col.name.clone(),
//                         symbol: infused_col.symbol.clone(),
//                         base_uri: infused_col.base_uri.clone(),
//                         num_tokens: infused_col.num_tokens.clone(),
//                         royalty_info: infused_col.royalty_info.clone(),
//                         description: infused_col.description.clone(),
//                         image: infused_col.image.clone(),
//                         start_trading_time: infused_col.start_trading_time.clone(),
//                         explicit_content: infused_col.explicit_content.clone(),
//                         external_link: infused_col.external_link.clone(),
//                     },
//                     infusion_params: v020infuse::state::InfusionParams {
//                         mint_fee: Some(coin(100, "ustars")),
//                         params: None,
//                     },
//                     payment_recipient: env.infusion.payment_recipient.clone(),
//                     description: env.infusion.description.clone(),
//                     owner: env.infusion.owner.clone(),
//                 }],
//             },
//             &[coin(1, "ustars".to_string())],
//             &infuse_addrs[ia],
//         )?;

//         InfuserSuite::<MockBech32>::default_nft_approvals(
//             env.chain.clone(),
//             env.nfts.clone(),
//             infuse_addrs[ia ].clone(),
//             env.admin.clone(),
//         )?;
//         // infuse once for both
//         let mut bundle = v020infuse::state::Bundle {
//             nfts: vec![
//                 v020infuse::state::NFT {
//                     addr: env.nfts[0].clone(),
//                     token_id: 8,
//                 },
//                 v020infuse::state::NFT {
//                     addr: env.nfts[1].clone(),
//                     token_id: 9,
//                 },
//             ],
//         };

//         //mint 5 tokens for infusion 1 and 1 token for infusion 2
//         for i in 0..5 {
//             env.chain.execute(
//                 &v020infuse::msg::ExecuteMsg::Infuse {
//                     infusion_id: 1,
//                     bundle: vec![bundle.clone()],
//                 },
//                 &[coin(100u128, "ustars")],
//                 &infuse_addrs[0],
//             )?;
//         }
//         for i in 0..2 {}
//     }
//     Ok(())
// }

// #[test]
// fn test_migration_v030() -> anyhow::Result<()> {
//     // setup infuser with admin fees
//     let env = InfuserSuite::<MockBech32>::setup_fee_suite()?;

//     // store cw721
//     let v020_infusion = v020_infusion();
//     let v020_infusion_code_id = env
//         .chain
//         .upload_custom("v020infusion", v020_infusion)?
//         .uploaded_code_id()?;

//     // instantiate 2 infusion contracts from same code-id
//     let mut infuse_addrs = vec![];
//     for _ in 0..2 {
//         let ia = env
//             .chain
//             .instantiate(
//                 v020_infusion_code_id,
//                 &v020infuse::msg::InstantiateMsg {
//                     contract_owner: Some(env.admin.to_string()),
//                     owner_fee: 10,
//                     min_creation_fee: Some(coin(1, "ustars")),
//                     min_infusion_fee: Some(coin(2, "ustars")),
//                     min_per_bundle: None,
//                     max_per_bundle: None,
//                     max_bundles: None,
//                     max_infusions: None,
//                     cw721_code_id: 2u64,
//                 },
//                 Some("infusion being migrated"),
//                 Some(&env.admin),
//                 &[],
//             )?
//             .instantiated_contract_address()?;

//         infuse_addrs.push(ia);
//     }

//     // configure cw721

//     let good_nfts = env.default_good_nfts(&vec![env.nfts[0].clone(), env.nfts[1].clone()])?;

//     let infused_col = InfuserSuite::<MockBech32>::default_infused_collection()?;
//     // create infusion for both
//     for ia in &infuse_addrs {
//         env.chain.execute(
//             &v020infuse::msg::ExecuteMsg::CreateInfusion {
//                 infusions: vec![v020infuse::state::Infusion {
//                     collections: good_nfts
//                         .clone()
//                         .into_iter()
//                         .map(|n| v020infuse::state::NFTCollection {
//                             addr: n.addr,
//                             min_req: n.min_req,
//                             max_req: n.max_req,
//                             payment_substitute: n.payment_substitute,
//                         })
//                         .collect(),
//                     infused_collection: v020infuse::state::InfusedCollection {
//                         sg: infused_col.sg,
//                         addr: None,
//                         admin: infused_col.admin.clone(),
//                         name: infused_col.name.clone(),
//                         symbol: infused_col.symbol.clone(),
//                         base_uri: infused_col.base_uri.clone(),
//                         num_tokens: infused_col.num_tokens.clone(),
//                         royalty_info: infused_col.royalty_info.clone(),
//                         description: infused_col.description.clone(),
//                         image: infused_col.image.clone(),
//                         start_trading_time: infused_col.start_trading_time.clone(),
//                         explicit_content: infused_col.explicit_content.clone(),
//                         external_link: infused_col.external_link.clone(),
//                     },
//                     infusion_params: v020infuse::state::InfusionParams {
//                         mint_fee: Some(coin(100, "ustars")),
//                         params: None,
//                     },
//                     payment_recipient: env.infusion.payment_recipient.clone(),
//                     description: env.infusion.description.clone(),
//                     owner: env.infusion.owner.clone(),
//                 }],
//             },
//             &[coin(1, "ustars".to_string())],
//             &ia,
//         )?;

//         InfuserSuite::<MockBech32>::default_nft_approvals(
//             env.chain.clone(),
//             env.nfts.clone(),
//             ia.clone(),
//             env.admin.clone(),
//         )?;
//     }
//     // infuse once for both
//     let mut bundle = v020infuse::state::Bundle {
//         nfts: vec![
//             v020infuse::state::NFT {
//                 addr: env.nfts[0].clone(),
//                 token_id: 8,
//             },
//             v020infuse::state::NFT {
//                 addr: env.nfts[1].clone(),
//                 token_id: 9,
//             },
//         ],
//     };

//     env.chain.execute(
//         &v020infuse::msg::ExecuteMsg::Infuse {
//             infusion_id: 1,
//             bundle: vec![bundle.clone()],
//         },
//         &[coin(100, "ustars")],
//         &infuse_addrs[0].clone(),
//     )?;
//     bundle.nfts[0].token_id = 7;
//     bundle.nfts[1].token_id = 6;

//     env.chain.wait_blocks(1)?;
//     env.chain.execute(
//         &v020infuse::msg::ExecuteMsg::Infuse {
//             infusion_id: 1,
//             bundle: vec![bundle.clone()],
//         },
//         &[coin(100, "ustars")],
//         &infuse_addrs[0].clone(),
//     )?;

//     env.chain.call_as(&env.admin).migrate(
//         &cw_infuser::msg::MigrateMsg {},
//         env.infuser.code_id()?,
//         &infuse_addrs[0],
//     )?;
//     env.chain.call_as(&env.admin).migrate(
//         &cw_infuser::msg::MigrateMsg {},
//         env.infuser.code_id()?,
//         &infuse_addrs[1],
//     )?;

//     // migrate 1st contract, expect new v030 global config to be saved
//     let cfg1: Config = env.chain.query(&QueryMsg::Config {}, &infuse_addrs[0])?;
//     assert_eq!(cfg1.owner_fee, Decimal::percent(10));
//     // confirm first migration worked
//     let inf: InfusionState = env
//         .chain
//         .query(&QueryMsg::InfusionById { id: 1u64 }, &infuse_addrs[0])?;

//     assert_eq!(inf.infusion_params.bundle_type, BundleType::AllOf {});

//     // migrate 2nd contract,
//     // confirm second migration worked
//     let cfg1: Config = env.chain.query(&QueryMsg::Config {}, &infuse_addrs[1])?;
//     assert_eq!(cfg1.owner_fee, Decimal::percent(10));
//     // confirm first migration worked
//     let inf: InfusionState = env
//         .chain
//         .query(&QueryMsg::InfusionById { id: 1u64 }, &infuse_addrs[1])?;

//     assert_eq!(inf.infusion_params.bundle_type, BundleType::AllOf {});

//     // check can infuse
//     env.chain.wait_blocks(1)?;
//     bundle.nfts[0].token_id = 5;
//     bundle.nfts[1].token_id = 4;

//     env.chain.execute(
//         &v020infuse::msg::ExecuteMsg::Infuse {
//             infusion_id: 1,
//             bundle: vec![bundle.clone()],
//         },
//         &[coin(100, "ustars")],
//         &infuse_addrs[0].clone(),
//     )?;

//     Ok(())
// }

// confirm funds go to destination
