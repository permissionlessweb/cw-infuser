use cosmwasm_std::{to_json_binary, Addr, Binary, Deps, HexBinary, QueryRequest, Storage, WasmQuery};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw_storage_plus::{Item, Map};

use crate::CwInfuserError;

#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub default_infusion_params: DefaultInfusionParams,
    pub latest_infusion_id: Option<u64>,
}


#[cosmwasm_schema::cw_serde]
pub struct Infusion {
    pub collections: Vec<NFTCollection>,
    pub infused_collection: InfusedCollection,
    pub infusion_params: InfusionParams,
    pub infusion_id: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const COUNT: Item<i32> = Item::new("count");
pub const INFUSION: Map<(Addr, u64), Infusion> = Map::new("infusion");
pub const INFUSION_ID: Map<u64, (Addr, u64)> = Map::new("infusion_id");
pub const INFUSION_INFO: Map<&Addr, InfusionInfo> = Map::new("infusion_info");


#[cosmwasm_schema::cw_serde]
pub struct DefaultInfusionParams {
    /// min nfts required to be included in a bundle 
    pub min_required: u64,
    /// cw721-base code_id
    pub code_id: u64,
}
#[cosmwasm_schema::cw_serde]
pub struct InfusionParams {
    pub amount_required: u64,
    pub params: BurnParams,
}


#[cosmwasm_schema::cw_serde]
pub struct NFT {
    pub addr: Addr,
    pub token_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct NFTCollection {
    pub addr: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct InfusedCollection {
    pub addr: Addr,
    pub admin: Option<String>,
    pub name: String,
    pub symbol: String,
}

#[cosmwasm_schema::cw_serde]
pub struct BurnParams {
    pub compatible_traits: Option<CompatibleTraits>,
}

#[cosmwasm_schema::cw_serde]
pub struct CompatibleTraits {
    pub a: String,
    pub b: String,
}

#[cosmwasm_schema::cw_serde]
pub struct Bundle {
    pub nfts: Vec<NFT>,
}
#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct InfusionInfo {
    pub next_id: u64,
}



pub const SALT_POSTFIX: &[u8] = b"infusion";
pub fn generate_instantiate_salt2(checksum: &HexBinary) -> Binary {
    let account_id_hash = <sha2::Sha256 as sha2::Digest>::digest(checksum.to_string());
    let mut hash = account_id_hash.to_vec();
    hash.extend(SALT_POSTFIX);
    Binary(hash.to_vec())
}

fn get_next_id(storage: &mut dyn Storage, addr: Addr) -> Result<u64, CwInfuserError> {
    let token_id = INFUSION_INFO
        .update::<_, CwInfuserError>(storage, &addr, |x| match x {
            Some(mut info) => {
                info.next_id += 1;
                Ok(info)
            }
            None => Ok(InfusionInfo::default()),
        })?
        .next_id;
    Ok(token_id)
}

pub fn is_nft_owner(deps: Deps, sender: Addr, nfts: Vec<NFT>) -> Result<(), CwInfuserError> {
    for nft in nfts {
        let nft_address = nft.addr;
        let token_id = nft.token_id;

        let owner_response: OwnerOfResponse =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: nft_address.to_string(),
                msg: to_json_binary(&Cw721QueryMsg::OwnerOf {
                    token_id: token_id.to_string(),
                    include_expired: None,
                })?,
            }))?;

        if owner_response.owner != sender.to_string() {
            return Err(CwInfuserError::SenderNotOwner {});
        }
    }
    Ok(())
}