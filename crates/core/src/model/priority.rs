use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// SR5 priority table categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PriorityLevel {
    A,
    B,
    C,
    D,
    E,
}

/// SR5 priority category types. Each priority level is assigned to exactly one category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum PriorityCategory {
    Metatype,
    Attributes,
    MagicOrResonance,
    Skills,
    Resources,
}

/// A complete SR5 priority selection — one level per category, each used exactly once.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PrioritySelection {
    pub metatype: PriorityLevel,
    pub attributes: PriorityLevel,
    pub magic_or_resonance: PriorityLevel,
    pub skills: PriorityLevel,
    pub resources: PriorityLevel,
}
