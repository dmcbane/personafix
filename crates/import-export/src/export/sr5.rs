use std::collections::HashMap;

use sqlx::SqlitePool;

use personafix_core::model::{
    character::CharacterBase,
    magic::MagicTradition,
    priority::PriorityLevel,
};

use crate::error::ImportResult;

pub async fn export_sr5(
    base: &CharacterBase,
    game_data_pool: &SqlitePool,
) -> ImportResult<String> {
    let suid_by_name = load_skill_suids(game_data_pool, "SR5").await?;
    Ok(build_xml(base, &suid_by_name))
}

async fn load_skill_suids(pool: &SqlitePool, edition: &str) -> ImportResult<HashMap<String, String>> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT name, id FROM skills_data WHERE edition = ?")
        .bind(edition)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().collect())
}

fn priority_str(level: PriorityLevel) -> &'static str {
    match level {
        PriorityLevel::A => "A",
        PriorityLevel::B => "B",
        PriorityLevel::C => "C",
        PriorityLevel::D => "D",
        PriorityLevel::E => "E",
    }
}

fn magic_flags(tradition: Option<MagicTradition>) -> (&'static str, &'static str, &'static str, &'static str) {
    // (magenabled, adept, magician, technomancer)
    match tradition {
        Some(MagicTradition::Adept) => ("true", "true", "false", "false"),
        Some(MagicTradition::Magician) => ("true", "false", "true", "false"),
        Some(MagicTradition::MysticAdept) => ("true", "true", "true", "false"),
        Some(MagicTradition::Technomancer) => ("false", "false", "false", "true"),
        None => ("false", "false", "false", "false"),
    }
}

