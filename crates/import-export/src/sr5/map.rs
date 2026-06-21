use std::collections::HashMap;

use sqlx::SqlitePool;

use personafix_core::model::{
    attributes::{Attributes, Metatype},
    augmentations::{Augmentation, AugmentationGrade, AugmentationType},
    character::CharacterBase,
    contacts::Contact,
    edition::Edition,
    gear::{Armor, GearItem, Vehicle, Weapon},
    magic::{AdeptPower, ComplexForm, MagicTradition, Spell, SpellCategory, SpellType},
    priority::{PriorityLevel, PrioritySelection},
    qualities::{Quality, QualityType},
    skills::{KnowledgeSkill, Skill, Specialization},
};

use crate::error::ImportResult;
use super::parse::{parse_bool, parse_float, parse_int, Chum5Character};

pub async fn import_sr5(
    chum: &Chum5Character,
    pool: &SqlitePool,
    campaign_id: &str,
    character_id: &str,
) -> ImportResult<CharacterBase> {
    let game_skills = load_skills(pool, "SR5").await?;
    let game_qualities = load_qualities(pool, "SR5").await?;

    let metatype = parse_metatype(&chum.metatype);
    let attributes = map_attributes(&chum.attributes.items, &metatype);
    let magic_tradition = detect_tradition(chum);

    let skills = map_skills(&chum.newskills.skills.items, &game_skills);
    let knowledge_skills = map_kno_skills(&chum.newskills.knoskills.items);
    let qualities = map_qualities(&chum.qualities.items, &game_qualities);
    let augmentations = map_augmentations(&chum.cyberwares.items);
    let spells = map_spells(&chum.spells.items);
    let adept_powers = map_powers(&chum.powers.items);
    let complex_forms = map_complex_forms(&chum.complexforms.items);
    let contacts = map_contacts(&chum.contacts.items);
    let weapons = map_weapons(&chum.weapons.items);
    let armor = map_armor(&chum.armors.items);
    let gear = map_gear(&chum.gears.items);
    let vehicles = map_vehicles(&chum.vehicles.items);
    let priority_selection = map_priority(chum);

    Ok(CharacterBase {
        id: character_id.to_string(),
        campaign_id: campaign_id.to_string(),
        name: if chum.name.is_empty() { "Imported Character".to_string() } else { chum.name.clone() },
        edition: Edition::SR5,
        metatype,
        attributes,
        skills,
        skill_groups: vec![],
        knowledge_skills,
        qualities,
        augmentations,
        spells,
        adept_powers,
        complex_forms,
        contacts,
        weapons,
        armor,
        gear,
        vehicles,
        priority_selection,
        magic_tradition,
        tradition_name: None,
        notes: String::new(),
    })
}

// -- Attribute mapping --

fn map_attributes(attrs: &[super::parse::Chum5Attribute], metatype: &Metatype) -> Attributes {
    let mut map: HashMap<&str, i32> = HashMap::new();
    for a in attrs {
        map.insert(a.name.as_str(), a.total());
    }
    let get = |key: &str| map.get(key).copied().unwrap_or(1).clamp(1, 12) as u8;
    let get_opt = |key: &str| {
        let v = map.get(key).copied().unwrap_or(0);
        if v > 0 { Some(v.clamp(1, 12) as u8) } else { None }
    };
    let _ = metatype; // metatype-specific minimums are enforced at creation
    Attributes {
        body: get("BOD"),
        agility: get("AGI"),
        reaction: get("REA"),
        strength: get("STR"),
        willpower: get("WIL"),
        logic: get("LOG"),
        intuition: get("INT"),
        charisma: get("CHA"),
        edge: get("EDG"),
        essence: 600,
        magic: get_opt("MAG"),
        resonance: get_opt("RES"),
    }
}

