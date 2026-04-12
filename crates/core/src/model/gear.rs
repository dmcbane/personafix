use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub category: String,
    pub damage: String,
    pub ap: String,
    pub mode: String,
    pub recoil_comp: i32,
    pub ammo: String,
    pub availability: String,
    pub cost: i64,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Armor {
    pub id: String,
    pub name: String,
    pub armor_value: i32,
    pub availability: String,
    pub cost: i64,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GearItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub rating: Option<u8>,
    pub availability: String,
    pub cost: i64,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Vehicle {
    pub id: String,
    pub name: String,
    pub handling: String,
    pub speed: i32,
    pub acceleration: i32,
    pub body: i32,
    pub armor: i32,
    pub pilot: i32,
    pub sensor: i32,
    pub availability: String,
    pub cost: i64,
    pub source: String,
    pub page: String,
}
