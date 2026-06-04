pub mod suite;
use cw_orch::environment::{ChainInfo, ChainKind, NetworkInfo};

pub mod prelude {
    pub use crate::suite::{CwSvgSuite, CwSvgSuiteDeployData};
}
