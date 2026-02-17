use cosmwasm_std::Binary;
use cw_orch::environment::{ChainInfo, ChainKind, NetworkInfo};
use cw_svg::InstantiateMsg as SvgInitMsg;

pub mod suite;

/// Load a cw721-svg InstantiateMsg from a JSON file.
///
/// The `seed` field can be either:
///   - A base64 string (standard Binary encoding)
///   - A plain UTF-8 string (will be blake3-hashed into 32 bytes)
pub fn load_svg_init_msg(path: &str) -> anyhow::Result<SvgInitMsg> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path, e))?;

    let mut value: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("failed to parse JSON from {}: {}", path, e))?;

    // If `seed` is a string but not valid base64, hash it and replace with base64
    if let Some(seed_val) = value.get("seed") {
        if let Some(seed_str) = seed_val.as_str() {
            if Binary::from_base64(seed_str).is_err() {
                let hashed = blake3::hash(seed_str.as_bytes());
                let b64 = Binary::from(hashed.as_bytes().as_slice()).to_base64();
                value["seed"] = serde_json::Value::String(b64);
            }
        }
    }

    let msg: SvgInitMsg = serde_json::from_value(value).map_err(|e| {
        anyhow::anyhow!("failed to deserialize InstantiateMsg from {}: {}", path, e)
    })?;

    Ok(msg)
}

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

/// Local Docker chain (localterp bootstrap)
pub const LOCAL_TERP: ChainInfo = ChainInfo {
    kind: ChainKind::Local,
    chain_id: "120u-1",
    gas_denom: "uterp",
    gas_price: 0.25,
    grpc_urls: &["http://localhost:9090"],
    network_info: TERPNET,
    lcd_url: None,
    fcd_url: None,
};
