mod db;
mod error;
mod xml;

use std::path::PathBuf;

use clap::Parser;

use error::MigrateResult;

#[derive(Parser)]
#[command(name = "personafix-migrate")]
#[command(about = "Migrate Chummer XML game data to personafix SQLite format")]
struct Cli {
    /// Path to ChummerGenSR4 data directory (e.g. vendor/chummer-sr4/bin/data)
    #[arg(long)]
    sr4_path: Option<PathBuf>,

    /// Path to Chummer5a data directory (e.g. vendor/chummer5a/Chummer/data)
    #[arg(long)]
    sr5_path: Option<PathBuf>,

    /// Output SQLite database path
    #[arg(long, default_value = "game_data.db")]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> MigrateResult<()> {
    let cli = Cli::parse();

    if cli.sr4_path.is_none() && cli.sr5_path.is_none() {
        return Err(error::MigrateError::Other(
            "At least one of --sr4-path or --sr5-path must be provided".to_string(),
        ));
    }

    let mut datasets = Vec::new();

    if let Some(ref sr5_path) = cli.sr5_path {
        println!("Parsing SR5 data from: {}", sr5_path.display());
        let data = xml::sr5::parse_sr5(sr5_path)?;
        println!(
            "  SR5: {} books, {} metatypes, {} skills, {} qualities, {} weapons, {} armor, {} augmentations, {} spells",
            data.books.len(),
            data.metatypes.len(),
            data.skills.len(),
            data.qualities.len(),
            data.weapons.len(),
            data.armor.len(),
            data.augmentations.len(),
            data.spells.len(),
        );
        datasets.push(data);
    }

    if let Some(ref sr4_path) = cli.sr4_path {
        println!("Parsing SR4 data from: {}", sr4_path.display());
        let data = xml::sr4::parse_sr4(sr4_path)?;
        println!(
            "  SR4: {} books, {} metatypes, {} skills, {} qualities, {} weapons, {} armor, {} augmentations, {} spells",
            data.books.len(),
            data.metatypes.len(),
            data.skills.len(),
            data.qualities.len(),
            data.weapons.len(),
            data.armor.len(),
            data.augmentations.len(),
            data.spells.len(),
        );
        datasets.push(data);
    }

    println!("Seeding database: {}", cli.output.display());
    db::seed(&cli.output, &datasets).await?;
    println!("Done.");

    Ok(())
}
