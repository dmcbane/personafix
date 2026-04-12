use std::path::Path;

use crate::error::MigrateResult;

use super::*;

/// Parse all SR5 game data from a Chummer5a data directory.
pub fn parse_sr5(data_dir: &Path) -> MigrateResult<ParsedGameData> {
    let mut data = ParsedGameData {
        edition: "SR5".to_string(),
        ..Default::default()
    };

    data.books = parse_books(&data_dir.join("books.xml"))?;
    data.metatypes = parse_metatypes(&data_dir.join("metatypes.xml"))?;
    data.skills = parse_skills(&data_dir.join("skills.xml"))?;
    data.qualities = parse_qualities(&data_dir.join("qualities.xml"))?;
    data.weapons = parse_weapons(&data_dir.join("weapons.xml"))?;
    data.armor = parse_armor(&data_dir.join("armor.xml"))?;

    let mut augmentations = parse_cyberware(&data_dir.join("cyberware.xml"), "Cyberware")?;
    augmentations.extend(parse_cyberware(&data_dir.join("bioware.xml"), "Bioware")?);
    data.augmentations = augmentations;

    data.spells = parse_spells(&data_dir.join("spells.xml"))?;

    Ok(data)
}

fn parse_books(path: &Path) -> MigrateResult<Vec<ParsedBook>> {
    let xml = std::fs::read_to_string(path)?;
    let chummer: ChummerBooks = quick_xml::de::from_str(&xml)?;
    Ok(chummer
        .books
        .items
        .into_iter()
        .map(|b| ParsedBook {
            id: b.id,
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
            id: m.id,
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
            id: s.id,
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
        .map(|q| ParsedQuality {
            id: q.id,
            name: q.name,
            quality_type: q.category.clone(),
            cost: parse_int(&q.karma),
            source: q.source,
            page: q.page,
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
            id: w.id,
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
            id: a.id,
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
    // Both cyberware.xml and bioware.xml use the same structure,
    // but the collection element differs.
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
            id: c.id,
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
                id: s.id,
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

    fn sr5_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/chummer5a/Chummer/data")
    }

    #[test]
    fn parse_sr5_books_returns_entries() {
        let books = parse_books(&sr5_data_dir().join("books.xml")).unwrap();
        assert!(!books.is_empty(), "should parse at least one book");
        let sr5_book = books.iter().find(|b| b.abbreviation == "SR5");
        assert!(sr5_book.is_some(), "should find SR5 core rulebook");
    }

    #[test]
    fn parse_sr5_metatypes_has_human_with_correct_ranges() {
        let metatypes = parse_metatypes(&sr5_data_dir().join("metatypes.xml")).unwrap();
        assert!(!metatypes.is_empty());
        let human = metatypes.iter().find(|m| m.name == "Human").unwrap();
        assert_eq!(human.body_min, 1);
        assert_eq!(human.body_max, 6);
        assert_eq!(human.edge_min, 2);
        assert_eq!(human.edge_max, 7);
    }

    #[test]
    fn parse_sr5_metatypes_has_all_core_five() {
        let metatypes = parse_metatypes(&sr5_data_dir().join("metatypes.xml")).unwrap();
        let names: Vec<&str> = metatypes.iter().map(|m| m.name.as_str()).collect();
        for expected in &["Human", "Elf", "Dwarf", "Ork", "Troll"] {
            assert!(names.contains(expected), "missing metatype: {expected}");
        }
    }

    #[test]
    fn parse_sr5_skills_returns_entries() {
        let skills = parse_skills(&sr5_data_dir().join("skills.xml")).unwrap();
        assert!(!skills.is_empty());
        let pistols = skills.iter().find(|s| s.name == "Pistols");
        assert!(pistols.is_some(), "should find Pistols skill");
        let pistols = pistols.unwrap();
        assert_eq!(pistols.linked_attribute, "AGI");
    }

    #[test]
    fn parse_sr5_qualities_returns_positive_and_negative() {
        let qualities = parse_qualities(&sr5_data_dir().join("qualities.xml")).unwrap();
        assert!(!qualities.is_empty());
        let has_positive = qualities.iter().any(|q| q.quality_type == "Positive");
        let has_negative = qualities.iter().any(|q| q.quality_type == "Negative");
        assert!(has_positive, "should have positive qualities");
        assert!(has_negative, "should have negative qualities");
    }

    #[test]
    fn parse_sr5_weapons_returns_entries() {
        let weapons = parse_weapons(&sr5_data_dir().join("weapons.xml")).unwrap();
        assert!(!weapons.is_empty());
    }

    #[test]
    fn parse_sr5_armor_returns_entries() {
        let armor = parse_armor(&sr5_data_dir().join("armor.xml")).unwrap();
        assert!(!armor.is_empty());
    }

    #[test]
    fn parse_sr5_cyberware_returns_entries() {
        let cyberware =
            parse_cyberware(&sr5_data_dir().join("cyberware.xml"), "Cyberware").unwrap();
        assert!(!cyberware.is_empty());
        assert!(cyberware.iter().all(|c| c.augmentation_type == "Cyberware"));
    }

    #[test]
    fn parse_sr5_bioware_returns_entries() {
        let bioware = parse_cyberware(&sr5_data_dir().join("bioware.xml"), "Bioware").unwrap();
        assert!(!bioware.is_empty());
        assert!(bioware.iter().all(|b| b.augmentation_type == "Bioware"));
    }

    #[test]
    fn parse_sr5_spells_returns_entries_with_types() {
        let spells = parse_spells(&sr5_data_dir().join("spells.xml")).unwrap();
        assert!(!spells.is_empty());
        let has_physical = spells.iter().any(|s| s.spell_type == "Physical");
        let has_mana = spells.iter().any(|s| s.spell_type == "Mana");
        assert!(has_physical, "should have Physical spells");
        assert!(has_mana, "should have Mana spells");
    }

    #[test]
    fn parse_sr5_full_succeeds() {
        let data = parse_sr5(&sr5_data_dir()).unwrap();
        assert_eq!(data.edition, "SR5");
        assert!(!data.books.is_empty());
        assert!(!data.metatypes.is_empty());
        assert!(!data.skills.is_empty());
        assert!(!data.qualities.is_empty());
        assert!(!data.weapons.is_empty());
        assert!(!data.armor.is_empty());
        assert!(!data.augmentations.is_empty());
        assert!(!data.spells.is_empty());
    }
}