fn parse_metatype(s: &str) -> Metatype {
    match s.trim() {
        "Elf" => Metatype::Elf,
        "Dwarf" => Metatype::Dwarf,
        "Ork" | "Orc" => Metatype::Ork,
        "Troll" => Metatype::Troll,
        _ => Metatype::Human,
    }
}

fn detect_tradition(chum: &Chum5Character) -> Option<MagicTradition> {
    let is_adept = parse_bool(&chum.adept);
    let is_magician = parse_bool(&chum.magician);
    let is_tech = parse_bool(&chum.technomancer);
    match (is_adept, is_magician, is_tech) {
        (true, true, _) => Some(MagicTradition::MysticAdept),
        (true, false, _) => Some(MagicTradition::Adept),
        (false, true, _) => Some(MagicTradition::Magician),
        (_, _, true) => Some(MagicTradition::Technomancer),
        _ => None,
    }
}

// -- Priority mapping --

fn map_priority(chum: &Chum5Character) -> Option<PrioritySelection> {
    let parse_level = |s: &str| match s.trim().to_uppercase().as_str() {
        "A" => Some(PriorityLevel::A),
        "B" => Some(PriorityLevel::B),
        "C" => Some(PriorityLevel::C),
        "D" => Some(PriorityLevel::D),
        "E" => Some(PriorityLevel::E),
        _ => None,
    };
    let m = parse_level(&chum.priority_metatype)?;
    let a = parse_level(&chum.priority_attributes)?;
    let s = parse_level(&chum.priority_special)?;
    let sk = parse_level(&chum.priority_skills)?;
    let r = parse_level(&chum.priority_resources)?;
    Some(PrioritySelection {
        metatype: m,
        attributes: a,
        magic_or_resonance: s,
        skills: sk,
        resources: r,
    })
}

// -- Skills --

fn map_skills(
    items: &[super::parse::Chum5Skill],
    game_skills: &HashMap<String, GameSkillMeta>,
) -> Vec<Skill> {
    items
        .iter()
        .filter(|s| s.total_rating() > 0 || !s.name.is_empty())
        .map(|s| {
            let meta = game_skills.get(s.name.as_str())
                .or_else(|| game_skills.get(s.suid.as_str()));
            let linked_attribute = meta.map(|m| m.linked_attribute.clone()).unwrap_or_else(|| "AGI".to_string());
            let group = meta.and_then(|m| m.group.clone());
            let id = if !s.suid.is_empty() { s.suid.clone() } else { slugify(&s.name) };
            Skill {
                id,
                name: s.name.clone(),
                linked_attribute,
                group,
                rating: s.total_rating(),
                specializations: s.spec_names().into_iter().map(|n| Specialization { name: n, bonus: 2 }).collect(),
            }
        })
        .collect()
}

fn map_kno_skills(items: &[super::parse::Chum5KnoSkill]) -> Vec<KnowledgeSkill> {
    items.iter()
        .filter(|k| !k.name.is_empty())
        .map(|k| KnowledgeSkill {
            name: k.name.clone(),
            category: if k.category.is_empty() { "Knowledge".to_string() } else { k.category.clone() },
            rating: k.total_rating(),
        })
        .collect()
}

// -- Qualities --

fn map_qualities(items: &[super::parse::Chum5Quality], game_qualities: &HashMap<String, i32>) -> Vec<Quality> {
    items.iter()
        .filter(|q| !q.name.is_empty())
        .map(|q| {
            let quality_type = if q.quality_type == "Positive" { QualityType::Positive } else { QualityType::Negative };
            let cost = game_qualities.get(q.name.as_str())
                .copied()
                .unwrap_or_else(|| parse_int(&q.bp));
            Quality {
                id: slugify(&q.name),
                name: q.name.clone(),
                quality_type,
                cost,
                source: q.source.clone(),
                page: q.page.clone(),
                improvements: vec![],
                incompatible_with: vec![],
            }
        })
        .collect()
}

// -- Augmentations --

