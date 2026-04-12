use crate::model::{
    attributes::{Attributes, Metatype, RacialLimits},
    augmentations::{Augmentation, AugmentationGrade, Essence},
    character::{CharacterBase, CharacterDraft, ComputedCharacter},
    edition::Edition,
    improvements::Improvement,
    validation::{ValidationError, ValidationSeverity},
};

use crate::engine::{resolver, stacker};

use super::{
    sr4_bp::{
        self, MAX_NEGATIVE_QUALITY_BP, MAX_POSITIVE_QUALITY_BP, MAX_RESOURCE_BP, STANDARD_BP,
    },
    traits::CharacterRules,
};

/// SR4 rules implementation using the Build Point system.
pub struct SR4Rules;

impl SR4Rules {
    /// Grade multiplier for essence cost calculation (in hundredths).
    /// Standard=100, Alpha=80, Beta=70, Delta=50, Used=125.
    fn grade_multiplier(grade: AugmentationGrade) -> i32 {
        match grade {
            AugmentationGrade::Standard => 100,
            AugmentationGrade::Alpha => 80,
            AugmentationGrade::Beta => 70,
            AugmentationGrade::Delta => 50,
            AugmentationGrade::Used => 125,
        }
    }
}

impl CharacterRules for SR4Rules {
    fn creation_method(&self) -> &str {
        "Build Points"
    }

    fn karma_cost_skill(&self, from: u8, to: u8) -> u32 {
        // New rating × 2 for each step. New skill (0→1) costs 4 (treated as 2×2).
        (from + 1..=to).map(|r| r as u32 * 2).sum()
    }

    fn karma_cost_attribute(&self, from: u8, to: u8) -> u32 {
        // New rating × 5 for each step.
        (from + 1..=to).map(|r| r as u32 * 5).sum()
    }

    fn calculate_essence(&self, augmentations: &[Augmentation]) -> Essence {
        let mut remaining = Essence::MAX.0; // 600 centessence

        for aug in augmentations {
            let base_cost = aug.essence_cost; // Already in centessence
            let adjusted = (base_cost * Self::grade_multiplier(aug.grade)) / 100;
            remaining -= adjusted;
        }

        Essence(remaining)
    }

    fn validate_creation(&self, draft: &CharacterDraft) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let limits = self.racial_limits(draft.metatype);

        // Check attribute bounds
        validate_attribute_bounds(&mut errors, &draft.attributes, &limits);

        // Check skill rating caps
        for skill in &draft.skills {
            if skill.rating > self.max_skill_rating_at_creation() {
                errors.push(ValidationError {
                    severity: ValidationSeverity::Error,
                    field: format!("skills.{}", skill.name),
                    message: format!(
                        "Skill {} rating {} exceeds creation maximum of {}",
                        skill.name,
                        skill.rating,
                        self.max_skill_rating_at_creation()
                    ),
                });
            }
        }

