use crate::model::{
    attributes::Metatype,
    augmentations::{Augmentation, Essence},
    character::{CharacterBase, CharacterDraft, ComputedCharacter},
    improvements::Improvement,
    validation::ValidationError,
};

/// The core rules interface. SR4 and SR5 each provide an implementation.
/// This trait is the critical boundary: all game math lives behind it.
///
/// No I/O, no database, no UI — pure computation.
pub trait CharacterRules: Send + Sync {
    /// The creation method name (e.g. "Build Points" for SR4, "Priority" for SR5).
    fn creation_method(&self) -> &str;

    /// Karma cost to raise a skill from one rating to another.
    fn karma_cost_skill(&self, from: u8, to: u8) -> u32;

    /// Karma cost to raise an attribute from one rating to another.
    fn karma_cost_attribute(&self, from: u8, to: u8) -> u32;

    /// Calculate current essence given a list of augmentations.
    fn calculate_essence(&self, augmentations: &[Augmentation]) -> Essence;

    /// Validate a character draft during creation. Returns all errors and warnings.
    fn validate_creation(&self, draft: &CharacterDraft) -> Vec<ValidationError>;

    /// Apply improvements to a base character, producing a fully computed character.
    fn apply_improvements(
        &self,
        base: &CharacterBase,
        improvements: &[Improvement],
    ) -> ComputedCharacter;

    /// Physical condition monitor boxes: 8 + (Body / 2) rounded up.
    fn physical_condition_monitor(&self, body: u8) -> u8;

    /// Stun condition monitor boxes: 8 + (Willpower / 2) rounded up.
    fn stun_condition_monitor(&self, willpower: u8) -> u8;

    /// Base initiative: Reaction + Intuition.
    fn initiative_score(&self, reaction: u8, intuition: u8) -> i32;

    /// Base initiative dice (before augmentation bonuses).
    fn initiative_dice(&self) -> u8;

    /// Racial attribute limits for a given metatype.
    fn racial_limits(&self, metatype: Metatype) -> crate::model::attributes::RacialLimits;

    /// Maximum rating for a skill during creation.
    fn max_skill_rating_at_creation(&self) -> u8;

    /// Maximum rating for a skill during career play.
    fn max_skill_rating(&self) -> u8;
}
