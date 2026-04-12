use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SpellCategory {
    Combat,
    Detection,
    Health,
    Illusion,
    Manipulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SpellType {
    Physical,
    Mana,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Spell {
    pub id: String,
    pub name: String,
    pub category: SpellCategory,
    pub spell_type: SpellType,
    pub range: String,
    pub damage: String,
    pub duration: String,
    pub drain: String,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AdeptPower {
    pub id: String,
    pub name: String,
    /// Power point cost in hundredths (25 = 0.25 PP).
    pub cost: i32,
    pub levels: bool,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComplexForm {
    pub id: String,
    pub name: String,
    pub target: String,
    pub duration: String,
    pub fading: String,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InitiationGrade(pub u8);
