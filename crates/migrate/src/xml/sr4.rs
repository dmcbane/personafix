use std::path::Path;

use crate::error::MigrateResult;

use super::*;

/// Parse all SR4 game data from a ChummerGenSR4 data directory.
/// SR4 XML files differ from SR5: many entries lack <id>, and use <bp> instead of <karma>.
pub fn parse_sr4(data_dir: &Path) -> MigrateResult<ParsedGameData> {
    let mut data = ParsedGameData {
        edition: "SR4".to_string(),
        ..Default::default()
    };

    data.books = parse_books(&data_dir.join("books.xml"))?;
    data.metatypes = parse_metatypes(&data_dir.join("metatypes.xml"))?;
    data.skills = parse_skills(&data_dir.join("skills.xml"))?;

    // SR4 has qualities.xml in the same format
    if data_dir.join("qualities.xml").exists() {
        data.qualities = parse_qualities(&data_dir.join("qualities.xml"))?;
    }

    data.weapons = parse_weapons(&data_dir.join("weapons.xml"))?;
    data.armor = parse_armor(&data_dir.join("armor.xml"))?;

    // SR4 has cyberware.xml but may not have separate bioware.xml
    if data_dir.join("cyberware.xml").exists() {
        data.augmentations = parse_cyberware(&data_dir.join("cyberware.xml"), "Cyberware")?;
    }
    if data_dir.join("bioware.xml").exists() {
        data.augmentations
            .extend(parse_cyberware(&data_dir.join("bioware.xml"), "Bioware")?);
    }

    data.spells = parse_spells(&data_dir.join("spells.xml"))?;

    Ok(data)
}

/// Generate a stable ID from edition + name for entries that lack UUIDs.
fn make_id(name: &str) -> String {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, format!("sr4:{name}").as_bytes()).to_string()
}

fn ensure_id(id: &str, name: &str) -> String {
    if id.is_empty() {
        make_id(name)
    } else {
        id.to_string()
    }
}

fn parse_books(path: &Path) -> MigrateResult<Vec<ParsedBook>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerBooks = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .books
        .items
        .into_iter()
        .map(|b| ParsedBook {
            id: ensure_id(&b.id, &b.name),
            name: b.name,
            abbreviation: b.code,
        })
        .collect())
}

fn parse_metatypes(path: &Path) -> MigrateResult<Vec<ParsedMetatype>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerMetatypes = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .metatypes
        .items
        .into_iter()
        .filter(|m| m.category == "Metahuman")
        .map(|m| ParsedMetatype {
            id: ensure_id(&m.id, &m.name),
            name: m.name,
            body_min: parse_int(&m.bodmin),
            body_max: parse_int(&m.bodmax),
            agility_min: parse_int(&m.agimin),
            agility_max: parse_int(&m.agimax),
            reaction_min: parse_int(&m.reamin),
            reaction_max: parse_int(&m.reamax),
            strength_min: parse_int(&m.strmin),
            strength_max: parse_int(&m.strmax),
            willpower_min: parse_int(&m.wilmin),
            willpower_max: parse_int(&m.wilmax),
            logic_min: parse_int(&m.logmin),
            logic_max: parse_int(&m.logmax),
            intuition_min: parse_int(&m.intmin),
            intuition_max: parse_int(&m.intmax),
            charisma_min: parse_int(&m.chamin),
            charisma_max: parse_int(&m.chamax),
            edge_min: parse_int(&m.edgmin),
            edge_max: parse_int(&m.edgmax),
            source: m.source,
            page: m.page,
        })
        .collect())
}

fn parse_skills(path: &Path) -> MigrateResult<Vec<ParsedSkill>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerSkills = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .skills
        .items
        .into_iter()
        .map(|s| ParsedSkill {
            id: ensure_id(&s.id, &s.name),
            name: s.name,
            linked_attribute: s.attribute,
            skill_group: if s.skillgroup.is_empty() {
                None
            } else {
                Some(s.skillgroup)
            },
            source: s.source,
            page: s.page,
        })
        .collect())
}

fn parse_qualities(path: &Path) -> MigrateResult<Vec<ParsedQuality>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerQualities = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .qualities
        .items
        .into_iter()
        .map(|q| {
            // SR4 uses <bp>, SR5 uses <karma>
            let cost = if !q.bp.is_empty() {
                parse_int(&q.bp)
            } else {
                parse_int(&q.karma)
            };
            ParsedQuality {
                id: ensure_id(&q.id, &q.name),
                name: q.name,
                quality_type: q.category.clone(),
                cost,
                source: q.source,
                page: q.page,
            }
        })
        .collect())
}

