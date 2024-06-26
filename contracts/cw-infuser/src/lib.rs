pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
mod replies;
pub mod state;

pub use error::CwInfuserError;

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use contract::interface::CwInfuserInterface;

pub const MY_NAMESPACE: &str = "yournamespace";
pub const MY_APP_NAME: &str = "cw-infuser";
pub const MY_APP_ID: &str = const_format::formatcp!("{MY_NAMESPACE}:{MY_APP_NAME}");
