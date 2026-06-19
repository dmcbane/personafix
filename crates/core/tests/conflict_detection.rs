//! TDD seed for the harness: SR4 conflict detection ("identify and report conflicts").
//!
//! These tests encode the *spec* for a capability that does not exist yet.
//! `SR4Rules::validate_creation` currently checks attribute bounds, skill caps,
//! BP budget, quality BP limits, and resources — but it does NOT detect:
//!   1. mutually-incompatible qualities (the `Quality::incompatible_with` field
//!      is populated from game data but never consulted),
//!   2. essence overage (augmentations whose total essence cost drives essence <= 0),
//!   3. magic + resonance simultaneously (a character cannot be both Awakened and
//!      a technomancer in SR4).
//!
//! They are `#[ignore]`d ON PURPOSE so the committed suite stays green, but they are
//! NOT silently skipped: each carries a loud reason pointing at the backlog item.
//! Run them with `cargo test -p personafix-core -- --ignored` to watch the oracle
//! fail for the right reason. The first job of the loop (backlog P4-*) is to remove
//! the `#[ignore]` and implement detection until these pass.

use personafix_core::model::{
    attributes::{Attributes, Metatype},
    augmentations::{Augmentation, AugmentationGrade, AugmentationType},
    character::CharacterDraft,
    edition::Edition,
    qualities::{Quality, QualityType},
    validation::ValidationSeverity,
};
use personafix_core::rules::{sr4::SR4Rules, traits::CharacterRules};

/// A minimal, BP-legal SR4 Human draft with no conflicts. Tests mutate this.
fn legal_human_draft() -> CharacterDraft {
    CharacterDraft {
        name: "Conflict Test".to_string(),
        edition: Edition::SR4,
        metatype: Metatype::Human,
        attributes: Attributes {
            body: 3,
            agility: 3,
            reaction: 3,
            strength: 3,
            willpower: 3,
            logic: 3,
            intuition: 3,
            charisma: 3,
            edge: 3,
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

fn quality(id: &str, qtype: QualityType, incompatible_with: Vec<String>) -> Quality {
    Quality {
        id: id.to_string(),
        name: id.to_string(),
        quality_type: qtype,
        cost: 5,
        source: "SR4".to_string(),
        page: "1".to_string(),
        improvements: vec![],
        incompatible_with,
    }
}

fn augmentation(id: &str, essence_cost: i32) -> Augmentation {
    Augmentation {
        id: id.to_string(),
        name: id.to_string(),
        augmentation_type: AugmentationType::Cyberware,
        grade: AugmentationGrade::Standard,
        essence_cost,
        availability: "4".to_string(),
        cost: 10_000,
        source: "SR4".to_string(),
        page: "1".to_string(),
        improvements: vec![],
    }
}

#[test]
fn incompatible_qualities_are_reported_as_warning() {
    let mut draft = legal_human_draft();
    // Two qualities that declare each other incompatible.
    draft.qualities.push(quality(
        "lucky",
        QualityType::Positive,
        vec!["bad_luck".to_string()],
    ));
    draft.qualities.push(quality(
        "bad_luck",
        QualityType::Negative,
        vec!["lucky".to_string()],
    ));

    let errors = SR4Rules.validate_creation(&draft);

    // The user wants conflicts REPORTED, not hard-blocked → severity Warning.
    let conflict = errors
        .iter()
        .find(|e| e.field.contains("incompat") && e.message.to_lowercase().contains("lucky"));
    assert!(
        conflict.is_some(),
        "expected an incompatible-quality conflict to be reported, got: {errors:?}"
    );
    assert_eq!(
        conflict.unwrap().severity,
        ValidationSeverity::Warning,
        "incompatible qualities should be a reported Warning, not a hard Error"
    );
}

#[test]
fn essence_overage_is_reported_as_error() {
    let mut draft = legal_human_draft();
    // Augmentations totaling 7.00 essence — impossible, base essence is 6.00.
    draft.augmentations.push(augmentation("cyberlimbs", 400));
    draft
        .augmentations
        .push(augmentation("wired_reflexes_3", 300));

    let errors = SR4Rules.validate_creation(&draft);

    let essence_err = errors
        .iter()
        .find(|e| e.field.contains("essence") && e.severity == ValidationSeverity::Error);
    assert!(
        essence_err.is_some(),
        "expected an essence-overage Error (augmentations exceed 6.00 essence), got: {errors:?}"
    );
}

#[test]
#[ignore = "TDD seed — implement in backlog item P4-3 (magic/resonance exclusivity); remove #[ignore] then"]
fn magic_and_resonance_together_is_reported_as_error() {
    let mut draft = legal_human_draft();
    // A character cannot be both Awakened (Magic) and a technomancer (Resonance).
    draft.attributes.magic = Some(3);
    draft.attributes.resonance = Some(3);

    let errors = SR4Rules.validate_creation(&draft);

    let exclusivity_err = errors.iter().find(|e| {
        e.field.contains("magic")
            || e.field.contains("resonance")
            || e.message.to_lowercase().contains("resonance")
    });
    assert!(
        exclusivity_err.is_some(),
        "expected a magic/resonance mutual-exclusivity conflict, got: {errors:?}"
    );
}
