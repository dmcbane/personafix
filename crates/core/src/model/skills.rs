use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub linked_attribute: String,
    pub group: Option<String>,
    pub rating: u8,
    pub specializations: Vec<Specialization>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Specialization {
    pub name: String,
    /// +2 dice bonus in both editions.
    pub bonus: u8,
}

/// SR4 has explicit skill groups; SR5 collapsed them.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SkillGroup {
    pub name: String,
    pub skills: Vec<String>,
    pub rating: u8,
}
