use cosmwasm_std::{
    Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_storage_plus::Item;
use cosmwasm_std::entry_point;

// storage info
const CONTRACT_NAME: &str = "crates.io:buy-me-coffee";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Storage keys
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: String,
    pub coffee_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Memo {
    pub supporter: String,
    pub message: String,
    pub amount: Uint128,
    pub timestamp: u64,
}

// storage
pub const CONFIG: Item<Config> = Item::new("config");
pub const MEMOS: Item<Vec<Memo>> = Item::new("memos");
pub const TOTAL_COFFEES: Item<Uint128> = Item::new("total_coffees");

// Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub coffee_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum ExecuteMsg {
    BuyCoffee { message: String },
    UpdatePrice { new_price: Coin },
    Withdraw {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub coffee_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MemosResponse {
    pub memos: Vec<Memo>,
}

// -------------------- Entry Points --------------------

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        owner: info.sender.to_string(),
        coffee_price: msg.coffee_price,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    MEMOS.save(deps.storage, &vec![])?;
    TOTAL_COFFEES.save(deps.storage, &Uint128::zero())?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("coffee_price", config.coffee_price.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::BuyCoffee { message } => buy_coffee(deps, env, info, message),
        ExecuteMsg::UpdatePrice { new_price } => update_price(deps, info, new_price),
        ExecuteMsg::Withdraw {} => withdraw(deps, info),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, _msg: ()) -> StdResult<ConfigResponse> {
    query_config(deps)
}

// -------------------- Execute functions --------------------

fn buy_coffee(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    message: String,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // Check if sent enough funds
    if info.funds.len() != 1 || info.funds[0] != config.coffee_price {
        return Err(cosmwasm_std::StdError::generic_err(
            format!("Please send exactly {} to buy a coffee", config.coffee_price),
        ));
    }

    // create memo
    let memo = Memo {
        supporter: info.sender.to_string(),
        message,
        amount: info.funds[0].amount,
        timestamp: env.block.time.seconds(),
    };

    // Update storage
    let mut memos = MEMOS.load(deps.storage)?;
    memos.push(memo);
    MEMOS.save(deps.storage, &memos)?;

    let mut total = TOTAL_COFFEES.load(deps.storage)?;
    total += Uint128::new(1);
    TOTAL_COFFEES.save(deps.storage, &total)?;

    Ok(Response::new()
        .add_attribute("action", "buy_coffee")
        .add_attribute("supporter", info.sender)
        .add_attribute("total_coffees", total.to_string()))
}

fn update_price(
    deps: DepsMut,
    info: MessageInfo,
    new_price: Coin,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // Only owner can update price
    if info.sender.to_string() != config.owner {
        return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
    }

    config.coffee_price = new_price;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_price")
        .add_attribute("new_price", config.coffee_price.to_string()))
}

fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // Only owner can withdraw
    if info.sender.to_string() != config.owner {
        return Err(cosmwasm_std::StdError::generic_err("Unauthorized"));
    }

    // In a real contract, you'd add logic to send funds to owner

    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("owner", config.owner))
}

// -------------------- Queries --------------------

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        coffee_price: config.coffee_price,
    })
}

fn main() {
    // For now, nothing runs here because CosmWasm contracts 
    // are not executed like normal Rust binaries.
    println!("Coffee contract compiled successfully!");
}

