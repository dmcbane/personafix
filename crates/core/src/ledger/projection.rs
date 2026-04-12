//! Projection logic: replays a sequence of LedgerEvents against a CharacterBase
//! to produce a ComputedCharacter representing the current state.
//!
//! This is the core of the append-only ledger model: the character's current
//! sheet is always a projection of the creation base + all career events.

use crate::model::{
    character::{CharacterBase, ComputedCharacter},
    contacts::Contact,
    gear::GearItem,
    qualities::{Quality, QualityType},
};
use crate::rules::traits::CharacterRules;

use super::events::LedgerEvent;

/// Project a character's current state by replaying all ledger events against
/// the creation base, then computing derived stats via the rules engine.
pub fn project(
    base: &CharacterBase,
    events: &[LedgerEvent],
    rules: &dyn CharacterRules,
) -> ComputedCharacter {
    let mut working = base.clone();
    let mut karma_earned: i32 = 0;
    let mut karma_spent: i32 = 0;
    let mut nuyen: i64 = 0;

    for event in events {
        match event {
            LedgerEvent::RunCompleted { .. } => {
                // Informational only — karma/nuyen come from separate events
            }

            LedgerEvent::KarmaReceived { amount, .. } => {
                karma_earned += amount;
            }

            LedgerEvent::KarmaSpent { amount, .. } => {
                karma_spent += amount;
            }

            LedgerEvent::NuyenReceived { amount, .. } => {
                nuyen += amount;
            }

            LedgerEvent::NuyenSpent { amount, .. } => {
                nuyen -= amount;
            }

            LedgerEvent::SkillImproved {
                skill_name,
                to,
                karma_cost,
                ..
            } => {
                if let Some(skill) = working.skills.iter_mut().find(|s| s.name == *skill_name) {
                    skill.rating = *to;
                }
                karma_spent += karma_cost;
            }

            LedgerEvent::AttributeImproved {
                attribute,
                to,
                karma_cost,
                ..
            } => {
                apply_attribute_change(&mut working, attribute, *to);
                karma_spent += karma_cost;
            }

            LedgerEvent::GearAcquired {
                item_id,
                item_name,
                cost,
            } => {
                working.gear.push(GearItem {
                    id: item_id.clone(),
                    name: item_name.clone(),
                    category: String::new(),
                    rating: None,
                    availability: String::new(),
                    cost: *cost,
                    source: String::new(),
                    page: String::new(),
                });
                nuyen -= cost;
            }

            LedgerEvent::GearLost { item_id, .. } => {
                working.gear.retain(|g| g.id != *item_id);
            }

            LedgerEvent::ContactAdded {
                contact_id,
                name,
                connection,
                loyalty,
            } => {
                working.contacts.push(Contact {
                    id: contact_id.clone(),
                    name: name.clone(),
                    connection: *connection,
                    loyalty: *loyalty,
                    archetype: String::new(),
                    notes: String::new(),
                });
            }

            LedgerEvent::ContactChanged {
                contact_id,
                new_connection,
                new_loyalty,
            } => {
                if let Some(contact) = working.contacts.iter_mut().find(|c| c.id == *contact_id) {
                    contact.connection = *new_connection;
                    contact.loyalty = *new_loyalty;
                }
            }

            LedgerEvent::ContactLost { contact_id, .. } => {
                working.contacts.retain(|c| c.id != *contact_id);
            }

            LedgerEvent::Initiated {
                new_grade,
                karma_cost,
            } => {
                // Increase magic if present (initiation grants +1 Magic per grade)
                if let Some(ref mut mag) = working.attributes.magic {
                    *mag = (*mag).max(*new_grade);
                }
                karma_spent += karma_cost;
            }

            LedgerEvent::Submerged {
                new_grade,
                karma_cost,
            } => {
                if let Some(ref mut res) = working.attributes.resonance {
                    *res = (*res).max(*new_grade);
                }
                karma_spent += karma_cost;
            }

            LedgerEvent::QualityAdded {
                quality_id,
                quality_name,
                karma_cost,
            } => {
                working.qualities.push(Quality {
                    id: quality_id.clone(),
                    name: quality_name.clone(),
                    quality_type: QualityType::Positive,
                    cost: *karma_cost,
                    source: String::new(),
                    page: String::new(),
                    improvements: vec![],
                    incompatible_with: vec![],
                });
                karma_spent += karma_cost;
            }

            LedgerEvent::QualityRemoved {
                quality_id,
                karma_cost,
                ..
            } => {
                working.qualities.retain(|q| q.id != *quality_id);
                karma_spent += karma_cost;
            }
        }
    }

    // Compute derived stats using the rules engine
    let mut computed = rules.apply_improvements(&working, &[]);
    computed.total_karma_earned = karma_earned;
    computed.total_karma_spent = karma_spent;
    computed.nuyen = nuyen;

    computed
}

