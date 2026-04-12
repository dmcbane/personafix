//! personafix-migrate: One-time migration tool that converts ChummerGenSR4 and
//! Chummer5a XML data files into the personafix SQLite game data schema.
//!
//! Usage:
//!   cargo run --bin personafix-migrate -- --sr4-path <path> --sr5-path <path> --output <db-path>
//!
//! This binary is a developer/install-time tool, not an end-user tool.

fn main() {
    println!("personafix-migrate: XML → SQLite game data migration");
    println!("Not yet implemented. Supply --sr4-path and --sr5-path to Chummer XML directories.");

    // TODO: Parse CLI args
    // TODO: xml::parse_sr4() — deserialize ChummerGenSR4 XML files
    // TODO: xml::parse_sr5() — deserialize Chummer5a XML files
    // TODO: db::seed() — INSERT parsed records into SQLite with edition tags
}
