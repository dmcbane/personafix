//! Improvement resolver: processes the full improvement chain from qualities,
//! augmentations, adept powers, and gear into a flat list of active improvements.

use crate::model::{character::CharacterBase, improvements::Improvement};

/// Collect all active improvements from a character's qualities, augmentations,
/// and other sources into a single flat list.
pub fn resolve_improvements(base: &CharacterBase) -> Vec<Improvement> {
    let mut improvements = Vec::new();

    for quality in &base.qualities {
        improvements.extend(quality.improvements.iter().cloned());
    }

    for aug in &base.augmentations {
        improvements.extend(aug.improvements.iter().cloned());
    }

    // Future: adept powers, gear bonuses, etc.

    improvements
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        attributes::{Attributes, Metatype},
        edition::Edition,
        improvements::Improvement,
        qualities::{Quality, QualityType},
    };

    fn empty_base() -> CharacterBase {
        CharacterBase {
            id: "test".to_string(),
            campaign_id: "test".to_string(),
            name: "Test".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes::default(),
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
        }
    }

    #[test]
    fn resolve_empty_base_returns_no_improvements() {
        let base = empty_base();
        let imps = resolve_improvements(&base);
        assert!(imps.is_empty());
    }

    #[test]
    fn resolve_collects_quality_improvements() {
        let mut base = empty_base();
        base.qualities.push(Quality {
            id: "q1".to_string(),
            name: "Toughness".to_string(),
            quality_type: QualityType::Positive,
            cost: 10,
            source: "SR4".to_string(),
            page: "1".to_string(),
            improvements: vec![Improvement::AttributeModifier {
                attribute: "body".to_string(),
                value: 1,
            }],
            incompatible_with: vec![],
        });
        let imps = resolve_improvements(&base);
        assert_eq!(imps.len(), 1);
    }
}
