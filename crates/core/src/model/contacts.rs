use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub connection: u8,
    pub loyalty: u8,
    pub archetype: String,
    pub notes: String,
}
