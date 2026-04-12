use crate::model::{
    attributes::{Metatype, RacialLimits},
    augmentations::{Augmentation, Essence},
    character::{CharacterBase, CharacterDraft, ComputedCharacter},
    edition::Edition,
    improvements::Improvement,
    validation::ValidationError,
};

use super::traits::CharacterRules;

/// SR5 rules implementation using the Priority system.
pub struct SR5Rules;

impl CharacterRules for SR5Rules {
    fn creation_method(&self) -> &str {
        "Priority"
    }

    fn karma_cost_skill(&self, _from: u8, _to: u8) -> u32 {
        todo!("SR5 karma cost for skill advancement")
    }

    fn karma_cost_attribute(&self, _from: u8, _to: u8) -> u32 {
        todo!("SR5 karma cost for attribute advancement")
    }

    fn calculate_essence(&self, _augmentations: &[Augmentation]) -> Essence {
        todo!("SR5 essence calculation with grade modifiers")
    }

    fn validate_creation(&self, _draft: &CharacterDraft) -> Vec<ValidationError> {
        todo!("SR5 priority validation — each priority used exactly once")
    }

    fn apply_improvements(
        &self,
        _base: &CharacterBase,
        _improvements: &[Improvement],
    ) -> ComputedCharacter {
        todo!("SR5 improvement resolution")
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
            _ => todo!("SR5 racial limits for {metatype:?}"),
        }
    }

    fn max_skill_rating_at_creation(&self) -> u8 {
        6
    }

    fn max_skill_rating(&self) -> u8 {
        12
    }
}