fn map_augmentations(items: &[super::parse::Chum5Cyberware]) -> Vec<Augmentation> {
    items.iter()
        .filter(|a| !a.name.is_empty())
        .map(|a| {
            let aug_type = if a.category.to_lowercase().contains("bio") {
                AugmentationType::Bioware
            } else {
                AugmentationType::Cyberware
            };
            let grade = match a.grade.as_str() {
                "Alpha" | "Alphaware" => AugmentationGrade::Alpha,
                "Beta" | "Betaware" => AugmentationGrade::Beta,
                "Delta" | "Deltaware" => AugmentationGrade::Delta,
                "Used" | "Usedware" => AugmentationGrade::Used,
                _ => AugmentationGrade::Standard,
            };
            let essence_cost = (parse_float(&a.ess) * 100.0).round() as i32;
            let cost = parse_int(&a.cost) as i64;
            Augmentation {
                id: slugify(&a.name),
                name: a.name.clone(),
                augmentation_type: aug_type,
                grade,
                essence_cost,
                availability: a.availability.clone(),
                cost,
                source: a.source.clone(),
                page: a.page.clone(),
                improvements: vec![],
            }
        })
        .collect()
}

// -- Spells --

fn map_spells(items: &[super::parse::Chum5Spell]) -> Vec<Spell> {
    items.iter()
        .filter(|s| !s.name.is_empty())
        .map(|s| {
            let category = match s.category.as_str() {
                "Detection" => SpellCategory::Detection,
                "Health" => SpellCategory::Health,
                "Illusion" => SpellCategory::Illusion,
                "Manipulation" => SpellCategory::Manipulation,
                _ => SpellCategory::Combat,
            };
            let spell_type = if s.spell_type == "M" { SpellType::Mana } else { SpellType::Physical };
            Spell {
                id: slugify(&s.name),
                name: s.name.clone(),
                category,
                spell_type,
                range: s.range.clone(),
                damage: s.damage.clone(),
                duration: s.duration.clone(),
                drain: s.drain.clone(),
                source: s.source.clone(),
                page: s.page.clone(),
            }
        })
        .collect()
}

// -- Adept Powers --

fn map_powers(items: &[super::parse::Chum5Power]) -> Vec<AdeptPower> {
    items.iter()
        .filter(|p| !p.name.is_empty())
        .map(|p| {
            let cost = (parse_float(&p.pointsperlevel) * 100.0).round() as i32;
            AdeptPower {
                id: slugify(&p.name),
                name: p.name.clone(),
                cost,
                levels: false,
                source: p.source.clone(),
                page: p.page.clone(),
            }
        })
        .collect()
}

// -- Complex Forms --

fn map_complex_forms(items: &[super::parse::Chum5ComplexForm]) -> Vec<ComplexForm> {
    items.iter()
        .filter(|f| !f.name.is_empty())
        .map(|f| ComplexForm {
            id: slugify(&f.name),
            name: f.name.clone(),
            target: f.target.clone(),
            duration: f.duration.clone(),
            fading: f.fv.clone(),
            source: f.source.clone(),
            page: f.page.clone(),
        })
        .collect()
}

// -- Contacts --

fn map_contacts(items: &[super::parse::Chum5Contact]) -> Vec<Contact> {
    items.iter()
        .filter(|c| !c.name.is_empty())
        .map(|c| Contact {
            id: uuid::Uuid::new_v4().to_string(),
            name: c.name.clone(),
            connection: parse_int(&c.connection).clamp(1, 6) as u8,
            loyalty: parse_int(&c.loyalty).clamp(1, 6) as u8,
            archetype: c.role.clone(),
            notes: String::new(),
        })
        .collect()
}

// -- Equipment --

