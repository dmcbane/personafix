//! Modifier stacker: combines multiple improvements targeting the same stat
//! into final computed values, respecting stacking rules.

use crate::model::{attributes::Attributes, improvements::Improvement};

/// Apply a list of improvements to base attributes, returning computed attributes.
pub fn apply_improvements_to_attributes(
    base: &Attributes,
    improvements: &[Improvement],
) -> Attributes {
    let mut computed = base.clone();

    for imp in improvements {
        if let Improvement::AttributeModifier { attribute, value } = imp {
            match attribute.to_lowercase().as_str() {
                "body" | "bod" => computed.body = add_to_u8(computed.body, *value),
                "agility" | "agi" => computed.agility = add_to_u8(computed.agility, *value),
                "reaction" | "rea" => computed.reaction = add_to_u8(computed.reaction, *value),
                "strength" | "str" => computed.strength = add_to_u8(computed.strength, *value),
                "willpower" | "wil" => computed.willpower = add_to_u8(computed.willpower, *value),
                "logic" | "log" => computed.logic = add_to_u8(computed.logic, *value),
                "intuition" | "int" => computed.intuition = add_to_u8(computed.intuition, *value),
                "charisma" | "cha" => computed.charisma = add_to_u8(computed.charisma, *value),
                "edge" | "edg" => computed.edge = add_to_u8(computed.edge, *value),
                _ => {}
            }
        }
    }

    computed
}

/// Safely add an i32 modifier to a u8 attribute, clamping to [0, 255].
fn add_to_u8(base: u8, modifier: i32) -> u8 {
    let result = base as i32 + modifier;
    result.clamp(0, 255) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_single_attribute_modifier() {
        let base = Attributes {
            body: 3,
            ..Default::default()
        };
        let improvements = vec![Improvement::AttributeModifier {
            attribute: "body".to_string(),
            value: 2,
        }];
        let computed = apply_improvements_to_attributes(&base, &improvements);
        assert_eq!(computed.body, 5);
    }

    #[test]
    fn apply_multiple_modifiers_stack() {
        let base = Attributes {
            agility: 4,
            ..Default::default()
        };
        let improvements = vec![
            Improvement::AttributeModifier {
                attribute: "agility".to_string(),
                value: 1,
            },
            Improvement::AttributeModifier {
                attribute: "agility".to_string(),
                value: 2,
            },
        ];
        let computed = apply_improvements_to_attributes(&base, &improvements);
        assert_eq!(computed.agility, 7);
    }

    #[test]
    fn apply_negative_modifier_clamps_to_zero() {
        let base = Attributes {
            charisma: 2,
            ..Default::default()
        };
        let improvements = vec![Improvement::AttributeModifier {
            attribute: "charisma".to_string(),
            value: -5,
        }];
        let computed = apply_improvements_to_attributes(&base, &improvements);
        assert_eq!(computed.charisma, 0);
    }

    #[test]
    fn non_attribute_improvements_ignored() {
        let base = Attributes {
            body: 3,
            ..Default::default()
        };
        let improvements = vec![Improvement::InitiativeDice { value: 2 }];
        let computed = apply_improvements_to_attributes(&base, &improvements);
        assert_eq!(computed.body, 3);
    }
}
