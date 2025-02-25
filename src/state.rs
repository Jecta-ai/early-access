use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhitelistData {
    pub ref_code: String,
    pub ref_address:String,
    pub count: u64,
}

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const WHITELIST: Map<String, bool> = Map::new("whitelist");
pub const REFFERALS: Map<String,WhitelistData> = Map::new("refferals");
