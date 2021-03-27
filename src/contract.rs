use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg, log, Coin, CosmosMsg, CanonicalAddr, HandleResult, Uint128
};

use cw20::{Cw20HandleMsg, MinterResponse};
use crate::msg::{HandleMsg, InitMsg, QueryMsg, ConfigResponse, PriceResponse};
use crate::state::{config, config_read, State};
use terraswap::querier::{simulate};
use terraswap::asset::{Asset};
use terraswap::pair::{HandleMsg as TerraswapHandleMsg};
use terraswap::token::InitMsg as TokenInitMsg;
use terraswap::hook::InitHook;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state: &State = &State {
        owner: deps.api.canonical_address(&env.message.sender)?,
        asset: deps.api.canonical_address(&msg.asset)?,
        pair: deps.api.canonical_address(&msg.pair)?,
        token: CanonicalAddr::default()
    };

    config(&mut deps.storage, &state)?;
    // Create LP token
    let mut messages: Vec<CosmosMsg> = vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: msg.token_code_id,
        msg: to_binary(&TokenInitMsg {
            name: "terraswap liquidity token".to_string(),
            symbol: "uLP".to_string(),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: env.contract.address.clone(),
                cap: None,
            }),
            init_hook: Some(InitHook {
                msg: to_binary(&HandleMsg::PostInitialize {})?,
                contract_addr: env.contract.address,
            }),
        })?,
        send: vec![],
        label: None,
    })];

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult  {
    match msg {
        HandleMsg::PostInitialize {} => try_post_initialize(deps, env),
        HandleMsg::Mint { asset, } => try_mint(deps, env, asset),
        HandleMsg::Redeem { asset, } => try_redeem(deps, env, asset),
    }
}

pub fn try_mint<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: Asset
) -> HandleResult {

    let state = config_read(&deps.storage)?;

    // swap asset from terraswap
    // deduct tax first
    let amount = (asset.deduct_tax(&deps)?).amount;
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&state.pair)?,
        msg: to_binary(&TerraswapHandleMsg::Swap {
            offer_asset: Asset {
                amount,
                ..asset
            },
            max_spread: None,
            belief_price: None,
            to: None,
        })?,
        send: vec![Coin {
            denom: "uusd".to_string(),
            amount,
        }],
    }));

    // mint token
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.human_address(&state.token)?,
        msg: to_binary(&Cw20HandleMsg::Mint {
            recipient: env.message.sender,
            amount: Uint128(1000000u128),
        })?,
        send: vec![],
    }));

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "swap"),
        ],
        data: None,
    })
}

// Must token contract execute it
pub fn try_post_initialize<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let state: State = config_read(&deps.storage)?;

    // permission check
    if state.token != CanonicalAddr::default() {
        return Err(StdError::unauthorized());
    }

    config(
        &mut deps.storage,
        &State {
            token: deps.api.canonical_address(&env.message.sender)?,
            ..state
        },
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("liquidity_token_addr", env.message.sender.as_str())],
        data: None,
    })
}

pub fn try_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: Asset,
) -> HandleResult {

    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Price { asset } => to_binary(&query_price(deps, asset)?),
    }
}

fn query_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<ConfigResponse> {
    let state = config_read(&deps.storage)?;
    Ok(ConfigResponse { owner: deps.api.human_address(&state.owner)? })
}

fn query_price<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, asset: Asset) -> StdResult<PriceResponse> {
    let state = config_read(&deps.storage)?;
    // query price from terraswap
    let response = simulate(&deps, &deps.api.human_address(&state.pair)?, &asset)?;
    Ok(PriceResponse { pair: deps.api.human_address(&state.pair)?, price: response })
}