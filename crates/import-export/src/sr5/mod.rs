pub mod map;
pub mod parse;

use std::path::Path;

use crate::error::{ImportError, ImportResult};
use parse::Chum5Character;

pub fn parse_file(path: &Path) -> ImportResult<Chum5Character> {
    let xml = std::fs::read_to_string(path)?;
    parse_str(&xml)
}

pub fn parse_str(xml: &str) -> ImportResult<Chum5Character> {
    quick_xml::de::from_str(xml).map_err(ImportError::Xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_CHUM5: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<character>
  <name>Test Runner</name>
  <metatype>Human</metatype>
  <prioritymetatype>E</prioritymetatype>
  <priorityattributes>B</priorityattributes>
  <priorityspecial>E</priorityspecial>
  <priorityskills>A</priorityskills>
  <priorityresources>C</priorityresources>
  <magenabled>false</magenabled>
  <adept>false</adept>
  <magician>false</magician>
  <technomancer>false</technomancer>
  <nuyen>5000</nuyen>
  <karma>5</karma>
  <totalkarma>50</totalkarma>
  <attributes>
    <attribute><name>BOD</name><base>4</base><karma>0</karma></attribute>
    <attribute><name>AGI</name><base>6</base><karma>0</karma></attribute>
    <attribute><name>REA</name><base>4</base><karma>0</karma></attribute>
    <attribute><name>STR</name><base>3</base><karma>0</karma></attribute>
    <attribute><name>WIL</name><base>3</base><karma>0</karma></attribute>
    <attribute><name>LOG</name><base>3</base><karma>0</karma></attribute>
    <attribute><name>INT</name><base>4</base><karma>0</karma></attribute>
    <attribute><name>CHA</name><base>3</base><karma>0</karma></attribute>
    <attribute><name>EDG</name><base>3</base><karma>0</karma></attribute>
    <attribute><name>MAG</name><base>0</base><karma>0</karma></attribute>
    <attribute><name>RES</name><base>0</base><karma>0</karma></attribute>
  </attributes>
  <newskills>
    <skills>
      <skill>
        <suid>abc123</suid>
        <name>Pistols</name>
        <base>6</base>
        <karma>0</karma>
        <spec>Heavy Pistols</spec>
        <specs></specs>
      </skill>
    </skills>
    <knoskills>
      <skill>
        <name>English</name>
        <base>6</base>
        <karma>0</karma>
        <category>Language</category>
      </skill>
    </knoskills>
  </newskills>
  <qualities></qualities>
  <cyberwares></cyberwares>
  <spells></spells>
  <powers></powers>
  <complexforms></complexforms>
  <contacts></contacts>
  <weapons></weapons>
  <armors></armors>
  <gears></gears>
  <vehicles></vehicles>
</character>"#;

    #[test]
    fn test_parse_minimal_chum5() {
        let chum = parse_str(MINIMAL_CHUM5).expect("parse should succeed");
        assert_eq!(chum.name, "Test Runner");
        assert_eq!(chum.metatype, "Human");
        assert_eq!(chum.attributes.items.len(), 11);
        let bod = chum.attributes.items.iter().find(|a| a.name == "BOD").unwrap();
        assert_eq!(bod.total(), 4);
        assert_eq!(chum.newskills.skills.items.len(), 1);
        assert_eq!(chum.newskills.skills.items[0].name, "Pistols");
        assert_eq!(chum.newskills.skills.items[0].total_rating(), 6);
        assert_eq!(chum.newskills.skills.items[0].spec_names(), vec!["Heavy Pistols"]);
        assert_eq!(chum.newskills.knoskills.items.len(), 1);
        assert_eq!(chum.newskills.knoskills.items[0].name, "English");
    }

    #[test]
    #[ignore = "requires Chummer5a test file; set CHUMMER_TEST_FILE env var or clone vendor/chummer5a"]
    fn test_import_sr5_apex_predator() {
        let path = std::env::var("CHUMMER_TEST_FILE").unwrap_or_else(|_| {
            "vendor/chummer5a/Chummer.Tests/TestFiles/Apex Predator.chum5".to_string()
        });
        let chum = parse_file(std::path::Path::new(&path)).expect("should parse Apex Predator.chum5");
        assert!(!chum.name.is_empty(), "name should be non-empty");
        assert!(!chum.metatype.is_empty(), "metatype should be non-empty");
        let has_skill = chum.newskills.skills.items.iter().any(|s| s.total_rating() > 0);
        assert!(has_skill, "should have at least one skill with rating > 0");
    }
}
