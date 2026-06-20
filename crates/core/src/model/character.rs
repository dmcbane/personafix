use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    attributes::{Attributes, Metatype},
    augmentations::Augmentation,
    contacts::Contact,
    edition::Edition,
    gear::{Armor, GearItem, Vehicle, Weapon},
    improvements::Improvement,
    magic::{AdeptPower, ComplexForm, MagicTradition, Spell},
    priority::PrioritySelection,
    qualities::Quality,
    skills::{KnowledgeSkill, Skill, SkillGroup},
    validation::ValidationError,
};

/// A character in-progress during creation. Not yet validated or finalized.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CharacterDraft {
    pub name: String,
    pub edition: Edition,
    pub metatype: Metatype,
    pub attributes: Attributes,
    pub skills: Vec<Skill>,
    pub skill_groups: Vec<SkillGroup>,
    #[serde(default)]
    pub knowledge_skills: Vec<KnowledgeSkill>,
    pub qualities: Vec<Quality>,
    pub augmentations: Vec<Augmentation>,
    pub spells: Vec<Spell>,
    pub adept_powers: Vec<AdeptPower>,
    pub complex_forms: Vec<ComplexForm>,
    pub contacts: Vec<Contact>,
    pub weapons: Vec<Weapon>,
    pub armor: Vec<Armor>,
    pub gear: Vec<GearItem>,
    pub vehicles: Vec<Vehicle>,
    /// SR5 only.
    pub priority_selection: Option<PrioritySelection>,
    /// SR5 only. Which awakened tradition the character follows.
    #[serde(default)]
    pub magic_tradition: Option<MagicTradition>,
    /// Magician sub-tradition (SR5): "Hermetic", "Shaman", "Other", etc.
    #[serde(default)]
    pub tradition_name: Option<String>,
    /// SR4: BP total. SR5: karma total.
    pub creation_points_spent: i32,
    pub nuyen_spent: i64,
}

/// The persisted base state of a finalized character.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CharacterBase {
    pub id: String,
    pub campaign_id: String,
    pub name: String,
    pub edition: Edition,
    pub metatype: Metatype,
    pub attributes: Attributes,
    pub skills: Vec<Skill>,
    pub skill_groups: Vec<SkillGroup>,
    #[serde(default)]
    pub knowledge_skills: Vec<KnowledgeSkill>,
    pub qualities: Vec<Quality>,
    pub augmentations: Vec<Augmentation>,
    pub spells: Vec<Spell>,
    pub adept_powers: Vec<AdeptPower>,
    pub complex_forms: Vec<ComplexForm>,
    pub contacts: Vec<Contact>,
    pub weapons: Vec<Weapon>,
    pub armor: Vec<Armor>,
    pub gear: Vec<GearItem>,
    pub vehicles: Vec<Vehicle>,
    pub priority_selection: Option<PrioritySelection>,
    #[serde(default)]
    pub magic_tradition: Option<MagicTradition>,
    /// Magician sub-tradition: "Hermetic", "Shaman", "Other", etc.
    /// SR4 uses totem/tradition qualities instead; leave null for SR4.
    #[serde(default)]
    pub tradition_name: Option<String>,
    #[serde(default)]
    pub notes: String,
}

/// Fully computed character state — the projection of base + all ledger events.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ComputedCharacter {
    pub base: CharacterBase,
    /// All resolved improvements (from qualities, augmentations, gear, etc.).
    pub active_improvements: Vec<Improvement>,
    /// Final computed attributes after all improvements.
    pub computed_attributes: Attributes,
    /// Validation errors/warnings for the current state.
    pub validation_errors: Vec<ValidationError>,
    /// Total karma earned across all runs.
    pub total_karma_earned: i32,
    /// Total karma spent on improvements.
    pub total_karma_spent: i32,
    /// Current nuyen balance.
    pub nuyen: i64,
    /// Physical condition monitor boxes.
    pub physical_condition_monitor: u8,
    /// Stun condition monitor boxes.
    pub stun_condition_monitor: u8,
    /// Initiative score.
    pub initiative: i32,
    /// Initiative dice.
    pub initiative_dice: u8,
    /// Current initiation grade (Awakened characters). 0 = not initiated.
    pub initiation_grade: u8,
    /// Current submersion grade (Emerged characters). 0 = not submerged.
    pub submersion_grade: u8,
}

/// Lightweight summary for list views.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CharacterSummary {
    pub id: String,
    pub name: String,
    pub edition: Edition,
    pub metatype: Metatype,
    pub total_karma: i32,
}