fn parse_weapons(path: &Path) -> MigrateResult<Vec<ParsedWeapon>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerWeapons = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .weapons
        .items
        .into_iter()
        .map(|w| ParsedWeapon {
            id: ensure_id(&w.id, &w.name),
            name: w.name,
            category: w.category,
            damage: w.damage,
            ap: w.ap,
            mode: w.mode,
            recoil_comp: w.rc,
            ammo: w.ammo,
            availability: w.avail,
            cost: w.cost,
            source: w.source,
            page: w.page,
        })
        .collect())
}

fn parse_armor(path: &Path) -> MigrateResult<Vec<ParsedArmor>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerArmor = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .armors
        .items
        .into_iter()
        .map(|a| ParsedArmor {
            id: ensure_id(&a.id, &a.name),
            name: a.name,
            armor_value: a.armor_value,
            availability: a.avail,
            cost: a.cost,
            source: a.source,
            page: a.page,
        })
        .collect())
}

fn parse_cyberware(path: &Path, aug_type: &str) -> MigrateResult<Vec<ParsedAugmentation>> {
    let xml = std::fs::read_to_string(path)?;
    let items: Vec<XmlCyberware> = if aug_type == "Cyberware" {
        let chummer: ChummerCyberware = quick_xml::de::from_str(&xml)?;
        chummer.cyberwares.items
    } else {
        let chummer: ChummerBioware = quick_xml::de::from_str(&xml)?;
        chummer.biowares.items
    };

    Ok(items
        .into_iter()
        .map(|c| ParsedAugmentation {
            id: ensure_id(&c.id, &c.name),
            name: c.name,
            augmentation_type: aug_type.to_string(),
            essence_cost: c.ess,
            capacity: c.capacity,
            availability: c.avail,
            cost: c.cost,
            source: c.source,
            page: c.page,
        })
        .collect())
}

fn parse_spells(path: &Path) -> MigrateResult<Vec<ParsedSpell>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerSpells = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .spells
        .items
        .into_iter()
        .map(|s| {
            let spell_type = match s.spell_type.as_str() {
                "P" => "Physical".to_string(),
                "M" => "Mana".to_string(),
                other => other.to_string(),
            };
            ParsedSpell {
                id: ensure_id(&s.id, &s.name),
                name: s.name,
                category: s.category,
                spell_type,
                range: s.range,
                damage: s.damage,
                duration: s.duration,
                drain: s.dv,
                source: s.source,
                page: s.page,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sr4_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/chummer-sr4/bin/data")
    }

    #[test]
    #[ignore = "requires vendor/ Chummer data — run with: cargo test -- --ignored"]
    fn parse_sr4_books_returns_entries() {
        let books = parse_books(&sr4_data_dir().join("books.xml")).unwrap();
        assert!(!books.is_empty());
        // SR4 books should all have generated IDs (non-empty)
        assert!(books.iter().all(|b| !b.id.is_empty()));
    }

    #[test]
    #[ignore = "requires vendor/ Chummer data — run with: cargo test -- --ignored"]
    fn parse_sr4_metatypes_has_human() {
        let metatypes = parse_metatypes(&sr4_data_dir().join("metatypes.xml")).unwrap();
        let human = metatypes.iter().find(|m| m.name == "Human").unwrap();
        assert_eq!(human.body_min, 1);
        assert_eq!(human.body_max, 6);
        assert_eq!(human.edge_min, 2);
        assert_eq!(human.edge_max, 7);
    }

    #[test]
    #[ignore = "requires vendor/ Chummer data — run with: cargo test -- --ignored"]
    fn parse_sr4_skills_returns_entries() {
        let skills = parse_skills(&sr4_data_dir().join("skills.xml")).unwrap();
        assert!(!skills.is_empty());
    }

    #[test]
    #[ignore = "requires vendor/ Chummer data — run with: cargo test -- --ignored"]
    fn parse_sr4_weapons_returns_entries() {
        let weapons = parse_weapons(&sr4_data_dir().join("weapons.xml")).unwrap();
        assert!(!weapons.is_empty());
    }

    #[test]
    #[ignore = "requires vendor/ Chummer data — run with: cargo test -- --ignored"]
    fn parse_sr4_full_succeeds() {
        let data = parse_sr4(&sr4_data_dir()).unwrap();
        assert_eq!(data.edition, "SR4");
        assert!(!data.books.is_empty());
        assert!(!data.metatypes.is_empty());
        assert!(!data.skills.is_empty());
        assert!(!data.weapons.is_empty());
    }
}
