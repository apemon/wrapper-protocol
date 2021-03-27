use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Uint128};
use terraswap::asset::{Asset};
use terraswap::pair::{SimulationResponse};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub token_code_id: u64,
    pub asset: HumanAddr,
    pub pair: HumanAddr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {

    //////////////////////
    /// Owner Operations
    //////////////////////
    PostInitialize {},
    //////////////////////
    /// User Operations
    //////////////////////
    Mint {
        asset: Asset,
    },
    Redeem {
        asset: Asset,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Price {
        asset: Asset
    }
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub pair: HumanAddr,
    pub price: SimulationResponse
}
