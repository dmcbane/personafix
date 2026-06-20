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
    sr5_priority::{self, MAX_NEGATIVE_QUALITY_KARMA, MAX_POSITIVE_QUALITY_KARMA},
    traits::CharacterRules,
};

/// SR5 rules implementation using the Priority system.
pub struct SR5Rules;

impl SR5Rules {
    /// Grade multiplier for essence cost (same as SR4).
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

impl CharacterRules for SR5Rules {
    fn creation_method(&self) -> &str {
        "Priority"
    }

    fn karma_cost_skill(&self, from: u8, to: u8) -> u32 {
        // Same formula as SR4: new_rating × 2 per step.
        (from + 1..=to).map(|r| r as u32 * 2).sum()
    }

    fn karma_cost_attribute(&self, from: u8, to: u8) -> u32 {
        // Same formula as SR4: new_rating × 5 per step.
        (from + 1..=to).map(|r| r as u32 * 5).sum()
    }

    fn calculate_essence(&self, augmentations: &[Augmentation]) -> Essence {
        let mut remaining = Essence::MAX.0;

        for aug in augmentations {
            let adjusted = (aug.essence_cost * Self::grade_multiplier(aug.grade)) / 100;
            remaining -= adjusted;
        }

        Essence(remaining)
    }

    fn validate_creation(&self, draft: &CharacterDraft) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let limits = self.racial_limits(draft.metatype);

