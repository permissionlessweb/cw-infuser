use cw_orch::environment::{ChainInfo, ChainKind, NetworkInfo};

pub mod deploy;
pub use deploy::infuser::CwInfuser;

pub const TERPNET: NetworkInfo = NetworkInfo {
    chain_name: "terp network",
    pub_address_prefix: "terp",
    coin_type: 118u32,
};

/// https://github.com/cosmos/chain-registry/blob/master/testnets/stargazetestnet/chain.json
pub const MOROCCO_1: ChainInfo = ChainInfo {
    kind: ChainKind::Mainnet,
    chain_id: "morocco-1",
    gas_denom: "uthiol",
    gas_price: 1.2,
    grpc_urls: &["http://grpc-mainnet.terp.network:443"],
    network_info: TERPNET,
    lcd_url: None,
    fcd_url: None,
};
