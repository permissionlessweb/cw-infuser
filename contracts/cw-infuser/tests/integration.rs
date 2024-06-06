use cw_infuser::{
    contract::interface::MyAppInterface,
    msg::{ConfigResponse, CwInfuserExecuteMsgFns, CwInfuserInstantiateMsg, CwInfuserQueryMsgFns},
    state::{
        Bundle, BurnParams, DefaultInfusionParams, InfusedCollection, Infusion, InfusionParams,
        NFTCollection, NFT,
    },
    MY_NAMESPACE,
};

use abstract_app::objects::namespace::Namespace;
use abstract_client::{AbstractClient, Application, Environment};
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
    app: Application<Env, MyAppInterface<Env>>,
    nfts: Option<Addr>,
}

impl TestEnv<MockBech32> {
    /// Set up the test environment with an Account that has the App installed
    fn setup() -> anyhow::Result<TestEnv<MockBech32>> {
        // Create a sender and mock env
        let mock = MockBech32::new("mock");
        let sender = mock.sender();
        let namespace = Namespace::new(MY_NAMESPACE)?;

        // You can set up Abstract with a builder.
        let abs_client = AbstractClient::builder(mock.clone()).build()?;
        // The app supports setting balances for addresses and configuring ANS.
        abs_client.set_balance(sender.clone(), &coins(123, "ucosm"))?;

        // Publish the app
        let publisher = abs_client.publisher_builder(namespace).build()?;
        publisher.publish_app::<MyAppInterface<_>>()?;

        // store cw721
        let cw721 = cw721_contract();
        mock.upload_custom("cw721", cw721)?;
        // instanatiate cw721
        let msg_a = mock.instantiate(
            10,
            &cw721_base::InstantiateMsg {
                name: "test1".to_string(),
                symbol: "TEST1".to_string(),
                minter: sender.to_string(),
            },
            Some("test1-collection"),
            None,
            &[],
        )?;
        let cw721_a = msg_a.instantiated_contract_address()?;

        // mint cw721
        let mint_msg = mock.execute(
            &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Mint {
                token_id: "1".to_string(),
                owner: sender.to_string(),
                token_uri: None,
                extension: None,
            },
            &vec![],
            &cw721_a.clone(),
        )?;

        // create cw-infsion app
        let app = publisher.account().install_app::<MyAppInterface<_>>(
            &CwInfuserInstantiateMsg {
                default_infusion_params: DefaultInfusionParams {
                    min_required: 1,
                    code_id: 10,
                },
            },
            &[],
        )?;

        mock.execute(
            &cw721_base::ExecuteMsg::<Option<Empty>, Empty>::Approve {
                spender: app.address()?.to_string(),
                token_id: "1".to_string(),
                expires: None,
            },
            &vec![],
            &cw721_a.clone(),
        )?;
        app.create_infusion(vec![Infusion {
            collections: vec![NFTCollection {
                addr: cw721_a.clone(),
                admin: None,
                name: "test1".to_string(),
                symbol: "TEST1".to_string(),
            }],
            infused_collection: InfusedCollection {
                addr: Addr::unchecked("test"),
                admin: None,
                name: "test".to_string(),
                symbol: "TEST".to_string(),
            },
            infusion_params: InfusionParams {
                amount_required: 1,
                params: BurnParams {
                    compatible_traits: None,
                },
            },
            infusion_id: 1,
        }])?;

        Ok(TestEnv {
            abs: abs_client,
            app,
            nfts: Some(cw721_a),
        })
    }
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    let env = TestEnv::setup()?;
    let app = env.app;

    let config = app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            infusion_params: DefaultInfusionParams {
                min_required: 1,
                code_id: 10,
            }
        }
    );
    Ok(())
}

#[test]
fn successful_infusion() -> anyhow::Result<()> {
    let env = TestEnv::setup()?;
    let app = env.app;
    let sender = env.abs.sender();

    app.call_as(&sender).infuse(
        vec![Bundle {
            nfts: vec![NFT {
                addr: env.nfts.unwrap(),
                token_id: 1,
            }],
        }],
        0,
    )?;

    Ok(())
}

#[test]
fn balance_added() -> anyhow::Result<()> {
    let env = TestEnv::setup()?;
    let account = env.app.account();

    // You can add balance to your account in test environment
    let add_balance = coins(100, "ucosm");
    account.add_balance(&add_balance)?;
    let balances = account.query_balances()?;

    assert_eq!(balances, add_balance);

    // Or set balance to any other address using cw_orch
    let mock_env = env.abs.environment();
    mock_env.add_balance(&env.app.address()?, add_balance.clone())?;
    let balances = mock_env.query_all_balances(&env.app.address()?)?;

    assert_eq!(balances, add_balance);
    Ok(())
}
