use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An improvement modifies a character's stats. Qualities, augmentations, adept powers,
/// and gear all produce improvements. The engine resolves them into a final ComputedCharacter.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Improvement {
    AttributeModifier {
        attribute: String,
        value: i32,
    },
    SkillModifier {
        skill: String,
        value: i32,
    },
    InitiativeDice {
        value: i32,
    },
    ArmorModifier {
        value: i32,
    },
    EssenceCost {
        /// Centessence (hundredths).
        value: i32,
    },
    DamageResistance {
        value: i32,
    },
    LimitModifier {
        limit: String,
        value: i32,
    },
    ConditionMonitorModifier {
        monitor: String,
        value: i32,
    },
    SpecialModifier {
        name: String,
        value: String,
    },
}
