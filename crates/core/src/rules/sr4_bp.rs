//! SR4 Build Point (BP) calculation helpers for character creation.
//!
//! Standard SR4 character creation uses 400 BP.

use crate::model::{attributes::RacialLimits, character::CharacterDraft, qualities::QualityType};

/// Standard BP budget for SR4 character creation.
pub const STANDARD_BP: i32 = 400;

/// Maximum BP that can be spent on resources (nuyen).
pub const MAX_RESOURCE_BP: i32 = 50;

/// Maximum BP from positive qualities.
pub const MAX_POSITIVE_QUALITY_BP: i32 = 35;

/// Maximum BP from negative qualities.
pub const MAX_NEGATIVE_QUALITY_BP: i32 = 35;

/// Nuyen per BP spent on resources.
pub const NUYEN_PER_BP: i64 = 5000;

/// BP cost for attributes: 10 BP per point above racial minimum.
pub fn bp_cost_attributes(draft: &CharacterDraft, limits: &RacialLimits) -> i32 {
    let mut cost = 0i32;
    cost += (draft.attributes.body as i32 - limits.body.0 as i32) * 10;
    cost += (draft.attributes.agility as i32 - limits.agility.0 as i32) * 10;
    cost += (draft.attributes.reaction as i32 - limits.reaction.0 as i32) * 10;
    cost += (draft.attributes.strength as i32 - limits.strength.0 as i32) * 10;
    cost += (draft.attributes.willpower as i32 - limits.willpower.0 as i32) * 10;
    cost += (draft.attributes.logic as i32 - limits.logic.0 as i32) * 10;
    cost += (draft.attributes.intuition as i32 - limits.intuition.0 as i32) * 10;
    cost += (draft.attributes.charisma as i32 - limits.charisma.0 as i32) * 10;
    cost += (draft.attributes.edge as i32 - limits.edge.0 as i32) * 10;
    // Magic/Resonance cost same as attributes if present
    if let Some(mag) = draft.attributes.magic {
        cost += mag as i32 * 10;
    }
    if let Some(res) = draft.attributes.resonance {
        cost += res as i32 * 10;
    }
    cost
}

/// BP cost for active skills: 4 BP per rating point.
/// BP cost for skill groups: 10 BP per rating point.
pub fn bp_cost_skills(draft: &CharacterDraft) -> i32 {
    let active: i32 = draft.skills.iter().map(|s| s.rating as i32 * 4).sum();
    let groups: i32 = draft
        .skill_groups
        .iter()
        .map(|g| g.rating as i32 * 10)
        .sum();
    active + groups
}

/// BP cost/gain for qualities.
/// Positive qualities cost BP, negative qualities give BP back (returned as negative).
pub fn bp_cost_qualities(draft: &CharacterDraft) -> i32 {
    draft
        .qualities
        .iter()
        .map(|q| match q.quality_type {
            QualityType::Positive => q.cost,
            QualityType::Negative => -q.cost.abs(),
        })
        .sum()
}

/// BP spent on positive qualities only.
pub fn bp_positive_qualities(draft: &CharacterDraft) -> i32 {
    draft
        .qualities
        .iter()
        .filter(|q| q.quality_type == QualityType::Positive)
        .map(|q| q.cost)
        .sum()
}

/// BP from negative qualities (absolute value).
pub fn bp_negative_qualities(draft: &CharacterDraft) -> i32 {
    draft
        .qualities
        .iter()
        .filter(|q| q.quality_type == QualityType::Negative)
        .map(|q| q.cost.abs())
        .sum()
}

/// BP cost for resources: nuyen_spent / NUYEN_PER_BP.
pub fn bp_cost_resources(draft: &CharacterDraft) -> i32 {
    (draft.nuyen_spent / NUYEN_PER_BP) as i32
}

/// BP cost for contacts: 1 BP per (connection + loyalty) point.
pub fn bp_cost_contacts(draft: &CharacterDraft) -> i32 {
    draft
        .contacts
        .iter()
        .map(|c| c.connection as i32 + c.loyalty as i32)
        .sum()
}

