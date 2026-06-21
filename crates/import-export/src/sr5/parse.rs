use serde::Deserialize;

/// Root element of a Chummer5a .chum5 character file.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "character", default)]
pub struct Chum5Character {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub metatype: String,
    #[serde(rename = "prioritymetatype", default)]
    pub priority_metatype: String,
    #[serde(rename = "priorityattributes", default)]
    pub priority_attributes: String,
    #[serde(rename = "priorityspecial", default)]
    pub priority_special: String,
    #[serde(rename = "priorityskills", default)]
    pub priority_skills: String,
    #[serde(rename = "priorityresources", default)]
    pub priority_resources: String,
    #[serde(default)]
    pub magenabled: String,
    #[serde(default)]
    pub adept: String,
    #[serde(default)]
    pub magician: String,
    #[serde(default)]
    pub technomancer: String,
    #[serde(default)]
    pub nuyen: String,
    #[serde(default)]
    pub karma: String,
    #[serde(default)]
    pub totalkarma: String,
    #[serde(default)]
    pub attributes: Chum5Attributes,
    #[serde(default)]
    pub newskills: Chum5NewSkills,
    #[serde(default)]
    pub qualities: Chum5QualityList,
    #[serde(default)]
    pub cyberwares: Chum5CyberwareList,
    #[serde(default)]
    pub spells: Chum5SpellList,
    #[serde(default)]
    pub powers: Chum5PowerList,
    #[serde(default)]
    pub complexforms: Chum5ComplexFormList,
    #[serde(default)]
    pub contacts: Chum5ContactList,
    #[serde(default)]
    pub weapons: Chum5WeaponList,
    #[serde(default)]
    pub armors: Chum5ArmorList,
    #[serde(default)]
    pub gears: Chum5GearList,
    #[serde(default)]
    pub vehicles: Chum5VehicleList,
}

// -- Attributes --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Attributes {
    #[serde(rename = "attribute", default)]
    pub items: Vec<Chum5Attribute>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Attribute {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub base: String,
    #[serde(default)]
    pub karma: String,
}

impl Chum5Attribute {
    pub fn total(&self) -> i32 {
        parse_int(&self.base) + parse_int(&self.karma)
    }
}

// -- Skills --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5NewSkills {
    #[serde(default)]
    pub skills: Chum5SkillList,
    #[serde(default)]
    pub knoskills: Chum5KnoSkillList,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5SkillList {
    #[serde(rename = "skill", default)]
    pub items: Vec<Chum5Skill>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Skill {
    #[serde(default)]
    pub suid: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub base: String,
    #[serde(default)]
    pub karma: String,
    /// Single specialization (legacy format).
    #[serde(default)]
    pub spec: String,
    /// Multiple specializations (newer format).
    #[serde(default)]
    pub specs: Chum5SpecList,
}

impl Chum5Skill {
    pub fn total_rating(&self) -> u8 {
        (parse_int(&self.base) + parse_int(&self.karma)).clamp(0, 12) as u8
    }

    pub fn spec_names(&self) -> Vec<String> {
        let mut specs: Vec<String> = self
            .specs
            .items
            .iter()
            .filter(|s| !s.name.is_empty())
            .map(|s| s.name.clone())
            .collect();
        if specs.is_empty() && !self.spec.is_empty() {
            specs.push(self.spec.clone());
        }
        specs
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5SpecList {
    #[serde(rename = "spec", default)]
    pub items: Vec<Chum5Spec>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Spec {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5KnoSkillList {
    #[serde(rename = "skill", default)]
    pub items: Vec<Chum5KnoSkill>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5KnoSkill {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub base: String,
    #[serde(default)]
    pub karma: String,
    #[serde(default)]
    pub category: String,
}

impl Chum5KnoSkill {
    pub fn total_rating(&self) -> u8 {
        (parse_int(&self.base) + parse_int(&self.karma)).clamp(0, 12) as u8
    }
}

// -- Qualities --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5QualityList {
    #[serde(rename = "quality", default)]
    pub items: Vec<Chum5Quality>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Quality {
    #[serde(default)]
    pub name: String,
    /// "Positive" or "Negative"
    #[serde(rename = "type", default)]
    pub quality_type: String,
    #[serde(default)]
    pub bp: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Cyberware/Bioware --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5CyberwareList {
    #[serde(rename = "cyberware", default)]
    pub items: Vec<Chum5Cyberware>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Cyberware {
    #[serde(default)]
    pub name: String,
    /// "Cyberware" or "Bioware"
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub grade: String,
    /// Essence cost as decimal string (e.g. "2.00").
    #[serde(default)]
    pub ess: String,
    #[serde(default)]
    pub cost: String,
    #[serde(rename = "avail", default)]
    pub availability: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Spells --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5SpellList {
    #[serde(rename = "spell", default)]
    pub items: Vec<Chum5Spell>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Spell {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    /// "P" = Physical, "M" = Mana
    #[serde(rename = "type", default)]
    pub spell_type: String,
    #[serde(default)]
    pub range: String,
    #[serde(default)]
    pub damage: String,
    #[serde(default)]
    pub duration: String,
    #[serde(default)]
    pub drain: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Adept Powers --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5PowerList {
    #[serde(rename = "power", default)]
    pub items: Vec<Chum5Power>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Power {
    #[serde(default)]
    pub name: String,
    /// PP cost per level as decimal string (e.g. "0.25").
    #[serde(default)]
    pub pointsperlevel: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Complex Forms --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5ComplexFormList {
    #[serde(rename = "complexform", default)]
    pub items: Vec<Chum5ComplexForm>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5ComplexForm {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub duration: String,
    /// Fading value (field name is "fv" in .chum5).
    #[serde(default)]
    pub fv: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Contacts --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5ContactList {
    #[serde(rename = "contact", default)]
    pub items: Vec<Chum5Contact>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Contact {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub connection: String,
    #[serde(default)]
    pub loyalty: String,
}

// -- Weapons --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5WeaponList {
    #[serde(rename = "weapon", default)]
    pub items: Vec<Chum5Weapon>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Weapon {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub damage: String,
    #[serde(default)]
    pub ap: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub ammo: String,
    #[serde(default)]
    pub cost: String,
    #[serde(rename = "avail", default)]
    pub availability: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Armor --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5ArmorList {
    #[serde(rename = "armor", default)]
    pub items: Vec<Chum5Armor>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Armor {
    #[serde(default)]
    pub name: String,
    /// Armor value (field name matches element name "armor").
    #[serde(rename = "armor", default)]
    pub armor_value: String,
    #[serde(default)]
    pub cost: String,
    #[serde(rename = "avail", default)]
    pub availability: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Gear --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5GearList {
    #[serde(rename = "gear", default)]
    pub items: Vec<Chum5Gear>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Gear {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub rating: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Vehicles --

#[derive(Debug, Default, Deserialize)]
pub struct Chum5VehicleList {
    #[serde(rename = "vehicle", default)]
    pub items: Vec<Chum5Vehicle>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Chum5Vehicle {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub handling: String,
    #[serde(default)]
    pub speed: String,
    #[serde(default)]
    pub acceleration: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub pilot: String,
    #[serde(default)]
    pub sensor: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub page: String,
}

// -- Helpers --

pub fn parse_int(s: &str) -> i32 {
    s.trim().parse().unwrap_or(0)
}

pub fn parse_float(s: &str) -> f64 {
    s.trim().parse().unwrap_or(0.0)
}

pub fn parse_bool(s: &str) -> bool {
    matches!(s.trim().to_lowercase().as_str(), "true" | "1" | "yes")
}