fn map_weapons(items: &[super::parse::Chum5Weapon]) -> Vec<Weapon> {
    items.iter()
        .filter(|w| !w.name.is_empty())
        .map(|w| Weapon {
            id: slugify(&w.name),
            name: w.name.clone(),
            category: w.category.clone(),
            damage: w.damage.clone(),
            ap: w.ap.clone(),
            mode: w.mode.clone(),
            recoil_comp: 0,
            ammo: w.ammo.clone(),
            availability: w.availability.clone(),
            cost: parse_int(&w.cost) as i64,
            source: w.source.clone(),
            page: w.page.clone(),
        })
        .collect()
}

fn map_armor(items: &[super::parse::Chum5Armor]) -> Vec<Armor> {
    items.iter()
        .filter(|a| !a.name.is_empty())
        .map(|a| Armor {
            id: slugify(&a.name),
            name: a.name.clone(),
            armor_value: parse_int(&a.armor_value),
            availability: a.availability.clone(),
            cost: parse_int(&a.cost) as i64,
            source: a.source.clone(),
            page: a.page.clone(),
        })
        .collect()
}

fn map_gear(items: &[super::parse::Chum5Gear]) -> Vec<GearItem> {
    items.iter()
        .filter(|g| !g.name.is_empty())
        .map(|g| {
            let rating = if g.rating.is_empty() { None } else { g.rating.parse().ok() };
            GearItem {
                id: uuid::Uuid::new_v4().to_string(),
                name: g.name.clone(),
                category: g.category.clone(),
                rating,
                availability: String::new(),
                cost: 0,
                source: g.source.clone(),
                page: g.page.clone(),
            }
        })
        .collect()
}

fn map_vehicles(items: &[super::parse::Chum5Vehicle]) -> Vec<Vehicle> {
    items.iter()
        .filter(|v| !v.name.is_empty())
        .map(|v| Vehicle {
            id: slugify(&v.name),
            name: v.name.clone(),
            // Handling may be "4/2" (road/offroad) — take the first number
            handling: v.handling.split('/').next().unwrap_or("0").to_string(),
            speed: parse_int(&v.speed),
            acceleration: parse_int(&v.acceleration),
            body: parse_int(&v.body),
            armor: 0,
            pilot: parse_int(&v.pilot),
            sensor: parse_int(&v.sensor),
            availability: String::new(),
            cost: 0,
            source: v.source.clone(),
            page: v.page.clone(),
        })
        .collect()
}

// -- Game data helpers --

struct GameSkillMeta {
    linked_attribute: String,
    group: Option<String>,
}

async fn load_skills(pool: &SqlitePool, edition: &str) -> ImportResult<HashMap<String, GameSkillMeta>> {
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, name, linked_attribute, skill_group FROM skills_data WHERE edition = ?",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;

    let mut by_name = HashMap::new();
    let mut by_id = HashMap::new();
    for (id, name, linked_attribute, group) in rows {
        let meta = GameSkillMeta { linked_attribute: linked_attribute.clone(), group: group.clone() };
        by_name.insert(name.clone(), GameSkillMeta { linked_attribute: linked_attribute.clone(), group: group.clone() });
        by_id.insert(id.clone(), meta);
        // Also insert by suid without hyphens (some Chummer files strip hyphens)
        let clean_id = id.replace('-', "");
        by_id.entry(clean_id).or_insert(GameSkillMeta { linked_attribute, group });
    }
    // Merge by_id into by_name (both keyed differently but same data)
    let mut merged: HashMap<String, GameSkillMeta> = by_name;
    for (k, v) in by_id {
        merged.entry(k).or_insert(v);
    }
    Ok(merged)
}

async fn load_qualities(pool: &SqlitePool, edition: &str) -> ImportResult<HashMap<String, i32>> {
    let rows: Vec<(String, i32)> = sqlx::query_as(
        "SELECT name, cost FROM qualities WHERE edition = ?",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

// -- Utilities --

fn slugify(s: &str) -> String {
    s.to_lowercase().replace(' ', "-").replace(['\'', '.', ','], "")
}