        // Priority selection must be present
        let Some(ref priority) = draft.priority_selection else {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "priority_selection".to_string(),
                message: "Priority selection is required for SR5 character creation".to_string(),
            });
            return errors;
        };

        // Validate priority selection (each level used exactly once)
        let priority_errors = sr5_priority::validate_priority_selection(priority);
        for msg in priority_errors {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "priority_selection".to_string(),
                message: msg,
            });
        }

        // Attribute bounds
        validate_attribute_bounds(&mut errors, &draft.attributes, &limits);

        // Attribute points budget
        let attr_allocated = sr5_priority::attribute_points(priority.attributes);
        let attr_spent = sr5_priority::attribute_points_spent(draft, &limits);
        if attr_spent > attr_allocated {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "attributes".to_string(),
                message: format!(
                    "Attribute points spent ({attr_spent}) exceed allocation ({attr_allocated})"
                ),
            });
        }

        // Skill points budget
        let (skill_allocated, group_allocated) = sr5_priority::skill_points(priority.skills);
        let skill_spent = sr5_priority::skill_points_spent(draft);
        let group_spent = sr5_priority::skill_group_points_spent(draft);
        if skill_spent > skill_allocated {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "skills".to_string(),
                message: format!(
                    "Skill points spent ({skill_spent}) exceed allocation ({skill_allocated})"
                ),
            });
        }
        if group_spent > group_allocated {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "skill_groups".to_string(),
                message: format!(
                    "Skill group points spent ({group_spent}) exceed allocation ({group_allocated})"
                ),
            });
        }

        // Skill rating caps
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

        // Quality karma limits
        let pos_karma = sr5_priority::positive_quality_karma(draft);
        if pos_karma > MAX_POSITIVE_QUALITY_KARMA {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "qualities.positive".to_string(),
                message: format!(
                    "Positive quality karma ({pos_karma}) exceeds maximum ({MAX_POSITIVE_QUALITY_KARMA})"
                ),
            });
        }

        let neg_karma = sr5_priority::negative_quality_karma(draft);
        if neg_karma > MAX_NEGATIVE_QUALITY_KARMA {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "qualities.negative".to_string(),
                message: format!(
                    "Negative quality karma ({neg_karma}) exceeds maximum ({MAX_NEGATIVE_QUALITY_KARMA})"
                ),
            });
        }

        // Magic tradition must be chosen if a non-Mundane priority is selected
        if sr5_priority::magic_starting_rating(priority.magic_or_resonance).is_some()
            && draft.magic_tradition.is_none()
        {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "magic_tradition".to_string(),
                message: "Awakened tradition must be chosen (Magician, Adept, Mystic Adept, or Technomancer)".to_string(),
            });
        }

        // Magic / resonance attribute must be set if a non-Mundane priority is chosen.
        // The starting value is free; raising above it costs special attribute points.
        // Hard cap for magic/resonance is 6 for all SR5 metatypes.
        const MAGIC_RESONANCE_MAX: u8 = 6;
        if let Some(starting) = sr5_priority::magic_starting_rating(priority.magic_or_resonance) {
            let awakened_val = draft.attributes.magic.or(draft.attributes.resonance);
            match awakened_val {
                None => errors.push(ValidationError {
                    severity: ValidationSeverity::Error,
                    field: "attributes.magic".to_string(),
                    message: format!(
                        "Magic or resonance attribute is required for non-Mundane priority (starting {starting})"
                    ),
                }),
                Some(v) if v > MAGIC_RESONANCE_MAX => errors.push(ValidationError {
                    severity: ValidationSeverity::Error,
                    field: "attributes.magic".to_string(),
                    message: format!(
                        "Magic/resonance rating {v} exceeds racial maximum {MAGIC_RESONANCE_MAX}"
                    ),
                }),
                Some(v) => {
                    // Special attribute points: magic above starting + edge above racial min
                    let magic_special = v.saturating_sub(starting) as i32;
                    let racial_edge_min = limits.edge.0 as i32;
                    let edge_special = (draft.attributes.edge as i32 - racial_edge_min).max(0);
                    let resonance_special = draft
                        .attributes
                        .resonance
                        .map(|r| r.saturating_sub(starting) as i32)
                        .unwrap_or(0);
                    let special_spent = magic_special + edge_special + resonance_special;
                    let special_pool =
                        sr5_priority::special_attribute_points(priority.metatype);
                    if special_spent > special_pool {
                        errors.push(ValidationError {
                            severity: ValidationSeverity::Error,
                            field: "special_attribute_pool".to_string(),
                            message: format!(
                                "Special attribute points spent ({special_spent}) exceed pool ({special_pool})"
                            ),
                        });
                    }
                }
            }
        } else {
            // Mundane: edge still uses special attribute points
            let racial_edge_min = limits.edge.0 as i32;
            let edge_special = (draft.attributes.edge as i32 - racial_edge_min).max(0);
            let special_pool = sr5_priority::special_attribute_points(priority.metatype);
            if edge_special > special_pool {
                errors.push(ValidationError {
                    severity: ValidationSeverity::Error,
                    field: "special_attribute_pool".to_string(),
                    message: format!(
                        "Special attribute points spent ({edge_special}) exceed pool ({special_pool})"
                    ),
                });
            }
        }

        // Contacts: Charisma × 3 free points; excess overflows (Warning, not Error)
        let contact_free_pool = draft.attributes.charisma as i32 * 3;
        let contact_points_spent: i32 = draft
            .contacts
            .iter()
            .map(|c| c.connection as i32 + c.loyalty as i32)
            .sum();
        if contact_points_spent > contact_free_pool {
            let overflow = contact_points_spent - contact_free_pool;
            errors.push(ValidationError {
                severity: ValidationSeverity::Warning,
                field: "contacts".to_string(),
                message: format!(
                    "Contact points ({contact_points_spent}) exceed free pool ({contact_free_pool}); {overflow} point(s) cost 1 karma each"
                ),
            });
        }

        // Resource limit
        let resource_limit = sr5_priority::resource_nuyen(priority.resources);
        if draft.nuyen_spent > resource_limit {
            errors.push(ValidationError {
                severity: ValidationSeverity::Error,
                field: "resources".to_string(),
                message: format!(
                    "Nuyen spent ({}) exceeds priority allocation ({resource_limit})",
                    draft.nuyen_spent
                ),
            });
        }

        errors
    }

    fn apply_improvements(
        &self,
        base: &CharacterBase,
        improvements: &[Improvement],
    ) -> ComputedCharacter {
        let mut all_improvements = resolver::resolve_improvements(base);
        all_improvements.extend(improvements.iter().cloned());

        let computed_attributes =
            stacker::apply_improvements_to_attributes(&base.attributes, &all_improvements);

        let essence = self.calculate_essence(&base.augmentations);
        let mut final_attributes = computed_attributes;
        final_attributes.essence = essence.0;

        let phys_cm = self.physical_condition_monitor(final_attributes.body);
        let stun_cm = self.stun_condition_monitor(final_attributes.willpower);
        let init = self.initiative_score(final_attributes.reaction, final_attributes.intuition);

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

        ComputedCharacter {
            base: base.clone(),
            active_improvements: all_improvements,
            computed_attributes: final_attributes,
            validation_errors: Vec::new(),
            total_karma_earned: 0,
            total_karma_spent: 0,
            nuyen: 0,
            physical_condition_monitor: phys_cm,
            stun_condition_monitor: stun_cm,
            initiative: init,
            initiative_dice: init_dice,
            initiation_grade: 0,
            submersion_grade: 0,
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
        let edition = Edition::SR5;
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
                body: (3, 8),
                agility: (1, 6),
                reaction: (1, 5),
                strength: (3, 8),
                willpower: (2, 7),
                logic: (1, 6),
                intuition: (1, 6),
                charisma: (1, 6),
                edge: (1, 6),
            },
            Metatype::Ork => RacialLimits {
                metatype,
                edition,
                body: (4, 9),
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
        magic::MagicTradition,
        priority::{PriorityLevel, PrioritySelection},
        qualities::{Quality, QualityType},
        skills::Skill,
    };

    fn rules() -> SR5Rules {
        SR5Rules
    }

    // -- Karma cost tests --

    #[test]
    fn karma_cost_skill_single_step() {
        assert_eq!(rules().karma_cost_skill(2, 3), 6);
    }

    #[test]
    fn karma_cost_skill_range() {
        assert_eq!(rules().karma_cost_skill(0, 4), 20);
    }

    #[test]
    fn karma_cost_attribute_range() {
        assert_eq!(rules().karma_cost_attribute(3, 5), 45);
    }

    // -- Essence tests --

    #[test]
    fn essence_no_augmentations() {
        assert_eq!(rules().calculate_essence(&[]), Essence::MAX);
    }

    #[test]
    fn essence_standard_grade() {
        let augs = vec![make_aug("Cybereyes", 50, AugmentationGrade::Standard)];
        assert_eq!(rules().calculate_essence(&augs), Essence(550));
    }

    #[test]
    fn essence_alpha_grade() {
        let augs = vec![make_aug("Cybereyes", 50, AugmentationGrade::Alpha)];
        // 50 × 0.8 = 40
        assert_eq!(rules().calculate_essence(&augs), Essence(560));
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
    }

    #[test]
    fn racial_limits_dwarf() {
        let limits = rules().racial_limits(Metatype::Dwarf);
        assert_eq!(limits.body, (3, 8));
        assert_eq!(limits.strength, (3, 8));
        assert_eq!(limits.willpower, (2, 7));
    }

    #[test]
    fn racial_limits_ork() {
        let limits = rules().racial_limits(Metatype::Ork);
        assert_eq!(limits.body, (4, 9));
        assert_eq!(limits.strength, (3, 8));
    }

    #[test]
    fn racial_limits_troll() {
        let limits = rules().racial_limits(Metatype::Troll);
        assert_eq!(limits.body, (5, 10));
        assert_eq!(limits.strength, (5, 10));
        assert_eq!(limits.charisma, (1, 4));
    }

    // -- Condition monitor tests --

    #[test]
    fn physical_condition_monitor_body_5() {
        assert_eq!(rules().physical_condition_monitor(5), 11);
    }

    #[test]
    fn stun_condition_monitor_willpower_4() {
        assert_eq!(rules().stun_condition_monitor(4), 10);
    }

    // -- Validation tests --

    #[test]
    fn validate_missing_priority_selection() {
        let mut draft = make_legal_human_draft();
        draft.priority_selection = None;
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "priority_selection"));
    }

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
    fn validate_attribute_over_racial_max() {
        let mut draft = make_legal_human_draft();
        draft.attributes.body = 8; // Human max is 6
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "attributes.body"));
    }

    #[test]
    fn validate_attribute_points_exceeded() {
        let mut draft = make_legal_human_draft();
        // Priority D attributes = 14 points. Max out everything to exceed.
        draft.attributes.body = 6;
        draft.attributes.agility = 6;
        draft.attributes.reaction = 6;
        // That's 5+5+5 = 15, over 14
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "attributes"));
    }

    #[test]
    fn validate_skill_points_exceeded() {
        let mut draft = make_legal_human_draft();
        // Priority C skills = 28 points.
        draft.skills = vec![
            make_skill("A", "AGI", 6),
            make_skill("B", "AGI", 6),
            make_skill("C", "AGI", 6),
            make_skill("D", "AGI", 6),
            make_skill("E", "AGI", 6),
        ]; // 30 > 28
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "skills"));
    }

    #[test]
    fn validate_skill_over_creation_max() {
        let mut draft = make_legal_human_draft();
        draft.skills.push(Skill {
            id: "s1".to_string(),
            name: "Pistols".to_string(),
            linked_attribute: "AGI".to_string(),
            group: None,
            rating: 7,
            specializations: vec![],
        });
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "skills.Pistols"));
    }

    #[test]
    fn validate_positive_quality_limit() {
        let mut draft = make_legal_human_draft();
        draft
            .qualities
            .push(make_quality("Q1", QualityType::Positive, 15));
        draft
            .qualities
            .push(make_quality("Q2", QualityType::Positive, 15));
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "qualities.positive"));
    }

    #[test]
    fn validate_resource_limit() {
        let mut draft = make_legal_human_draft();
        // Swap priorities: put E on resources, A on metatype
        draft.priority_selection = Some(PrioritySelection {
            metatype: PriorityLevel::A,
            attributes: PriorityLevel::B,
            magic_or_resonance: PriorityLevel::D,
            skills: PriorityLevel::C,
            resources: PriorityLevel::E, // 6,000¥ limit
        });
        draft.nuyen_spent = 10_000;
        let errors = rules().validate_creation(&draft);
        assert!(errors.iter().any(|e| e.field == "resources"));
    }

    #[test]
    fn validate_magic_required_when_magic_priority_set() {
        let mut draft = make_legal_human_draft();
        // Switch magic_or_resonance from E (Mundane) to B (Adept 6); adjust
        // resources away from A to keep all 5 levels distinct: D/A/B/C/E
        draft.priority_selection = Some(PrioritySelection {
            metatype: PriorityLevel::D,
            attributes: PriorityLevel::A,
            magic_or_resonance: PriorityLevel::B,
            skills: PriorityLevel::C,
            resources: PriorityLevel::E,
        });
        // magic attribute still None → should error
        draft.attributes.magic = None;
        let errors = rules().validate_creation(&draft);
        assert!(
            errors.iter().any(|e| e.field == "attributes.magic"),
            "Expected magic attribute error, got: {errors:?}"
        );
    }

    #[test]
    fn validate_magic_over_racial_max_errors() {
        // Magic 7 exceeds the racial max of 6 for all SR5 metatypes.
        let mut draft = make_legal_human_draft();
        draft.priority_selection = Some(PrioritySelection {
            metatype: PriorityLevel::A, // 13 special points — plenty
            attributes: PriorityLevel::B,
            magic_or_resonance: PriorityLevel::C,
            skills: PriorityLevel::D,
            resources: PriorityLevel::E,
        });
        draft.magic_tradition = Some(MagicTradition::Magician);
        draft.attributes.magic = Some(7); // exceeds racial max of 6
        let errors = rules().validate_creation(&draft);
        assert!(
            errors.iter().any(|e| e.field == "attributes.magic"),
            "Expected magic-exceeds-racial-max error, got: {errors:?}"
        );
    }

    #[test]
    fn validate_magic_tradition_required_when_awakened() {
        let mut draft = make_legal_human_draft();
        draft.priority_selection = Some(PrioritySelection {
            metatype: PriorityLevel::D,
            attributes: PriorityLevel::A,
            magic_or_resonance: PriorityLevel::B,
            skills: PriorityLevel::C,
            resources: PriorityLevel::E,
        });
        draft.attributes.magic = Some(6);
        draft.magic_tradition = None; // tradition not chosen → should error
        let errors = rules().validate_creation(&draft);
        assert!(
            errors.iter().any(|e| e.field == "magic_tradition"),
            "Expected magic_tradition error, got: {errors:?}"
        );
    }

    #[test]
    fn validate_mundane_magic_priority_allows_null_magic() {
        let draft = make_legal_human_draft(); // uses priority E / magic = None
        let errors = rules().validate_creation(&draft);
        let real_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            real_errors.is_empty(),
            "Mundane character should have no magic errors, got: {real_errors:?}"
        );
    }

    #[test]
    fn validate_special_attribute_pool_overspent_errors() {
        // Metatype E = 1 special point; Magician starting magic 3 (priority C);
        // spending 2 on magic (to 5) + 1 on edge (to 3) = 3 > 1 pool → error.
        let mut draft = make_legal_human_draft();
        draft.priority_selection = Some(PrioritySelection {
            metatype: PriorityLevel::E, // 1 special point
            attributes: PriorityLevel::B,
            magic_or_resonance: PriorityLevel::C,
            skills: PriorityLevel::A,
            resources: PriorityLevel::D,
        });
        draft.magic_tradition = Some(MagicTradition::Magician);
        draft.attributes.magic = Some(5); // 2 points above starting 3
        draft.attributes.edge = 3;        // 1 point above racial min 2
        let errors = rules().validate_creation(&draft);
        assert!(
            errors.iter().any(|e| e.field == "special_attribute_pool"),
            "Expected special pool error, got: {errors:?}"
        );
    }

    #[test]
    fn validate_special_attribute_pool_within_budget_passes() {
        // Metatype B = 11 special points; Magician starting magic 3 (priority C);
        // spending 2 on magic (to 5) + 0 on edge = 2 ≤ 11 → no error.
        let mut draft = make_legal_human_draft();
        draft.priority_selection = Some(PrioritySelection {
            metatype: PriorityLevel::B,
            attributes: PriorityLevel::D,
            magic_or_resonance: PriorityLevel::C,
            skills: PriorityLevel::A,
            resources: PriorityLevel::E,
        });
        draft.magic_tradition = Some(MagicTradition::Magician);
        draft.attributes.magic = Some(5); // 2 above starting 3
        let errors = rules().validate_creation(&draft);
        let pool_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.field == "special_attribute_pool")
            .collect();
        assert!(
            pool_errors.is_empty(),
            "Should be within budget, got: {pool_errors:?}"
        );
    }

    #[test]
    fn validate_contacts_over_free_pool_flagged_as_warning() {
        // Free pool = Cha × 3 = 2 × 3 = 6 points.
        // Contact with connection 4 + loyalty 4 = 8 points → 2 points over free pool.
        let mut draft = make_legal_human_draft();
        draft.attributes.charisma = 2;
        draft.contacts.push(Contact {
            id: "c1".to_string(),
            name: "Fixer".to_string(),
            connection: 4,
            loyalty: 4,
            archetype: "Fixer".to_string(),
            notes: String::new(),
        });
        let errors = rules().validate_creation(&draft);
        assert!(
            errors.iter().any(|e| e.field == "contacts" && e.severity == ValidationSeverity::Warning),
            "Expected contacts overflow warning, got: {errors:?}"
        );
    }

    #[test]
    fn validate_contacts_within_free_pool_is_clean() {
        // Free pool = Cha × 3 = 3 × 3 = 9 points.
        // Contact with connection 4 + loyalty 4 = 8 ≤ 9 → no contacts warning.
        let mut draft = make_legal_human_draft();
        draft.attributes.charisma = 3;
        draft.contacts.push(Contact {
            id: "c1".to_string(),
            name: "Fixer".to_string(),
            connection: 4,
            loyalty: 4,
            archetype: "Fixer".to_string(),
            notes: String::new(),
        });
        let errors = rules().validate_creation(&draft);
        assert!(
            !errors.iter().any(|e| e.field == "contacts" && e.severity == ValidationSeverity::Warning),
            "Should be within free pool, got: {errors:?}"
        );
    }

    // -- apply_improvements tests --

    #[test]
    fn apply_improvements_basic() {
        let base = make_base_character();
        let imps = vec![Improvement::AttributeModifier {
            attribute: "body".to_string(),
            value: 2,
        }];
        let computed = rules().apply_improvements(&base, &imps);
        assert_eq!(computed.computed_attributes.body, base.attributes.body + 2);
    }

    #[test]
    fn apply_improvements_computes_derived_stats() {
        let base = make_base_character();
        let computed = rules().apply_improvements(&base, &[]);
        assert_eq!(
            computed.physical_condition_monitor,
            rules().physical_condition_monitor(base.attributes.body)
        );
        assert_eq!(
            computed.initiative,
            rules().initiative_score(base.attributes.reaction, base.attributes.intuition)
        );
    }

    // -- Canonical SR5 Adept test --

    #[test]
    fn canonical_sr5_adept() {
        // Priority-legal SR5 Human Adept build:
        //   A: Attributes (24 points)
        //   B: Magic (Adept, Magic 6)
        //   C: Skills (28/2)
        //   D: Metatype (Human, 0 special)
        //   E: Resources (6,000¥)
        let draft = CharacterDraft {
            name: "Adept".to_string(),
            edition: Edition::SR5,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 4,      // +3
                agility: 5,   // +4
                reaction: 4,  // +3
                strength: 3,  // +2
                willpower: 3, // +2
                logic: 2,     // +1
                intuition: 5, // +4
                charisma: 3,  // +2
                // Total: 3+4+3+2+2+1+4+2 = 21 (≤ 24)
                edge: 3,
                essence: 600,
                magic: Some(6),
                resonance: None,
            },
            skills: vec![
                make_skill("Unarmed Combat", "AGI", 6), // 6
                make_skill("Blades", "AGI", 5),         // 5
                make_skill("Sneaking", "AGI", 5),       // 5
                make_skill("Perception", "INT", 4),     // 4
                make_skill("Gymnastics", "AGI", 4),     // 4
                make_skill("Running", "STR", 2),        // 2
                make_skill("Pilot Ground Craft", "REA", 1), // 1
                                                        // Total: 27 (≤ 28)
            ],
            skill_groups: vec![],
            qualities: vec![
                make_quality("Agile Defender", QualityType::Positive, 10),
                make_quality("Mentor Spirit", QualityType::Positive, 5),
                make_quality("SINner (National)", QualityType::Negative, 5),
                make_quality("Code of Honor", QualityType::Negative, 15),
            ],
            augmentations: vec![],
            spells: vec![],
            adept_powers: vec![],
            complex_forms: vec![],
            contacts: vec![Contact {
                id: "c1".to_string(),
                name: "Sensei".to_string(),
                connection: 3,
                loyalty: 3,
                archetype: "Martial Arts Instructor".to_string(),
                notes: String::new(),
            }],
            weapons: vec![],
            armor: vec![],
            gear: vec![],
            vehicles: vec![],
            knowledge_skills: vec![],
            priority_selection: Some(PrioritySelection {
                metatype: PriorityLevel::D,
                attributes: PriorityLevel::A,
                magic_or_resonance: PriorityLevel::B,
                skills: PriorityLevel::C,
                resources: PriorityLevel::E,
            }),
            magic_tradition: Some(MagicTradition::Adept),
            tradition_name: None,
            creation_points_spent: 0,
            nuyen_spent: 5_000,
        };

        // Validate attribute points: 21 spent ≤ 24 (Priority A)
        let limits = rules().racial_limits(Metatype::Human);
        let attr_spent = sr5_priority::attribute_points_spent(&draft, &limits);
        assert_eq!(attr_spent, 21);

        // Validate skill points: 27 spent ≤ 28 (Priority C)
        let skill_spent = sr5_priority::skill_points_spent(&draft);
        assert_eq!(skill_spent, 27);

        // Validate — should have no errors
        let errors = rules().validate_creation(&draft);
        let real_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Error)
            .collect();
        assert!(
            real_errors.is_empty(),
            "Adept should be valid, got: {real_errors:?}"
        );

        // Create base character and compute
        let base = CharacterBase {
            id: "adept1".to_string(),
            campaign_id: "camp1".to_string(),
            name: draft.name.clone(),
            edition: Edition::SR5,
            metatype: draft.metatype,
            attributes: draft.attributes.clone(),
            skills: draft.skills.clone(),
            skill_groups: vec![],
            qualities: draft.qualities.clone(),
            augmentations: vec![],
            spells: vec![],
            adept_powers: vec![],
            complex_forms: vec![],
            contacts: draft.contacts.clone(),
            weapons: vec![],
            armor: vec![],
            gear: vec![],
            vehicles: vec![],
            knowledge_skills: vec![],
            tradition_name: None,
            notes: String::new(),
            priority_selection: draft.priority_selection.clone(),
            magic_tradition: draft.magic_tradition,
        };

        let computed = rules().apply_improvements(&base, &[]);

        // Verify derived stats
        assert_eq!(computed.physical_condition_monitor, 10); // 8 + ceil(4/2) = 10
        assert_eq!(computed.stun_condition_monitor, 10); // 8 + ceil(3/2) = 10
        assert_eq!(computed.initiative, 9); // REA 4 + INT 5 = 9
        assert_eq!(computed.initiative_dice, 1);
        assert_eq!(computed.computed_attributes.essence, 600); // No augmentations, full essence
        assert_eq!(computed.computed_attributes.body, 4);
        assert_eq!(computed.computed_attributes.agility, 5);
        assert_eq!(computed.computed_attributes.magic, Some(6));
    }

    // -- Test helpers --

    fn make_legal_human_draft() -> CharacterDraft {
        // A minimal valid SR5 human with Priority D/A/B/C/E
        CharacterDraft {
            name: "Legal Human".to_string(),
            edition: Edition::SR5,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 3,      // +2
                agility: 3,   // +2
                reaction: 3,  // +2
                strength: 2,  // +1
                willpower: 2, // +1
                logic: 2,     // +1
                intuition: 3, // +2
                charisma: 2,  // +1
                // Total: 12 (≤ 14 from Priority D)
                edge: 2,
                essence: 600,
                magic: None,
                resonance: None,
            },
            skills: vec![
                make_skill("Pistols", "AGI", 4),
                make_skill("Dodge", "REA", 3),
            ],
            // 7 ≤ 28 (Priority C)
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
            knowledge_skills: vec![],
            priority_selection: Some(PrioritySelection {
                metatype: PriorityLevel::D,
                attributes: PriorityLevel::B,
                magic_or_resonance: PriorityLevel::E,
                skills: PriorityLevel::C,
                resources: PriorityLevel::A,
            }),
            magic_tradition: None,
            tradition_name: None,
            creation_points_spent: 0,
            nuyen_spent: 5_000,
        }
    }

    fn make_base_character() -> CharacterBase {
        CharacterBase {
            id: "test".to_string(),
            campaign_id: "test".to_string(),
            name: "Test".to_string(),
            edition: Edition::SR5,
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
            knowledge_skills: vec![],
            tradition_name: None,
            notes: String::new(),
            priority_selection: None,
            magic_tradition: None,
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
            source: "SR5".to_string(),
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
            source: "SR5".to_string(),
            page: "1".to_string(),
            improvements: vec![],
        }
    }

    #[test]
    fn initiation_karma_cost_is_10_plus_grade_times_3() {
        let rules = rules();
        assert_eq!(rules.initiation_karma_cost(1), 13); // 10 + 1×3
        assert_eq!(rules.initiation_karma_cost(2), 16); // 10 + 2×3
        assert_eq!(rules.initiation_karma_cost(5), 25); // 10 + 5×3
    }

    #[test]
    fn submersion_karma_cost_is_10_plus_grade_times_3() {
        let rules = rules();
        assert_eq!(rules.submersion_karma_cost(1), 13);
        assert_eq!(rules.submersion_karma_cost(3), 19);
    }
}
