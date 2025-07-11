use crate::error::ContractError;
use crate::{
    CreateEventTickets, ExecuteMsg, InstantiateMsg, MintTicketObject, OsmosisMintObject, QueryMsg,
};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_json_binary, AnyMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Reply, Response, StdResult, SubMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Item;

pub const WAVS_SMART_ACCOUNT: Item<String> = Item::new("wsa");
pub const EVENT_ID: Item<String> = Item::new("eid");

// version info for migration info
const INFUSION_COLLECTION_INIT_MSG_ID: u64 = 21;
const CONTRACT_NAME: &str = "crates.io:cw-infuser";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn handle_mint_tickets(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Vec<MintTicketObject>,
) -> Result<Response, ContractError> {
    // ensure admin only
    if info.sender.to_string() != WAVS_SMART_ACCOUNT.load(deps.storage)? {
        return Err(ContractError::SoldOut {});
    };

    let mut mint_msgs = Vec::new();
    for mint in msg {
        mint_msgs.push(AnyMsg {
            type_url: "/osmosis.tokenfactory.v1beta1.MsgMint".into(),
            value: to_json_binary(&OsmosisMintObject {
                sender: env.contract.address.to_string(),
                amount: coin(mint.amount, EVENT_ID.load(deps.storage)?),
                mint_to_address: mint.ticket, // account serving as ephemeral ticket
            })?,
        });
    }

    Ok(Response::new().add_messages(mint_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    WAVS_SMART_ACCOUNT.save(deps.storage, &info.sender.to_string())?;
    EVENT_ID.save(
        deps.storage,
        &format!(
            "factory/{}/{}",
            env.contract.address, msg.event_ticket_label
        ),
    )?;

    Ok(Response::new().add_submessage(
        SubMsg::reply_on_success(
            AnyMsg {
                type_url: "/osmosis.tokenfactory.v1beta1.MsgCreateDenom".into(),
                value: to_json_binary(&CreateEventTickets {
                    sender: env.contract.address.to_string(),
                    subdenom: msg.event_ticket_label,
                })?,
            },
            1,
        )
        .with_payload(to_json_binary(&msg.event_metadata)?),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintTickets { data } => handle_mint_tickets(deps, env, info, data),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        1 => Ok(Response::new().add_message(AnyMsg {
            type_url: "/osmosis.tokenfactory.v1beta1.MsgSetDenomMetadata".into(),
            value: to_json_binary(&msg.payload)?,
        })),
        _ => Err(ContractError::SoldOut {}),
    }
}
