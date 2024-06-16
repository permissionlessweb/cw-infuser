use crate::{
    error::CwInfuserError,
    handlers,
    msg::{CwInfuserExecuteMsg, CwInfuserInstantiateMsg, MyAppMigrateMsg, CwInfuserQueryMsg},
    replies::{self, CREATE_INFUSION_REPLY_ID, INSTANTIATE_REPLY_ID},
    APP_VERSION, MY_APP_ID,
};

use abstract_app::AppContract;
use cosmwasm_std::Response;

/// The type of the result returned by your app's entry points.
pub type CwInfuserResult<T = Response> = Result<T, CwInfuserError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type CwInfuser =
    AppContract<CwInfuserError, CwInfuserInstantiateMsg, CwInfuserExecuteMsg, CwInfuserQueryMsg, MyAppMigrateMsg>;

const APP: CwInfuser = CwInfuser::new(MY_APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_dependencies(&[])
    .with_replies(&[
        (INSTANTIATE_REPLY_ID, replies::instantiate_reply),
        (CREATE_INFUSION_REPLY_ID, replies::create_infusion_reply)
        ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, CwInfuser);

abstract_app::cw_orch_interface!(APP, CwInfuser, CwInfuserInterface);

// TODO: add to docmuentation
// https://linear.app/abstract-sdk/issue/ABS-414/add-documentation-on-dependencycreation-trait
#[cfg(not(target_arch = "wasm32"))]
impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
    for crate::CwInfuserInterface<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;
}
