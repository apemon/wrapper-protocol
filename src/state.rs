use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, CanonicalAddr, Storage, StdResult};
use cosmwasm_storage::{ReadonlySingleton, Singleton};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: CanonicalAddr,
    pub asset: CanonicalAddr,
    pub pair: CanonicalAddr,
    pub token: CanonicalAddr
}

pub fn config<S: Storage>(storage: &mut S, data: &State) -> StdResult<()> {
    Singleton::new(storage, CONFIG_KEY).save(data)
}

pub fn config_read<S: Storage>(storage: &S) -> StdResult<State> {
    ReadonlySingleton::new(storage, CONFIG_KEY).load()
}
