use cw_orch::prelude::*;
use whitelist_mtree::{
    contract::{execute, instantiate, query},
    msg::*,
};

#[cw_orch::interface(
    InstantiateMsg,
    ExecuteMsg,
    QueryMsg,
    Empty,
    id = "whitelist_merkletree"
)]
pub struct WhitelistMerkleTree;

impl<Chain: CwEnv> Uploadable for WhitelistMerkleTree<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("whitelist_merkletree")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }
}
