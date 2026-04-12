use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::improvements::Improvement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AugmentationGrade {
    Standard,
    Alpha,
    Beta,
    Delta,
    Used,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AugmentationType {
    Cyberware,
    Bioware,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Augmentation {
    pub id: String,
    pub name: String,
    pub augmentation_type: AugmentationType,
    pub grade: AugmentationGrade,
    /// Base essence cost in centessence (hundredths). 100 = 1.00 Essence.
    pub essence_cost: i32,
    pub availability: String,
    pub cost: i64,
    pub source: String,
    pub page: String,
    pub improvements: Vec<Improvement>,
}

/// Computed essence value. Stored as centessence to avoid floating point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Essence(pub i32);

impl Essence {
    pub const MAX: Essence = Essence(600);

    pub fn as_f64(self) -> f64 {
        self.0 as f64 / 100.0
    }
}
