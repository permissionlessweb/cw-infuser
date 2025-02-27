use std::{error::Error, str::FromStr};

use abstract_cw_multi_test::{Contract, IntoAddr};
use cosmwasm_std::{coin, coins, Event, HexBinary, Uint128};
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
    fn setup_fee_suite() -> anyhow::Result<InfuserSuite<MockBech32>> {
        // setup infuser with admin fees
        let env = InfuserSuite::<MockBech32>::setup()?;
        let payment_recipient = env
            .chain
            .addr_make_with_balance("payment-recipient", vec![])?;

        // create new infusion contract
        env.infuser.instantiate(
            &InstantiateMsg {
                contract_owner: Some(env.admin.to_string()),
                owner_fee: 10u64,
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

        let good_nfts = vec![
            NFTCollection {
                addr: nft1.clone(),
                min_req: 1,
                max_req: None,
                payment_substitute: None,
            },
            NFTCollection {
                addr: nft2.clone(),
                min_req: 1,
                max_req: None,
                payment_substitute: None,
            },
        ];

        let mut infusion_params = InfusionParams {
            params: None,
            mint_fee: None,
        };

        let good_infused = InfusedCollection {
            addr: None,
            admin: None,
            name: "test-".to_string(),
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
            description: "eret  jerret skeret".to_string(),
        };
        // ensure fee set is within contract level bounds
        infusion_params.mint_fee = Some(coin(100, "ustars"));

        // approve nfts for new infusion
        for i in env.nfts.clone() {
            // mint 11 nfts?
            for n in 0..10 {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: env.infuser.address()?.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                env.chain.execute(msg, &[], &i.clone())?;
            }
            // mint nfts to contract owner
            for n in 11..21 {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: env.infuser.address()?.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                env.chain
                    .call_as(&env.admin)
                    .execute(msg, &[], &i.clone())?;
            }
        }

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
        let admin = mock.addr_make("admin");
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

        // create 3 infusions
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
            contract_owner: Some(admin.to_string()),
            max_bundles: None,
            max_infusions: None,
            max_per_bundle: None,
            min_per_bundle: None,
            cw721_code_id,
            owner_fee: 0,
            min_creation_fee: None,
            min_infusion_fee: None,
        };
        // create cw-infsion app
        infuser.instantiate(&default_init, None, None)?;

        for i in nft_collection_addrs.clone() {
            // mint 11 nfts?
            for n in 0..10 {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Mint {
                        token_id: n.to_string(),
                        owner: sender.to_string(),
                        token_uri: None,
                        extension: None,
                    };
                mock.execute(msg, &[], &i.clone())?;
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: infuser.address()?.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                mock.execute(msg, &[], &i.clone())?;
            }
            for n in 11..21 {
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Mint {
                        token_id: n.to_string(),
                        owner: admin.to_string(),
                        token_uri: None,
                        extension: None,
                    };
                mock.execute(msg, &[], &i.clone())?;
                let msg: &cw721_base::ExecuteMsg<Option<Empty>, Empty> =
                    &cw721_base::ExecuteMsg::Approve {
                        spender: infuser.address()?.to_string(),
                        token_id: n.to_string(),
                        expires: None,
                    };
                // approve infuser for nft
                mock.call_as(&admin).execute(msg, &[], &i.clone())?;
            }
        }

        let mut infusion = Infusion {
            collections: vec![],
            infused_collection: InfusedCollection {
                addr: None,
                admin: None,
                name: "test-".to_string(),
                symbol: "TEST".to_string(),
                base_uri: "ipfs://bafybeidhcxcxolehykzlmmfxzcu5tr2bi4p5yaz7a2s6vsdyqkr25ykkku"
                    .to_string(),
                num_tokens: 100,
                sg: false,
                royalty_info: None,
                start_trading_time: None,
                explicit_content: None,
                external_link: None,
                image: "ipfs://bafybeidhcxcxolehykzlmmfxzcu5tr2bi4p5yaz7a2s6vsdyqkr25ykkku"
                    .to_string(),
                description: "eret  jerret skeret".to_string(),
            },
            infusion_params: InfusionParams {
                params: None,
                mint_fee: None,
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
fn successful_install() -> anyhow::Result<()> {
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
            owner_fee: 0u64,
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
fn multiple_collections_in_bundle() -> anyhow::Result<()> {
    let env = InfuserSuite::<MockBech32>::setup()?;
    let app = env.infuser;

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
    let good_infusion_params = InfusionParams {
        mint_fee: None,
        params: None,
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
        ContractError::CollectionNotEligible {
            col: env.nfts[1].to_string()
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
        ContractError::CollectionNotEligible {
            col: env.nfts[0].to_string()
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
        ContractError::CollectionNotEligible {
            col: env.nfts[0].to_string()
        }
        .to_string()
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
    let env = InfuserSuite::<MockBech32>::setup_fee_suite()?;
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
fn eligible_contract_is_nft_collection_test() -> anyhow::Result<()> {
    // setup infuser with admin fees
    let env = InfuserSuite::<MockBech32>::setup()?;
    let app = env.infuser;
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
    let good_infusion_params = InfusionParams {
        mint_fee: None,
        params: None,
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

#[test]
fn payment_substitute_tests() -> anyhow::Result<()> {
    // setup infuser with admin fees
    let mut env = InfuserSuite::<MockBech32>::setup_fee_suite()?;
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

    // ensure no token and no replacement errors
    let mut bundle = Bundle {
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
        ContractError::CollectionNotEligible {
            col: nft1.to_string(),
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

// burn during cw721 send
// confirm random token mints
// confirm funds go to destination