        // BP budget
        let total_bp = sr4_bp::bp_total(draft, &limits);
        if total_bp > STANDARD_BP {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "bp_total".to_string(),
                message: format!("Total BP ({total_bp}) exceeds maximum ({STANDARD_BP})"),
            });
        }

        // Positive quality limit
        let pos_bp = sr4_bp::bp_positive_qualities(draft);
        if pos_bp > MAX_POSITIVE_QUALITY_BP {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "qualities.positive".to_string(),
                message: format!(
                    "Positive quality BP ({pos_bp}) exceeds maximum ({MAX_POSITIVE_QUALITY_BP})"
                ),
            });
        }

        // Negative quality limit
        let neg_bp = sr4_bp::bp_negative_qualities(draft);
        if neg_bp > MAX_NEGATIVE_QUALITY_BP {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "qualities.negative".to_string(),
                message: format!(
                    "Negative quality BP ({neg_bp}) exceeds maximum ({MAX_NEGATIVE_QUALITY_BP})"
                ),
            });
        }

        // Resource limit
        let res_bp = sr4_bp::bp_cost_resources(draft);
        if res_bp > MAX_RESOURCE_BP {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "resources".to_string(),
                message: format!("Resource BP ({res_bp}) exceeds maximum ({MAX_RESOURCE_BP})"),
            });
        }

        errors
    }

    fn apply_improvements(
        &self,
        base: &CharacterBase,
        improvements: &[Improvement],
    ) -> ComputedCharacter {
        // Collect improvements from the character's qualities, augmentations, etc.
        let mut all_improvements = resolver::resolve_improvements(base);
        all_improvements.extend(improvements.iter().cloned());

        // Apply attribute modifiers
        let computed_attributes =
            stacker::apply_improvements_to_attributes(&base.attributes, &all_improvements);

        // Calculate essence from augmentations
        let essence = self.calculate_essence(&base.augmentations);
        let mut final_attributes = computed_attributes;
        final_attributes.essence = essence.0;

        // Derived stats
        let phys_cm = self.physical_condition_monitor(final_attributes.body);
        let stun_cm = self.stun_condition_monitor(final_attributes.willpower);
        let init = self.initiative_score(final_attributes.reaction, final_attributes.intuition);

        // Count initiative dice bonuses from improvements
        let bonus_init_dice: i32 = all_improvements
            .iter()
            .filter_map(|imp| {
                if let Improvement::InitiativeDice { value } = imp {
                    Some(*value)
                } else {
                    None
                }
            })
            .sum();
        let init_dice = (self.initiative_dice() as i32 + bonus_init_dice).clamp(1, 5) as u8;

        // Validation on current state
        let validation_errors = Vec::new();

        ComputedCharacter {
            base: base.clone(),
            active_improvements: all_improvements,
            computed_attributes: final_attributes,
            validation_errors,
            total_karma_earned: 0,
            total_karma_spent: 0,
            nuyen: 0,
            physical_condition_monitor: phys_cm,
            stun_condition_monitor: stun_cm,
            initiative: init,
            initiative_dice: init_dice,
        }
    }

    fn physical_condition_monitor(&self, body: u8) -> u8 {
        8 + body.div_ceil(2)
    }

    fn stun_condition_monitor(&self, willpower: u8) -> u8 {
        8 + willpower.div_ceil(2)
    }

    fn initiative_score(&self, reaction: u8, intuition: u8) -> i32 {
        reaction as i32 + intuition as i32
    }

    fn initiative_dice(&self) -> u8 {
        1
    }

    fn racial_limits(&self, metatype: Metatype) -> RacialLimits {
        let edition = Edition::SR4;
        match metatype {
            Metatype::Human => RacialLimits {
                metatype,
                edition,
                body: (1, 6),
                agility: (1, 6),
                reaction: (1, 6),
                strength: (1, 6),
                willpower: (1, 6),
                logic: (1, 6),
                intuition: (1, 6),
                charisma: (1, 6),
                edge: (2, 7),
            },
            Metatype::Elf => RacialLimits {
                metatype,
                edition,
                body: (1, 6),
                agility: (2, 7),
                reaction: (1, 6),
                strength: (1, 6),
                willpower: (1, 6),
                logic: (1, 6),
                intuition: (1, 6),
                charisma: (3, 8),
                edge: (1, 6),
            },
            Metatype::Dwarf => RacialLimits {
                metatype,
                edition,
                body: (2, 7),
                agility: (1, 6),
                reaction: (1, 5),
                strength: (2, 8),
                willpower: (2, 7),
                logic: (1, 6),
                intuition: (1, 6),
                charisma: (1, 6),
                edge: (1, 6),
            },
            Metatype::Ork => RacialLimits {
                metatype,
                edition,
                body: (3, 8),
                agility: (1, 6),
                reaction: (1, 6),
                strength: (3, 8),
                willpower: (1, 6),
                logic: (1, 5),
                intuition: (1, 6),
                charisma: (1, 5),
                edge: (1, 6),
            },
            Metatype::Troll => RacialLimits {
                metatype,
                edition,
                body: (5, 10),
                agility: (1, 5),
                reaction: (1, 6),
                strength: (5, 10),
                willpower: (1, 6),
                logic: (1, 5),
                intuition: (1, 5),
                charisma: (1, 4),
                edge: (1, 6),
            },
        }
    }

    fn max_skill_rating_at_creation(&self) -> u8 {
        6
    }

    fn max_skill_rating(&self) -> u8 {
        12
    }
}