/// Total BP spent on the draft character.
pub fn bp_total(draft: &CharacterDraft, limits: &RacialLimits) -> i32 {
    bp_cost_attributes(draft, limits)
        + bp_cost_skills(draft)
        + bp_cost_qualities(draft)
        + bp_cost_resources(draft)
        + bp_cost_contacts(draft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        attributes::{Attributes, Metatype},
        contacts::Contact,
        edition::Edition,
        qualities::Quality,
        skills::Skill,
    };

    fn human_limits() -> RacialLimits {
        RacialLimits {
            metatype: Metatype::Human,
            edition: Edition::SR4,
            body: (1, 6),
            agility: (1, 6),
            reaction: (1, 6),
            strength: (1, 6),
            willpower: (1, 6),
            logic: (1, 6),
            intuition: (1, 6),
            charisma: (1, 6),
            edge: (2, 7),
        }
    }

    fn minimal_draft() -> CharacterDraft {
        CharacterDraft {
            name: "Test".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 1,
                agility: 1,
                reaction: 1,
                strength: 1,
                willpower: 1,
                logic: 1,
                intuition: 1,
                charisma: 1,
                edge: 2,
                essence: 600,
                magic: None,
                resonance: None,
            },
            skills: vec![],
            skill_groups: vec![],
            qualities: vec![],
            augmentations: vec![],
            spells: vec![],
            adept_powers: vec![],
            complex_forms: vec![],
            contacts: vec![],
            weapons: vec![],
            armor: vec![],
            gear: vec![],
            vehicles: vec![],
            priority_selection: None,
            creation_points_spent: 0,
            nuyen_spent: 0,
        }
    }

    #[test]
    fn bp_attributes_at_racial_min_costs_zero() {
        let draft = minimal_draft();
        assert_eq!(bp_cost_attributes(&draft, &human_limits()), 0);
    }

    #[test]
    fn bp_attributes_above_min_costs_10_per_point() {
        let mut draft = minimal_draft();
        draft.attributes.body = 4; // 3 above min
        draft.attributes.agility = 3; // 2 above min
        assert_eq!(bp_cost_attributes(&draft, &human_limits()), 50); // 30 + 20
    }

    #[test]
    fn bp_skills_4_per_rating() {
        let mut draft = minimal_draft();
        draft.skills.push(Skill {
            id: "s1".to_string(),
            name: "Pistols".to_string(),
            linked_attribute: "AGI".to_string(),
            group: None,
            rating: 4,
            specializations: vec![],
        });
        assert_eq!(bp_cost_skills(&draft), 16);
    }

    #[test]
    fn bp_qualities_positive_costs_negative_gives() {
        let mut draft = minimal_draft();
        draft.qualities.push(Quality {
            id: "q1".to_string(),
            name: "Ambidextrous".to_string(),
            quality_type: QualityType::Positive,
            cost: 5,
            source: "SR4".to_string(),
            page: "1".to_string(),
            improvements: vec![],
            incompatible_with: vec![],
        });
        draft.qualities.push(Quality {
            id: "q2".to_string(),
            name: "Addiction".to_string(),
            quality_type: QualityType::Negative,
            cost: 10,
            source: "SR4".to_string(),
            page: "1".to_string(),
            improvements: vec![],
            incompatible_with: vec![],
        });
        assert_eq!(bp_cost_qualities(&draft), -5); // 5 - 10
    }

    #[test]
    fn bp_resources_5000_nuyen_per_bp() {
        let mut draft = minimal_draft();
        draft.nuyen_spent = 100_000;
        assert_eq!(bp_cost_resources(&draft), 20);
    }

    #[test]
    fn bp_contacts_connection_plus_loyalty() {
        let mut draft = minimal_draft();
        draft.contacts.push(Contact {
            id: "c1".to_string(),
            name: "Fixer".to_string(),
            connection: 3,
            loyalty: 2,
            archetype: "Fixer".to_string(),
            notes: String::new(),
        });
        assert_eq!(bp_cost_contacts(&draft), 5);
    }
}
