//! SR5 Priority system helpers for character creation.
//!
//! Each priority level (A-E) is assigned to exactly one category (Metatype,
//! Attributes, Magic/Resonance, Skills, Resources). Each level used exactly once.

use crate::model::{
    attributes::RacialLimits,
    character::CharacterDraft,
    priority::{PriorityLevel, PrioritySelection},
    qualities::QualityType,
};

/// Maximum karma on positive qualities in SR5.
pub const MAX_POSITIVE_QUALITY_KARMA: i32 = 25;

/// Maximum karma on negative qualities in SR5.
pub const MAX_NEGATIVE_QUALITY_KARMA: i32 = 25;

/// Attribute points granted by each priority level.
pub fn attribute_points(level: PriorityLevel) -> i32 {
    match level {
        PriorityLevel::A => 24,
        PriorityLevel::B => 20,
        PriorityLevel::C => 16,
        PriorityLevel::D => 14,
        PriorityLevel::E => 12,
    }
}

/// Skill points granted: (individual_skill_points, skill_group_points).
pub fn skill_points(level: PriorityLevel) -> (i32, i32) {
    match level {
        PriorityLevel::A => (46, 10),
        PriorityLevel::B => (36, 5),
        PriorityLevel::C => (28, 2),
        PriorityLevel::D => (22, 0),
        PriorityLevel::E => (18, 0),
    }
}

/// Nuyen granted by each resource priority level.
pub fn resource_nuyen(level: PriorityLevel) -> i64 {
    match level {
        PriorityLevel::A => 450_000,
        PriorityLevel::B => 275_000,
        PriorityLevel::C => 140_000,
        PriorityLevel::D => 50_000,
        PriorityLevel::E => 6_000,
    }
}

/// Validate that a priority selection uses each level exactly once.
/// Returns a list of error messages (empty if valid).
pub fn validate_priority_selection(selection: &PrioritySelection) -> Vec<String> {
    let levels = [
        selection.metatype,
        selection.attributes,
        selection.magic_or_resonance,
        selection.skills,
        selection.resources,
    ];

    let mut errors = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for level in &levels {
        if !seen.insert(level) {
            errors.push(format!("Priority level {level:?} is used more than once"));
        }
    }

    // Check all 5 levels are present
    for required in &[
        PriorityLevel::A,
        PriorityLevel::B,
        PriorityLevel::C,
        PriorityLevel::D,
        PriorityLevel::E,
    ] {
        if !seen.contains(required) {
            errors.push(format!("Priority level {required:?} is not assigned"));
        }
    }

    errors
}

/// Count attribute points spent above racial minimums.
pub fn attribute_points_spent(draft: &CharacterDraft, limits: &RacialLimits) -> i32 {
    let mut spent = 0i32;
    spent += draft.attributes.body as i32 - limits.body.0 as i32;
    spent += draft.attributes.agility as i32 - limits.agility.0 as i32;
    spent += draft.attributes.reaction as i32 - limits.reaction.0 as i32;
    spent += draft.attributes.strength as i32 - limits.strength.0 as i32;
    spent += draft.attributes.willpower as i32 - limits.willpower.0 as i32;
    spent += draft.attributes.logic as i32 - limits.logic.0 as i32;
    spent += draft.attributes.intuition as i32 - limits.intuition.0 as i32;
    spent += draft.attributes.charisma as i32 - limits.charisma.0 as i32;
    // Edge is bought separately in SR5 (special attribute points from metatype priority)
    // but we validate it against racial limits
    spent
}

/// Count skill points spent (individual skill rating points).
pub fn skill_points_spent(draft: &CharacterDraft) -> i32 {
    draft.skills.iter().map(|s| s.rating as i32).sum()
}

/// Count skill group points spent.
pub fn skill_group_points_spent(draft: &CharacterDraft) -> i32 {
    draft.skill_groups.iter().map(|g| g.rating as i32).sum()
}

/// Karma spent on positive qualities.
pub fn positive_quality_karma(draft: &CharacterDraft) -> i32 {
    draft
        .qualities
        .iter()
        .filter(|q| q.quality_type == QualityType::Positive)
        .map(|q| q.cost)
        .sum()
}

/// Karma from negative qualities (absolute value).
pub fn negative_quality_karma(draft: &CharacterDraft) -> i32 {
    draft
        .qualities
        .iter()
        .filter(|q| q.quality_type == QualityType::Negative)
        .map(|q| q.cost.abs())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_priority_selection_passes() {
        let sel = PrioritySelection {
            metatype: PriorityLevel::D,
            attributes: PriorityLevel::A,
            magic_or_resonance: PriorityLevel::B,
            skills: PriorityLevel::C,
            resources: PriorityLevel::E,
        };
        assert!(validate_priority_selection(&sel).is_empty());
    }

    #[test]
    fn duplicate_priority_level_fails() {
        let sel = PrioritySelection {
            metatype: PriorityLevel::A,
            attributes: PriorityLevel::A, // duplicate
            magic_or_resonance: PriorityLevel::B,
            skills: PriorityLevel::C,
            resources: PriorityLevel::E,
        };
        let errors = validate_priority_selection(&sel);
        assert!(!errors.is_empty());
    }

    #[test]
    fn attribute_points_a_is_24() {
        assert_eq!(attribute_points(PriorityLevel::A), 24);
    }

    #[test]
    fn skill_points_b_is_36_5() {
        assert_eq!(skill_points(PriorityLevel::B), (36, 5));
    }

    #[test]
    fn resource_nuyen_c_is_140k() {
        assert_eq!(resource_nuyen(PriorityLevel::C), 140_000);
    }
}
