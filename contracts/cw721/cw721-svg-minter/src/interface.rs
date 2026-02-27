use crate::{
    contract::{execute, instantiate, query},
    msg::*,
};
use cw_orch::prelude::*;

#[cw_orch::interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty, id = "cw_svg_minter")]
pub struct Cw721SvgMinter;

impl<Chain: CwEnv> Uploadable for Cw721SvgMinter<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("cw_svg_minter")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }
}
