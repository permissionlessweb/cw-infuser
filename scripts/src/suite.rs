use cosmwasm_std::{coin, coins, Decimal, Event, Fraction, HexBinary, Uint128};

use cosmwasm_std::Binary;
use cw721_svg::interface::Cw721Svg;
use cw_infusion_minter::interface::CwInfuser;
use cw_infusion_minter::{
    msg::{ExecuteMsg, ExecuteMsgFns, InstantiateMsg, QueryMsgFns},
    state::Config,
    AnyOfErr, ContractError,
};
use cw_infusions::{
    bundles::{Bundle, BundleType},
    nfts::{InfusedCollection, NFT},
    state::{EligibleNFTCollection, Infusion, InfusionParamState},
    wavs::WavsBundle,
};
use cw_orch::{anyhow, prelude::*};
use cw_svg::InstantiateMsg as SvgInitMsg;

#[derive(Clone, Debug)]
pub struct CwSvgSuiteDeployData {
    pub svg: Option<cw_svg::InstantiateMsg>,
    pub infuse: Option<cw_infusion_minter::msg::InstantiateMsg>,
    pub admin: Option<Addr>,
    pub infuse_coins: Vec<Coin>,
}

pub struct CwSvgSuite<Chain> {
    pub chain: Chain,
    pub infuser: CwInfuser<Chain>,
    pub cwsvg: Cw721Svg<Chain>,
    // pub nfts: Vec<Addr>,
    // pub admin: Addr,
    // pub wavs_service: Addr,
    // pub payment_recipient: Addr,
}

impl<Chain: CwEnv> CwSvgSuite<Chain> {
    pub fn new(chain: Chain) -> CwSvgSuite<Chain> {
        CwSvgSuite::<Chain> {
            chain: chain.clone(),
            infuser: CwInfuser::new(chain.clone()),
            cwsvg: Cw721Svg::new(chain.clone()),
        }
    }
    pub fn upload(&self) -> Result<(), CwOrchError> {
        self.cwsvg.upload()?;
        self.infuser.upload()?;
        Ok(())
    }
}
// Bitsong Accounts `Deploy` Suite
impl<Chain: CwEnv> cw_orch::contract::Deploy<Chain> for CwSvgSuite<Chain> {
    type Error = CwOrchError;
    type DeployData = Option<CwSvgSuiteDeployData>;

    fn store_on(chain: Chain) -> Result<Self, Self::Error> {
        let suite = CwSvgSuite::new(chain.clone());
        suite.upload()?;
        Ok(suite)
    }

    fn deployed_state_file_path() -> Option<String> {
        todo!()
    }

    fn get_contracts_mut(&mut self) -> Vec<Box<&mut dyn ContractInstance<Chain>>> {
        vec![Box::new(&mut self.cwsvg), Box::new(&mut self.infuser)]
    }

    fn load_from(chain: Chain) -> Result<Self, Self::Error> {
        todo!()
    }

    /// upload, and if DeployData is provided, instantiate
    fn deploy_on(chain: Chain, data: Self::DeployData) -> Result<Self, Self::Error> {
        let mut suite = CwSvgSuite::store_on(chain.clone())?;
        // upload and initalize
        if let Some(init) = data {
            let admin = init.admin.as_ref();
            if let Some(i) = &init.svg {
                suite.cwsvg.instantiate(i, admin, None)?;
            }
            if let Some(i) = &init.infuse {
                suite.infuser.instantiate(&i, admin, None)?;
            }
            // TODO: Vec<ExecuteMsg> for each contract
        }
        Ok(suite)
    }
    fn get_all_deployed_chains() -> Vec<String> {
        vec![]
    }
}

pub mod svg {
    use super::*;
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
}