fn validate_attribute_bounds(
    errors: &mut Vec<ValidationError>,
    attrs: &Attributes,
    limits: &RacialLimits,
) {
    let checks = [
        ("body", attrs.body, limits.body),
        ("agility", attrs.agility, limits.agility),
        ("reaction", attrs.reaction, limits.reaction),
        ("strength", attrs.strength, limits.strength),
        ("willpower", attrs.willpower, limits.willpower),
        ("logic", attrs.logic, limits.logic),
        ("intuition", attrs.intuition, limits.intuition),
        ("charisma", attrs.charisma, limits.charisma),
        ("edge", attrs.edge, limits.edge),
    ];

    for (name, value, (min, max)) in checks {
        if value < min {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: format!("attributes.{name}"),
                message: format!("{name} ({value}) is below racial minimum ({min})"),
            });
        }
        if value > max {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: format!("attributes.{name}"),
                message: format!("{name} ({value}) exceeds racial maximum ({max})"),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        augmentations::{AugmentationGrade, AugmentationType},
        contacts::Contact,
        qualities::{Quality, QualityType},
        skills::Skill,
    };

    fn rules() -> SR4Rules {
        SR4Rules
    }

    // -- Karma cost tests --

    #[test]
    fn karma_cost_skill_single_step() {
        // Rating 2→3 costs 3×2 = 6 karma
        assert_eq!(rules().karma_cost_skill(2, 3), 6);
    }

    #[test]
    fn karma_cost_skill_range() {
        // Rating 0→4: (1×2)+(2×2)+(3×2)+(4×2) = 2+4+6+8 = 20
        assert_eq!(rules().karma_cost_skill(0, 4), 20);
    }

    #[test]
    fn karma_cost_skill_no_change() {
        assert_eq!(rules().karma_cost_skill(3, 3), 0);
    }

    #[test]
    fn karma_cost_attribute_single_step() {
        // Rating 4→5 costs 5×5 = 25 karma
        assert_eq!(rules().karma_cost_attribute(4, 5), 25);
    }

    #[test]
    fn karma_cost_attribute_range() {
        // Rating 3→5: (4×5)+(5×5) = 20+25 = 45 karma
        assert_eq!(rules().karma_cost_attribute(3, 5), 45);
    }

    // -- Essence tests --

    #[test]
    fn essence_no_augmentations() {
        assert_eq!(rules().calculate_essence(&[]), Essence::MAX);
    }

    #[test]
    fn essence_standard_grade() {
        let augs = vec![make_aug(
            "Wired Reflexes 1",
            200,
            AugmentationGrade::Standard,
        )];
        let essence = rules().calculate_essence(&augs);
        assert_eq!(essence, Essence(400)); // 600 - 200 = 400
    }

    #[test]
    fn essence_alpha_grade() {
        let augs = vec![make_aug("Cybereyes", 50, AugmentationGrade::Alpha)];
        let essence = rules().calculate_essence(&augs);
        // 50 × 0.8 = 40 centessence
        assert_eq!(essence, Essence(560)); // 600 - 40
    }

    #[test]
    fn essence_beta_grade() {
        let augs = vec![make_aug("Cybereyes", 100, AugmentationGrade::Beta)];
        let essence = rules().calculate_essence(&augs);
        // 100 × 0.7 = 70
        assert_eq!(essence, Essence(530)); // 600 - 70
    }

    #[test]
    fn essence_delta_grade() {
        let augs = vec![make_aug("Cybereyes", 100, AugmentationGrade::Delta)];
        let essence = rules().calculate_essence(&augs);
        // 100 × 0.5 = 50
        assert_eq!(essence, Essence(550)); // 600 - 50
    }

    #[test]
    fn essence_used_grade() {
        let augs = vec![make_aug("Used Arm", 100, AugmentationGrade::Used)];
        let essence = rules().calculate_essence(&augs);
        // 100 × 1.25 = 125
        assert_eq!(essence, Essence(475)); // 600 - 125
    }

    #[test]
    fn essence_multiple_augmentations() {
        let augs = vec![
            make_aug("Wired Reflexes 1", 200, AugmentationGrade::Standard),
            make_aug("Cybereyes", 50, AugmentationGrade::Alpha),
        ];
        let essence = rules().calculate_essence(&augs);
        // 200 + 40 = 240 total cost
        assert_eq!(essence, Essence(360)); // 600 - 240
    }

    // -- Racial limits tests --

    #[test]
    fn racial_limits_human() {
        let limits = rules().racial_limits(Metatype::Human);
        assert_eq!(limits.body, (1, 6));
        assert_eq!(limits.edge, (2, 7));
    }

    #[test]
    fn racial_limits_elf() {
        let limits = rules().racial_limits(Metatype::Elf);
        assert_eq!(limits.agility, (2, 7));
        assert_eq!(limits.charisma, (3, 8));
        assert_eq!(limits.edge, (1, 6));
    }

    #[test]
    fn racial_limits_dwarf() {
        let limits = rules().racial_limits(Metatype::Dwarf);
        assert_eq!(limits.body, (2, 7));
        assert_eq!(limits.strength, (2, 8));
        assert_eq!(limits.reaction, (1, 5));
        assert_eq!(limits.willpower, (2, 7));
    }

    #[test]
    fn racial_limits_ork() {
        let limits = rules().racial_limits(Metatype::Ork);
        assert_eq!(limits.body, (3, 8));
        assert_eq!(limits.strength, (3, 8));
        assert_eq!(limits.charisma, (1, 5));
        assert_eq!(limits.logic, (1, 5));
    }

    #[test]
    fn racial_limits_troll() {
        let limits = rules().racial_limits(Metatype::Troll);
        assert_eq!(limits.body, (5, 10));
        assert_eq!(limits.strength, (5, 10));
        assert_eq!(limits.agility, (1, 5));
        assert_eq!(limits.charisma, (1, 4));
    }

    // -- Condition monitor tests --

    #[test]
    fn physical_condition_monitor_body_5() {
        assert_eq!(rules().physical_condition_monitor(5), 11);
    }

    #[test]
    fn physical_condition_monitor_body_4() {
        assert_eq!(rules().physical_condition_monitor(4), 10);
    }

    #[test]
    fn stun_condition_monitor_willpower_4() {
        assert_eq!(rules().stun_condition_monitor(4), 10);
    }

    #[test]
    fn stun_condition_monitor_willpower_3() {
        assert_eq!(rules().stun_condition_monitor(3), 10);
    }

    // -- Initiative tests --

    #[test]
    fn initiative_score_basic() {
        assert_eq!(rules().initiative_score(4, 5), 9);
    }

    #[test]
    fn initiative_dice_default() {
        assert_eq!(rules().initiative_dice(), 1);
    }

    // -- Validation tests --

    #[test]
    fn validate_legal_character_no_errors() {
        let draft = make_legal_human_draft();
        let errors = rules().validate_creation(&draft);
        let real_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            real_errors.is_empty(),
            "Expected no errors, got: {real_errors:?}"
        );
    }

    #[test]
    fn validate_over_bp_budget() {
        let mut draft = make_legal_human_draft();
        // Max out all attributes to blow the budget
        draft.attributes.body = 6;
        draft.attributes.agility = 6;
        draft.attributes.reaction = 6;
        draft.attributes.strength = 6;
        draft.attributes.willpower = 6;
        draft.attributes.logic = 6;
        draft.attributes.intuition = 6;
        draft.attributes.charisma = 6;
        draft.attributes.edge = 7;
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "bp_total"));
    }

    #[test]
    fn validate_attribute_over_racial_max() {
        let mut draft = make_legal_human_draft();
        draft.attributes.body = 8; // Human max is 6
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "attributes.body"));
    }

    #[test]
    fn validate_attribute_below_racial_min() {
        let mut draft = make_legal_human_draft();
        draft.attributes.edge = 1; // Human Edge min is 2
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "attributes.edge"));
    }

    #[test]
    fn validate_skill_over_creation_max() {
        let mut draft = make_legal_human_draft();
        draft.skills.push(Skill {
            id: "s1".to_string(),
            name: "Pistols".to_string(),
            linked_attribute: "AGI".to_string(),
            group: None,
            rating: 7, // Max is 6
            specializations: vec![],
        });
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "skills.Pistols"));
    }

    #[test]
    fn validate_positive_quality_limit() {
        let mut draft = make_legal_human_draft();
        // Add positive qualities totaling > 35 BP
        draft
            .qualities
            .push(make_quality("Q1", QualityType::Positive, 20));
        draft
            .qualities
            .push(make_quality("Q2", QualityType::Positive, 20));
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "qualities.positive"));
    }

    #[test]
    fn validate_negative_quality_limit() {
        let mut draft = make_legal_human_draft();
        draft
            .qualities
            .push(make_quality("Q1", QualityType::Negative, 20));
        draft
            .qualities
            .push(make_quality("Q2", QualityType::Negative, 20));
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "qualities.negative"));
    }

    #[test]
    fn validate_resource_bp_limit() {
        let mut draft = make_legal_human_draft();
        draft.nuyen_spent = 300_000; // 60 BP, max is 50
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "resources"));
    }

    // -- apply_improvements tests --

    #[test]
    fn apply_improvements_basic() {
        let base = make_base_character();
        let improvements = vec![Improvement::AttributeModifier {
            attribute: "body".to_string(),
            value: 2,
        }];
        let computed = rules().apply_improvements(&base, &improvements);
        assert_eq!(computed.computed_attributes.body, base.attributes.body + 2);
    }

    #[test]
    fn apply_improvements_computes_condition_monitors() {
        let base = make_base_character();
        let computed = rules().apply_improvements(&base, &[]);
        assert_eq!(
            computed.physical_condition_monitor,
            rules().physical_condition_monitor(base.attributes.body)
        );
        assert_eq!(
            computed.stun_condition_monitor,
            rules().stun_condition_monitor(base.attributes.willpower)
        );
    }

    #[test]
    fn apply_improvements_computes_initiative() {
        let base = make_base_character();
        let computed = rules().apply_improvements(&base, &[]);
        assert_eq!(
            computed.initiative,
            rules().initiative_score(base.attributes.reaction, base.attributes.intuition)
        );
        assert_eq!(computed.initiative_dice, 1);
    }

    #[test]
    fn apply_improvements_initiative_dice_bonus() {
        let base = make_base_character();
        let improvements = vec![Improvement::InitiativeDice { value: 2 }];
        let computed = rules().apply_improvements(&base, &improvements);
        assert_eq!(computed.initiative_dice, 3);
    }

    #[test]
    fn apply_improvements_computes_essence() {
        let mut base = make_base_character();
        base.augmentations.push(make_aug(
            "Wired Reflexes 1",
            200,
            AugmentationGrade::Standard,
        ));
        let computed = rules().apply_improvements(&base, &[]);
        assert_eq!(computed.computed_attributes.essence, 400); // 600 - 200
    }

    // -- Canonical Street Samurai test --

    #[test]
    fn canonical_sr4_street_samurai() {
        // A BP-legal SR4 Human Street Samurai build.
        // Attributes (costs): BOD 5(40), AGI 5(40), REA 4(30), STR 4(30),
        //   WIL 3(20), LOG 2(10), INT 4(30), CHA 2(10), EDG 3(10)
        // = 220 BP on attributes
        let mut draft = CharacterDraft {
            name: "Street Samurai".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 5,
                agility: 5,
                reaction: 4,
                strength: 4,
                willpower: 3,
                logic: 2,
                intuition: 4,
                charisma: 2,
                edge: 3,
                essence: 600,
                magic: None,
                resonance: None,
            },
            skills: vec![
                make_skill("Automatics", "AGI", 5),         // 20 BP
                make_skill("Pistols", "AGI", 4),            // 16 BP
                make_skill("Dodge", "REA", 4),              // 16 BP
                make_skill("Perception", "INT", 3),         // 12 BP
                make_skill("Infiltration", "AGI", 3),       // 12 BP
                make_skill("Intimidation", "CHA", 2),       // 8 BP
                make_skill("Pilot Ground Craft", "REA", 1), // 4 BP
            ],
            // Skills: 88 BP
            skill_groups: vec![],
            qualities: vec![
                make_quality("Ambidextrous", QualityType::Positive, 5), // 5 BP
                make_quality("High Pain Tolerance", QualityType::Positive, 5), // 5 BP
                make_quality("SINner", QualityType::Negative, 5),       // -5 BP
                make_quality("Addiction (Mild)", QualityType::Negative, 5), // -5 BP
            ],
            // Qualities: 5 + 5 - 5 - 5 = 0 BP
            augmentations: vec![],
            spells: vec![],
            adept_powers: vec![],
            complex_forms: vec![],
            contacts: vec![
                Contact {
                    id: "c1".to_string(),
                    name: "Fixer".to_string(),
                    connection: 3,
                    loyalty: 3,
                    archetype: "Fixer".to_string(),
                    notes: String::new(),
                },
                Contact {
                    id: "c2".to_string(),
                    name: "Street Doc".to_string(),
                    connection: 2,
                    loyalty: 2,
                    archetype: "Street Doc".to_string(),
                    notes: String::new(),
                },
            ],
            // Contacts: (3+3)+(2+2) = 10 BP
            weapons: vec![],
            armor: vec![],
            gear: vec![],
            vehicles: vec![],
            priority_selection: None,
            creation_points_spent: 0,
            nuyen_spent: 200_000, // 40 BP
        };
        draft.creation_points_spent = 0; // Will be calculated

        // Verify BP breakdown
        let limits = rules().racial_limits(Metatype::Human);
        assert_eq!(sr4_bp::bp_cost_attributes(&draft, &limits), 220);
        assert_eq!(sr4_bp::bp_cost_skills(&draft), 88);
        assert_eq!(sr4_bp::bp_cost_qualities(&draft), 0);
        assert_eq!(sr4_bp::bp_cost_contacts(&draft), 10);
        assert_eq!(sr4_bp::bp_cost_resources(&draft), 40);
        let total = sr4_bp::bp_total(&draft, &limits);
        assert_eq!(total, 358); // Under 400, legal

        // Validate — should have no errors
        let errors = rules().validate_creation(&draft);
        let real_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            real_errors.is_empty(),
            "Street Samurai should be valid, got: {real_errors:?}"
        );

        // Now create as a base character and apply improvements
        let base = CharacterBase {
            id: "ss1".to_string(),
            campaign_id: "camp1".to_string(),
            name: draft.name.clone(),
            edition: Edition::SR4,
            metatype: draft.metatype,
            attributes: draft.attributes.clone(),
            skills: draft.skills.clone(),
            skill_groups: draft.skill_groups.clone(),
            qualities: draft.qualities.clone(),
            augmentations: vec![make_aug(
                "Wired Reflexes 1",
                200,
                AugmentationGrade::Standard,
            )],
            spells: vec![],
            adept_powers: vec![],
            complex_forms: vec![],
            contacts: draft.contacts.clone(),
            weapons: vec![],
            armor: vec![],
            gear: vec![],
            vehicles: vec![],
            priority_selection: None,
        };

        let computed = rules().apply_improvements(&base, &[]);

        // Verify derived stats
        assert_eq!(computed.physical_condition_monitor, 11); // 8 + ceil(5/2) = 11
        assert_eq!(computed.stun_condition_monitor, 10); // 8 + ceil(3/2) = 10
        assert_eq!(computed.initiative, 8); // REA 4 + INT 4
        assert_eq!(computed.initiative_dice, 1);
        assert_eq!(computed.computed_attributes.essence, 400); // 6.00 - 2.00 = 4.00
        assert_eq!(computed.computed_attributes.body, 5);
        assert_eq!(computed.computed_attributes.agility, 5);
    }

    // -- Test helpers --

    fn make_legal_human_draft() -> CharacterDraft {
        // A minimal but legal 400 BP human character
        CharacterDraft {
            name: "Legal Human".to_string(),
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
            skills: vec![
                make_skill("Pistols", "AGI", 4),
                make_skill("Dodge", "REA", 3),
            ],
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
            nuyen_spent: 50_000,
        }
    }

    fn make_base_character() -> CharacterBase {
        CharacterBase {
            id: "test".to_string(),
            campaign_id: "test".to_string(),
            name: "Test Character".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 4,
                agility: 4,
                reaction: 3,
                strength: 3,
                willpower: 3,
                logic: 3,
                intuition: 3,
                charisma: 2,
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
        }
    }

    fn make_skill(name: &str, attr: &str, rating: u8) -> Skill {
        Skill {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            linked_attribute: attr.to_string(),
            group: None,
            rating,
            specializations: vec![],
        }
    }

    fn make_quality(name: &str, quality_type: QualityType, cost: i32) -> Quality {
        Quality {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            quality_type,
            cost,
            source: "SR4".to_string(),
            page: "1".to_string(),
            improvements: vec![],
            incompatible_with: vec![],
        }
    }

    fn make_aug(name: &str, essence_cost: i32, grade: AugmentationGrade) -> Augmentation {
        Augmentation {
            id: name.to_lowercase().replace(' ', "_"),
            name: name.to_string(),
            augmentation_type: AugmentationType::Cyberware,
            grade,
            essence_cost,
            availability: "4".to_string(),
            cost: 10000,
            source: "SR4".to_string(),
            page: "1".to_string(),
            improvements: vec![],
        }
    }
}
