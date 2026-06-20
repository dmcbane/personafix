pub mod sr4;
pub mod sr5;

use serde::Deserialize;

/// Parsed game data from a single edition's XML files.
#[derive(Debug, Default)]
pub struct ParsedGameData {
    pub edition: String,
    pub books: Vec<ParsedBook>,
    pub metatypes: Vec<ParsedMetatype>,
    pub skills: Vec<ParsedSkill>,
    pub qualities: Vec<ParsedQuality>,
    pub weapons: Vec<ParsedWeapon>,
    pub armor: Vec<ParsedArmor>,
    pub augmentations: Vec<ParsedAugmentation>,
    pub spells: Vec<ParsedSpell>,
    pub adept_powers: Vec<ParsedAdeptPower>,
    pub complex_forms: Vec<ParsedComplexForm>,
}

// -- XML root wrappers --
// Each Chummer XML file has <chummer> root with a collection element.

#[derive(Debug, Deserialize)]
pub struct ChummerBooks {
    #[serde(default)]
    pub books: Books,
}

#[derive(Debug, Default, Deserialize)]
pub struct Books {
    #[serde(rename = "book", default)]
    pub items: Vec<XmlBook>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerMetatypes {
    #[serde(default)]
    pub metatypes: Metatypes,
}

#[derive(Debug, Default, Deserialize)]
pub struct Metatypes {
    #[serde(rename = "metatype", default)]
    pub items: Vec<XmlMetatype>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerSkills {
    #[serde(default)]
    pub skills: Skills,
}

#[derive(Debug, Default, Deserialize)]
pub struct Skills {
    #[serde(rename = "skill", default)]
    pub items: Vec<XmlSkill>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerQualities {
    #[serde(default)]
    pub qualities: Qualities,
}

#[derive(Debug, Default, Deserialize)]
pub struct Qualities {
    #[serde(rename = "quality", default)]
    pub items: Vec<XmlQuality>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerWeapons {
    #[serde(default)]
    pub weapons: Weapons,
}

#[derive(Debug, Default, Deserialize)]
pub struct Weapons {
    #[serde(rename = "weapon", default)]
    pub items: Vec<XmlWeapon>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerArmor {
    #[serde(default)]
    pub armors: Armors,
}

#[derive(Debug, Default, Deserialize)]
pub struct Armors {
    #[serde(rename = "armor", default)]
    pub items: Vec<XmlArmorItem>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerCyberware {
    #[serde(default)]
    pub cyberwares: Cyberwares,
}

#[derive(Debug, Default, Deserialize)]
pub struct Cyberwares {
    #[serde(rename = "cyberware", default)]
    pub items: Vec<XmlCyberware>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerBioware {
    #[serde(default)]
    pub biowares: Biowares,
}

#[derive(Debug, Default, Deserialize)]
pub struct Biowares {
    #[serde(rename = "bioware", default)]
    pub items: Vec<XmlCyberware>,
}

#[derive(Debug, Deserialize)]
pub struct ChummerSpells {
    #[serde(default)]
    pub spells: Spells,
}

#[derive(Debug, Default, Deserialize)]
pub struct Spells {
    #[serde(rename = "spell", default)]
    pub items: Vec<XmlSpell>,
}

// -- Individual XML element types --

#[derive(Debug, Clone, Deserialize)]
pub struct XmlBook {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct XmlMetatype {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    // SR5 uses <karma>, SR4 uses <bp>
    #[serde(default)]
    pub karma: String,
    #[serde(default)]
    pub bp: String,
    // Body
    #[serde(default)]
    pub bodmin: String,
    #[serde(default)]
    pub bodmax: String,
    // Agility
    #[serde(default)]
    pub agimin: String,
    #[serde(default)]
    pub agimax: String,
    // Reaction
    #[serde(default)]
    pub reamin: String,
    #[serde(default)]
    pub reamax: String,
    // Strength
    #[serde(default)]
    pub strmin: String,
    #[serde(default)]
    pub strmax: String,
    // Charisma
    #[serde(default)]
    pub chamin: String,
    #[serde(default)]
    pub chamax: String,
    // Intuition
    #[serde(default)]
    pub intmin: String,
    #[serde(default)]
    pub intmax: String,
    // Logic
    #[serde(default)]
    pub logmin: String,
    #[serde(default)]
    pub logmax: String,
    // Willpower
    #[serde(default)]
    pub wilmin: String,
    #[serde(default)]
    pub wilmax: String,
    // Edge
    #[serde(default)]
    pub edgmin: String,
    #[serde(default)]
    pub edgmax: String,
    // Source
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct XmlSkill {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub attribute: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub skillgroup: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlQuality {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    // SR5 uses <karma>, SR4 uses <bp>
    #[serde(default)]
    pub karma: String,
    #[serde(default)]
    pub bp: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct XmlWeapon {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(rename = "type", default)]
    pub weapon_type: String,
    #[serde(default)]
    pub damage: String,
    #[serde(default)]
    pub ap: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub rc: String,
    #[serde(default)]
    pub ammo: String,
    #[serde(default)]
    pub avail: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct XmlArmorItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    /// In Chummer5a this is the <armor> child element (armor value).
    #[serde(rename = "armor", default)]
    pub armor_value: String,
    #[serde(default)]
    pub avail: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct XmlCyberware {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub ess: String,
    #[serde(default)]
    pub capacity: String,
    #[serde(default)]
    pub avail: String,
    #[serde(default)]
    pub cost: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlSpell {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    /// P = Physical, M = Mana
    #[serde(rename = "type", default)]
    pub spell_type: String,
    #[serde(default)]
    pub range: String,
    #[serde(default)]
    pub damage: String,
    #[serde(default)]
    pub duration: String,
    /// Drain value (SR5 uses <dv>)
    #[serde(default)]
    pub dv: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Parsed output types (edition-tagged, ready for DB insertion) --

#[derive(Debug, Clone)]
pub struct ParsedBook {
    pub id: String,
    pub name: String,
    pub abbreviation: String,
}

#[derive(Debug, Clone)]
pub struct ParsedMetatype {
    pub id: String,
    pub name: String,
    pub body_min: i32,
    pub body_max: i32,
    pub agility_min: i32,
    pub agility_max: i32,
    pub reaction_min: i32,
    pub reaction_max: i32,
    pub strength_min: i32,
    pub strength_max: i32,
    pub willpower_min: i32,
    pub willpower_max: i32,
    pub logic_min: i32,
    pub logic_max: i32,
    pub intuition_min: i32,
    pub intuition_max: i32,
    pub charisma_min: i32,
    pub charisma_max: i32,
    pub edge_min: i32,
    pub edge_max: i32,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedSkill {
    pub id: String,
    pub name: String,
    pub linked_attribute: String,
    pub skill_group: Option<String>,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedQuality {
    pub id: String,
    pub name: String,
    pub quality_type: String,
    pub cost: i32,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedWeapon {
    pub id: String,
    pub name: String,
    pub category: String,
    pub damage: String,
    pub ap: String,
    pub mode: String,
    pub recoil_comp: String,
    pub ammo: String,
    pub availability: String,
    pub cost: String,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedArmor {
    pub id: String,
    pub name: String,
    pub armor_value: String,
    pub availability: String,
    pub cost: String,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedAugmentation {
    pub id: String,
    pub name: String,
    pub augmentation_type: String,
    pub essence_cost: String,
    pub capacity: String,
    pub availability: String,
    pub cost: String,
    pub source: String,
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedSpell {
    pub id: String,
    pub name: String,
    pub category: String,
    pub spell_type: String,
    pub range: String,
    pub damage: String,
    pub duration: String,
    pub drain: String,
    pub source: String,
    pub page: String,
}

/// Complex forms XML wrapper.
#[derive(Debug, Deserialize)]
pub struct ChummerComplexForms {
    #[serde(default)]
    pub complexforms: ComplexFormItems,
}

#[derive(Debug, Default, Deserialize)]
pub struct ComplexFormItems {
    #[serde(rename = "complexform", default)]
    pub items: Vec<XmlComplexForm>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlComplexForm {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub duration: String,
    /// Fading value (e.g. "L-2", "L+2").
    #[serde(rename = "fv", default)]
    pub fading: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedComplexForm {
    pub id: String,
    pub name: String,
    pub target: String,
    pub duration: String,
    pub fading: String,
    pub source: String,
    pub page: String,
}

/// Adept powers XML wrapper.
#[derive(Debug, Deserialize)]
pub struct ChummerPowers {
    #[serde(default)]
    pub powers: Powers,
}

#[derive(Debug, Default, Deserialize)]
pub struct Powers {
    #[serde(rename = "power", default)]
    pub items: Vec<XmlPower>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XmlPower {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// Power point cost as decimal string (e.g. "0.25", "0.5", "1").
    #[serde(default)]
    pub points: String,
    /// Whether the power can be taken multiple times ("True"/"False").
    #[serde(default)]
    pub levels: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

#[derive(Debug, Clone)]
pub struct ParsedAdeptPower {
    pub id: String,
    pub name: String,
    /// Cost in hundredths (25 = 0.25 PP).
    pub cost: i32,
    pub levels: bool,
    pub source: String,
    pub page: String,
}

/// Parse a string as i32, defaulting to 0 for empty or invalid values.
pub fn parse_int(s: &str) -> i32 {
    s.trim().parse().unwrap_or(0)
}

/// Parse a power point decimal string to hundredths integer (e.g. "0.25" → 25).
pub fn parse_power_cost(s: &str) -> i32 {
    let v: f64 = s.trim().parse().unwrap_or(0.0);
    (v * 100.0).round() as i32
}
