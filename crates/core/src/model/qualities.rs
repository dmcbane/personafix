use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::improvements::Improvement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum QualityType {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Quality {
    pub id: String,
    pub name: String,
    pub quality_type: QualityType,
    /// BP cost (SR4) or karma cost (SR5).
    pub cost: i32,
    pub source: String,
    pub page: String,
    pub improvements: Vec<Improvement>,
    /// Quality IDs that are incompatible with this one.
    pub incompatible_with: Vec<String>,
}
