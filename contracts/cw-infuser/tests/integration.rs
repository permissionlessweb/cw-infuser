use cw_infuser::{
    contract::interface::CwInfuserInterface,
    msg::{ConfigResponse, CwInfuserExecuteMsgFns, CwInfuserInstantiateMsg, CwInfuserQueryMsgFns},
    state::{
        Bundle, BurnParams, DefaultInfusionParams, InfusedCollection, Infusion, InfusionParams,
        NFTCollection, NFT,
    },
    MY_NAMESPACE,
};

use abstract_app::objects::namespace::Namespace;
use abstract_client::{AbstractClient, Application};
use abstract_cw_multi_test::Contract;
use cosmwasm_std::coins;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

fn cw721_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

struct TestEnv<Env: CwEnv> {
    abs: AbstractClient<Env>,
    infuser: Application<Env, CwInfuserInterface<Env>>,
    nfts: Vec<Addr>,
}

impl TestEnv<MockBech32> {
    fn setup() -> anyhow::Result<TestEnv<MockBech32>> {
        let mock = MockBech32::new("mock");
        let sender = mock.sender();
        let namespace = Namespace::new(MY_NAMESPACE)?;
        let abs_client = AbstractClient::builder(mock.clone()).build()?;
        abs_client.set_balance(sender.clone(), &coins(123, "ucosm"))?;

        // publish the cw-infuser
        let publisher = abs_client.publisher_builder(namespace).build()?;
        publisher.publish_app::<CwInfuserInterface<_>>()?;

        // store cw721
        let cw721 = cw721_contract();
        mock.upload_custom("cw721", cw721)?;

        // instanatiate cw721
        let msg_a = mock.instantiate(
            10, // hard-coded to cw721
            &cw721_base::InstantiateMsg {
                name: "good-chronic".to_string(),
                symbol: "CHRONIC".to_string(),
                minter: sender.to_string(),
            },
            Some("cw721-base-good-chronic"),
            None,
            &[],
        )?;
        let cw721_a = msg_a.instantiated_contract_address()?;

        // mint 11 nfts?
        for n in 0..10 {
            // mint cw721
            mock.execute(
                &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint {
                    token_id: n.to_string(),
                    owner: sender.to_string(),
                    token_uri: None,
                    extension: None,
                },
                &vec![],
                &cw721_a.clone(),
            )?;
        }

        // create cw-infsion app
        let infuser = publisher.account().install_app::<CwInfuserInterface<_>>(
            &CwInfuserInstantiateMsg {
                default_infusion_params: DefaultInfusionParams {
                    min_required: 2,
                    code_id: 10,
                },
            },
            &[],
        )?;

        for n in 0..10 {
            // approve infuser for nft
            mock.execute(
                &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Approve {
                    spender: infuser.address()?.to_string(),
                    token_id: n.to_string(),
                    expires: None,
                },
                &vec![],
                &cw721_a.clone(),
            )?;
        }

        // create infusion
        infuser.create_infusion(vec![Infusion {
            collections: vec![NFTCollection {
                addr: cw721_a.clone(),
            }],
            infused_collection: InfusedCollection {
                addr: Addr::unchecked("test"),
                admin: None,
                name: "test".to_string(),
                symbol: "TEST".to_string(),
            },
            infusion_params: InfusionParams {
                amount_required: 2,
                params: BurnParams {
                    compatible_traits: None,
                },
            },
            infusion_id: 1,
        }])?;

        Ok(TestEnv {
            abs: abs_client,
            infuser,
            nfts: vec![cw721_a],
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    let env = TestEnv::setup()?;
    let app = env.infuser;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            infusion_params: DefaultInfusionParams {
                min_required: 2,
                code_id: 10,
            }
        }
    );
    Ok(())
}

#[test]
fn successful_infusion() -> anyhow::Result<()> {
    let env = TestEnv::setup()?;
    let app = env.infuser;
    let sender = env.abs.sender();

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
        0,
    )?;
    // error if too few nfts provided in bundle
    let messages = app.infuse(
        vec![Bundle {
            nfts: vec![NFT {
                addr: env.nfts[0].clone(),
                token_id: 2,
            }],
        }],
        0,
    );

    if !messages.is_err() {
        panic!()
    }
    // error if too many nfts provided in bundle
    let messages = app.infuse(
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
        0,
    );

    // assert queries
    let res = app.infusion_by_id(0);
    println!("{:#?}", res);
    Ok(())
}

// Multiple Collections In Bundle 

// Correct Trait Requirement Logic

// Correct Fees & Destination 