fn build_xml(base: &CharacterBase, suid_by_name: &HashMap<String, String>) -> String {
    let attrs = &base.attributes;
    let (magenabled, adept, magician, technomancer) = magic_flags(base.magic_tradition);

    let (pm, pa, ps, psk, pr) = base.priority_selection.as_ref().map(|p| {
        (
            priority_str(p.metatype),
            priority_str(p.attributes),
            priority_str(p.magic_or_resonance),
            priority_str(p.skills),
            priority_str(p.resources),
        )
    }).unwrap_or(("E", "B", "C", "A", "D"));

    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<character>
  <name>{}</name>
  <metatype>{:?}</metatype>
  <metatypecategory>Metahuman</metatypecategory>
  <prioritymetatype>{}</prioritymetatype>
  <priorityattributes>{}</priorityattributes>
  <priorityspecial>{}</priorityspecial>
  <priorityskills>{}</priorityskills>
  <priorityresources>{}</priorityresources>
  <magenabled>{}</magenabled>
  <adept>{}</adept>
  <magician>{}</magician>
  <technomancer>{}</technomancer>
  <nuyen>0</nuyen>
  <karma>0</karma>
  <totalkarma>0</totalkarma>
  <attributes>
    <attribute><name>BOD</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>AGI</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>REA</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>STR</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>WIL</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>LOG</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>INT</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>CHA</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>EDG</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>ESS</name><base>6</base><karma>0</karma></attribute>
    <attribute><name>MAG</name><base>{}</base><karma>0</karma></attribute>
    <attribute><name>RES</name><base>{}</base><karma>0</karma></attribute>
  </attributes>
"#,
        escape_xml(&base.name),
        base.metatype,
        pm, pa, ps, psk, pr,
        magenabled, adept, magician, technomancer,
        attrs.body, attrs.agility, attrs.reaction, attrs.strength,
        attrs.willpower, attrs.logic, attrs.intuition, attrs.charisma,
        attrs.edge,
        attrs.magic.unwrap_or(0),
        attrs.resonance.unwrap_or(0),
    );

    // Skills
    xml.push_str("  <newskills>\n    <skills>\n");
    for skill in &base.skills {
        let suid = suid_by_name.get(&skill.name).cloned().unwrap_or_else(|| skill.id.clone());
        let spec = skill.specializations.first().map(|s| s.name.as_str()).unwrap_or("");
        xml.push_str(&format!(
            "      <skill><suid>{}</suid><name>{}</name><base>{}</base><karma>0</karma><spec>{}</spec><specs></specs></skill>\n",
            escape_xml(&suid), escape_xml(&skill.name), skill.rating, escape_xml(spec)
        ));
    }
    xml.push_str("    </skills>\n    <knoskills>\n");
    for k in &base.knowledge_skills {
        xml.push_str(&format!(
            "      <skill><name>{}</name><base>{}</base><karma>0</karma><category>{}</category></skill>\n",
            escape_xml(&k.name), k.rating, escape_xml(&k.category)
        ));
    }
    xml.push_str("    </knoskills>\n  </newskills>\n");

    // Qualities
    xml.push_str("  <qualities>\n");
    for q in &base.qualities {
        let qtype = format!("{:?}", q.quality_type);
        xml.push_str(&format!(
            "    <quality><name>{}</name><type>{}</type><bp>{}</bp></quality>\n",
            escape_xml(&q.name), qtype, q.cost
        ));
    }
    xml.push_str("  </qualities>\n");

    // Cyberwares
    xml.push_str("  <cyberwares>\n");
    for a in &base.augmentations {
        let ess = format!("{:.2}", a.essence_cost as f64 / 100.0);
        xml.push_str(&format!(
            "    <cyberware><name>{}</name><category>{:?}</category><grade>{:?}</grade><ess>{}</ess></cyberware>\n",
            escape_xml(&a.name), a.augmentation_type, a.grade, ess
        ));
    }
    xml.push_str("  </cyberwares>\n");

    // Spells
    xml.push_str("  <spells>\n");
    for s in &base.spells {
        let t = if format!("{:?}", s.spell_type) == "Mana" { "M" } else { "P" };
        xml.push_str(&format!(
            "    <spell><name>{}</name><category>{:?}</category><type>{}</type><drain>{}</drain></spell>\n",
            escape_xml(&s.name), s.category, t, escape_xml(&s.drain)
        ));
    }
    xml.push_str("  </spells>\n");

    // Adept powers
    xml.push_str("  <powers>\n");
    for p in &base.adept_powers {
        let cost = format!("{:.2}", p.cost as f64 / 100.0);
        xml.push_str(&format!(
            "    <power><name>{}</name><pointsperlevel>{}</pointsperlevel></power>\n",
            escape_xml(&p.name), cost
        ));
    }
    xml.push_str("  </powers>\n");

    // Complex forms
    xml.push_str("  <complexforms>\n");
    for f in &base.complex_forms {
        xml.push_str(&format!(
            "    <complexform><name>{}</name><target>{}</target><duration>{}</duration><fv>{}</fv></complexform>\n",
            escape_xml(&f.name), escape_xml(&f.target), escape_xml(&f.duration), escape_xml(&f.fading)
        ));
    }
    xml.push_str("  </complexforms>\n");

    // Contacts
    xml.push_str("  <contacts>\n");
    for c in &base.contacts {
        xml.push_str(&format!(
            "    <contact><name>{}</name><role>{}</role><connection>{}</connection><loyalty>{}</loyalty></contact>\n",
            escape_xml(&c.name), escape_xml(&c.archetype), c.connection, c.loyalty
        ));
    }
    xml.push_str("  </contacts>\n");

    // Weapons
    xml.push_str("  <weapons>\n");
    for w in &base.weapons {
        xml.push_str(&format!(
            "    <weapon><name>{}</name><category>{}</category><damage>{}</damage><ap>{}</ap><mode>{}</mode><ammo>{}</ammo></weapon>\n",
            escape_xml(&w.name), escape_xml(&w.category), escape_xml(&w.damage), escape_xml(&w.ap), escape_xml(&w.mode), escape_xml(&w.ammo)
        ));
    }
    xml.push_str("  </weapons>\n");

    // Armors
    xml.push_str("  <armors>\n");
    for a in &base.armor {
        xml.push_str(&format!(
            "    <armor><name>{}</name><armor>{}</armor></armor>\n",
            escape_xml(&a.name), a.armor_value
        ));
    }
    xml.push_str("  </armors>\n");

    // Gears
    xml.push_str("  <gears>\n");
    for g in &base.gear {
        let rating = g.rating.map(|r| r.to_string()).unwrap_or_default();
        xml.push_str(&format!(
            "    <gear><name>{}</name><category>{}</category><rating>{}</rating></gear>\n",
            escape_xml(&g.name), escape_xml(&g.category), rating
        ));
    }
    xml.push_str("  </gears>\n");

    // Vehicles
    xml.push_str("  <vehicles>\n");
    for v in &base.vehicles {
        xml.push_str(&format!(
            "    <vehicle><name>{}</name><handling>{}</handling><speed>{}</speed><acceleration>{}</acceleration><body>{}</body><pilot>{}</pilot><sensor>{}</sensor></vehicle>\n",
            escape_xml(&v.name), escape_xml(&v.handling), v.speed, v.acceleration, v.body, v.pilot, v.sensor
        ));
    }
    xml.push_str("  </vehicles>\n");

    xml.push_str("</character>\n");
    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