fn apply_attribute_change(base: &mut CharacterBase, attribute: &str, new_value: u8) {
    match attribute.to_lowercase().as_str() {
        "body" | "bod" => base.attributes.body = new_value,
        "agility" | "agi" => base.attributes.agility = new_value,
        "reaction" | "rea" => base.attributes.reaction = new_value,
        "strength" | "str" => base.attributes.strength = new_value,
        "willpower" | "wil" => base.attributes.willpower = new_value,
        "logic" | "log" => base.attributes.logic = new_value,
        "intuition" | "int" => base.attributes.intuition = new_value,
        "charisma" | "cha" => base.attributes.charisma = new_value,
        "edge" | "edg" => base.attributes.edge = new_value,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        attributes::{Attributes, Metatype},
        edition::Edition,
        skills::Skill,
    };
    use crate::rules::sr4::SR4Rules;

    fn make_base() -> CharacterBase {
        CharacterBase {
            id: "char1".to_string(),
            campaign_id: "camp1".to_string(),
            name: "Test Runner".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 4,
                agility: 5,
                reaction: 4,
                strength: 3,
                willpower: 3,
                logic: 3,
                intuition: 4,
                charisma: 2,
                edge: 3,
                essence: 600,
                magic: None,
                resonance: None,
            },
            skills: vec![
                Skill {
                    id: "pistols".to_string(),
                    name: "Pistols".to_string(),
                    linked_attribute: "AGI".to_string(),
                    group: None,
                    rating: 4,
                    specializations: vec![],
                },
                Skill {
                    id: "dodge".to_string(),
                    name: "Dodge".to_string(),
                    linked_attribute: "REA".to_string(),
                    group: None,
                    rating: 3,
                    specializations: vec![],
                },
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
        }
    }

    #[test]
    fn project_empty_ledger() {
        let base = make_base();
        let rules = SR4Rules;
        let computed = project(&base, &[], &rules);
        assert_eq!(computed.total_karma_earned, 0);
        assert_eq!(computed.total_karma_spent, 0);
        assert_eq!(computed.nuyen, 0);
        assert_eq!(computed.computed_attributes.body, 4);
    }

    #[test]
    fn project_karma_earned_and_spent() {
        let base = make_base();
        let events = vec![
            LedgerEvent::KarmaReceived {
                amount: 10,
                reason: "Run reward".to_string(),
                run_id: Some("run1".to_string()),
            },
            LedgerEvent::KarmaSpent {
                amount: 6,
                description: "Bought stuff".to_string(),
            },
        ];
        let computed = project(&base, &events, &SR4Rules);
        assert_eq!(computed.total_karma_earned, 10);
        assert_eq!(computed.total_karma_spent, 6);
    }

    #[test]
    fn project_nuyen_flow() {
        let base = make_base();
        let events = vec![
            LedgerEvent::NuyenReceived {
                amount: 20_000,
                reason: "Run payment".to_string(),
                run_id: None,
            },
            LedgerEvent::NuyenSpent {
                amount: 5_000,
                description: "Ammo".to_string(),
            },
        ];
        let computed = project(&base, &events, &SR4Rules);
        assert_eq!(computed.nuyen, 15_000);
    }

    #[test]
    fn project_skill_improved() {
        let base = make_base();
        let events = vec![LedgerEvent::SkillImproved {
            skill_name: "Pistols".to_string(),
            from: 4,
            to: 5,
            karma_cost: 10,
        }];
        let computed = project(&base, &events, &SR4Rules);
        // The working base's Pistols skill should now be 5
        let pistols = computed
            .base
            .skills
            .iter()
            .find(|s| s.name == "Pistols")
            .unwrap();
        assert_eq!(pistols.rating, 5);
        assert_eq!(computed.total_karma_spent, 10);
    }

    #[test]
    fn project_attribute_improved() {
        let base = make_base();
        let events = vec![LedgerEvent::AttributeImproved {
            attribute: "body".to_string(),
            from: 4,
            to: 5,
            karma_cost: 25,
        }];
        let computed = project(&base, &events, &SR4Rules);
        assert_eq!(computed.computed_attributes.body, 5);
        assert_eq!(computed.total_karma_spent, 25);
        // Body 5 → physical CM = 8 + ceil(5/2) = 11
        assert_eq!(computed.physical_condition_monitor, 11);
    }

    #[test]
    fn project_gear_acquired_and_lost() {
        let base = make_base();
        let events = vec![
            LedgerEvent::GearAcquired {
                item_id: "g1".to_string(),
                item_name: "Medkit".to_string(),
                cost: 500,
            },
            LedgerEvent::GearAcquired {
                item_id: "g2".to_string(),
                item_name: "Ammo".to_string(),
                cost: 100,
            },
            LedgerEvent::GearLost {
                item_id: "g1".to_string(),
                item_name: "Medkit".to_string(),
            },
        ];
        let computed = project(&base, &events, &SR4Rules);
        assert_eq!(computed.base.gear.len(), 1);
        assert_eq!(computed.base.gear[0].name, "Ammo");
        assert_eq!(computed.nuyen, -600); // Spent 500 + 100, no income
    }

    #[test]
    fn project_contact_lifecycle() {
        let base = make_base();
        let events = vec![
            LedgerEvent::ContactAdded {
                contact_id: "c1".to_string(),
                name: "Fixer".to_string(),
                connection: 3,
                loyalty: 2,
            },
            LedgerEvent::ContactChanged {
                contact_id: "c1".to_string(),
                new_connection: 4,
                new_loyalty: 3,
            },
            LedgerEvent::ContactAdded {
                contact_id: "c2".to_string(),
                name: "Street Doc".to_string(),
                connection: 2,
                loyalty: 1,
            },
            LedgerEvent::ContactLost {
                contact_id: "c2".to_string(),
                reason: "Burned".to_string(),
            },
        ];
        let computed = project(&base, &events, &SR4Rules);
        assert_eq!(computed.base.contacts.len(), 1);
        let fixer = &computed.base.contacts[0];
        assert_eq!(fixer.name, "Fixer");
        assert_eq!(fixer.connection, 4);
        assert_eq!(fixer.loyalty, 3);
    }

    #[test]
    fn project_full_career_three_runs() {
        let base = make_base();
        let events = vec![
            // Run 1: milk run
            LedgerEvent::RunCompleted {
                run_id: "run1".to_string(),
                name: "Milk Run".to_string(),
                date: "2078-01-15".to_string(),
                notes: "Quick courier job".to_string(),
            },
            LedgerEvent::KarmaReceived {
                amount: 5,
                reason: "Run reward".to_string(),
                run_id: Some("run1".to_string()),
            },
            LedgerEvent::NuyenReceived {
                amount: 8_000,
                reason: "Run payment".to_string(),
                run_id: Some("run1".to_string()),
            },
            // Spend karma: improve Dodge 3→4
            LedgerEvent::SkillImproved {
                skill_name: "Dodge".to_string(),
                from: 3,
                to: 4,
                karma_cost: 8,
            },
            // Run 2: wetwork
            LedgerEvent::RunCompleted {
                run_id: "run2".to_string(),
                name: "Wetwork".to_string(),
                date: "2078-02-10".to_string(),
                notes: "Messy".to_string(),
            },
            LedgerEvent::KarmaReceived {
                amount: 8,
                reason: "Run reward".to_string(),
                run_id: Some("run2".to_string()),
            },
            LedgerEvent::NuyenReceived {
                amount: 15_000,
                reason: "Run payment".to_string(),
                run_id: Some("run2".to_string()),
            },
            // Buy gear
            LedgerEvent::NuyenSpent {
                amount: 3_000,
                description: "Armor vest".to_string(),
            },
            // Add a contact
            LedgerEvent::ContactAdded {
                contact_id: "c1".to_string(),
                name: "Fixer".to_string(),
                connection: 3,
                loyalty: 2,
            },
            // Improve Body 4→5
            LedgerEvent::AttributeImproved {
                attribute: "body".to_string(),
                from: 4,
                to: 5,
                karma_cost: 25,
            },
            // Run 3: heist
            LedgerEvent::RunCompleted {
                run_id: "run3".to_string(),
                name: "Heist".to_string(),
                date: "2078-03-20".to_string(),
                notes: "Big score".to_string(),
            },
            LedgerEvent::KarmaReceived {
                amount: 10,
                reason: "Run reward".to_string(),
                run_id: Some("run3".to_string()),
            },
            LedgerEvent::NuyenReceived {
                amount: 25_000,
                reason: "Run payment".to_string(),
                run_id: Some("run3".to_string()),
            },
        ];

        let computed = project(&base, &events, &SR4Rules);

        // Karma: earned 5+8+10=23, spent 8 (skill) + 25 (attr) = 33
        assert_eq!(computed.total_karma_earned, 23);
        assert_eq!(computed.total_karma_spent, 33);

        // Nuyen: earned 8k+15k+25k=48k, spent 3k
        assert_eq!(computed.nuyen, 45_000);

        // Skills: Dodge should be 4
        let dodge = computed
            .base
            .skills
            .iter()
            .find(|s| s.name == "Dodge")
            .unwrap();
        assert_eq!(dodge.rating, 4);

        // Attributes: Body should be 5 → physical CM = 11
        assert_eq!(computed.computed_attributes.body, 5);
        assert_eq!(computed.physical_condition_monitor, 11);

        // Contact
        assert_eq!(computed.base.contacts.len(), 1);
        assert_eq!(computed.base.contacts[0].name, "Fixer");

        // Initiative: REA 4 + INT 4 = 8
        assert_eq!(computed.initiative, 8);
    }
}
