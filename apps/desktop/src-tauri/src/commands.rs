use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

use personafix_core::ledger::events::LedgerEvent;
use personafix_core::ledger::projection;
use personafix_core::model::{
    attributes::{Attributes, Metatype},
    character::{CharacterBase, CharacterSummary, ComputedCharacter},
    edition::Edition,
};
use personafix_core::rules::{sr4::SR4Rules, sr5::SR5Rules, traits::CharacterRules};

use crate::error::AppError;
use crate::state::AppState;

// -- Types for IPC --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub name: String,
}

// -- Helper to get the pool or return an error --

async fn get_pool(state: &State<'_, AppState>) -> Result<SqlitePool, AppError> {
    let guard = state.campaign_pool.read().await;
    guard.clone().ok_or_else(|| AppError {
        kind: "no_campaign".to_string(),
        message: "No campaign is currently open".to_string(),
    })
}

fn rules_for_edition(edition: &Edition) -> Box<dyn CharacterRules> {
    match edition {
        Edition::SR4 => Box::new(SR4Rules),
        Edition::SR5 => Box::new(SR5Rules),
    }
}

fn parse_edition(s: &str) -> Result<Edition, AppError> {
    match s {
        "SR4" => Ok(Edition::SR4),
        "SR5" => Ok(Edition::SR5),
        _ => Err(AppError::validation(format!("Unknown edition: {s}"))),
    }
}

fn parse_metatype(s: &str) -> Result<Metatype, AppError> {
    match s {
        "Human" => Ok(Metatype::Human),
        "Elf" => Ok(Metatype::Elf),
        "Dwarf" => Ok(Metatype::Dwarf),
        "Ork" => Ok(Metatype::Ork),
        "Troll" => Ok(Metatype::Troll),
        _ => Err(AppError::validation(format!("Unknown metatype: {s}"))),
    }
}

// -- IPC Commands --

#[tauri::command]
pub async fn create_campaign(
    name: String,
    state: State<'_, AppState>,
) -> Result<Campaign, AppError> {
    let id = uuid::Uuid::new_v4().to_string();

    // Create the campaign database file in a default location
    let db_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("personafix")
        .join("campaigns");
    std::fs::create_dir_all(&db_dir).map_err(|e| AppError {
        kind: "io".to_string(),
        message: e.to_string(),
    })?;

    let db_path = db_dir.join(format!("{id}.srx"));
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&db_url).await?;

    // Run migrations
    let migrations = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/data/migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations).await?;
    migrator.run(&pool).await?;

    // Insert campaign record
    sqlx::query("INSERT INTO campaigns (id, name) VALUES (?, ?)")
        .bind(&id)
        .bind(&name)
        .execute(&pool)
        .await?;

    // Set as active campaign
    *state.campaign_pool.write().await = Some(pool);
    *state.campaign_path.write().await = Some(db_path);

    Ok(Campaign { id, name })
}

#[tauri::command]
pub async fn open_campaign(path: String, state: State<'_, AppState>) -> Result<Campaign, AppError> {
    let db_path = PathBuf::from(&path);
    let db_url = format!("sqlite:{}?mode=rw", db_path.display());
    let pool = SqlitePool::connect(&db_url).await?;

    // Read campaign info
    let row: (String, String) = sqlx::query_as("SELECT id, name FROM campaigns LIMIT 1")
        .fetch_one(&pool)
        .await?;

    *state.campaign_pool.write().await = Some(pool);
    *state.campaign_path.write().await = Some(db_path);

    Ok(Campaign {
        id: row.0,
        name: row.1,
    })
}

#[tauri::command]
pub async fn list_characters(
    campaign_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<CharacterSummary>, AppError> {
    let pool = get_pool(&state).await?;

    let rows: Vec<(String, String, String, String)> =
        sqlx::query_as("SELECT id, name, edition, metatype FROM characters WHERE campaign_id = ?")
            .bind(&campaign_id)
            .fetch_all(&pool)
            .await?;

    let summaries = rows
        .into_iter()
        .map(|(id, name, edition, metatype)| CharacterSummary {
            id,
            name,
            edition: match edition.as_str() {
                "SR4" => Edition::SR4,
                _ => Edition::SR5,
            },
            metatype: match metatype.as_str() {
                "Human" => Metatype::Human,
                "Elf" => Metatype::Elf,
                "Dwarf" => Metatype::Dwarf,
                "Ork" => Metatype::Ork,
                "Troll" => Metatype::Troll,
                _ => Metatype::Human,
            },
            total_karma: 0,
        })
        .collect();

    Ok(summaries)
}

#[tauri::command]
pub async fn create_character(
    campaign_id: String,
    edition: String,
    name: String,
    metatype: String,
    state: State<'_, AppState>,
) -> Result<CharacterSummary, AppError> {
    let pool = get_pool(&state).await?;
    let edition_enum = parse_edition(&edition)?;
    let metatype_enum = parse_metatype(&metatype)?;
    let id = uuid::Uuid::new_v4().to_string();

    let rules = rules_for_edition(&edition_enum);
    let limits = rules.racial_limits(metatype_enum);

    // Create default attributes at racial minimums
    let attributes = Attributes {
        body: limits.body.0,
        agility: limits.agility.0,
        reaction: limits.reaction.0,
        strength: limits.strength.0,
        willpower: limits.willpower.0,
        logic: limits.logic.0,
        intuition: limits.intuition.0,
        charisma: limits.charisma.0,
        edge: limits.edge.0,
        essence: 600,
        magic: None,
        resonance: None,
    };

    let attributes_json = serde_json::to_string(&attributes)?;

    // Insert character record
    sqlx::query("INSERT INTO characters (id, campaign_id, edition, name) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&campaign_id)
        .bind(&edition)
        .bind(&name)
        .execute(&pool)
        .await?;

    // Insert character base
    sqlx::query(
        "INSERT INTO character_base (character_id, metatype, attributes_json) VALUES (?, ?, ?)",
    )
    .bind(&id)
    .bind(&metatype)
    .bind(&attributes_json)
    .execute(&pool)
    .await?;

    Ok(CharacterSummary {
        id,
        name,
        edition: edition_enum,
        metatype: metatype_enum,
        total_karma: 0,
    })
}

#[tauri::command]
pub async fn get_character(
    id: String,
    state: State<'_, AppState>,
) -> Result<ComputedCharacter, AppError> {
    let pool = get_pool(&state).await?;

    // Load character metadata
    let (edition_str, name, campaign_id, metatype_str): (String, String, String, String) =
        sqlx::query_as(
            "SELECT c.edition, c.name, c.campaign_id, cb.metatype \
             FROM characters c JOIN character_base cb ON c.id = cb.character_id \
             WHERE c.id = ?",
        )
        .bind(&id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("character", &id))?;

    let edition = parse_edition(&edition_str)?;
    let metatype = parse_metatype(&metatype_str)?;

    // Load attributes
    let (attributes_json,): (String,) =
        sqlx::query_as("SELECT attributes_json FROM character_base WHERE character_id = ?")
            .bind(&id)
            .fetch_one(&pool)
            .await?;

    let attributes: Attributes = serde_json::from_str(&attributes_json)?;

    let base = CharacterBase {
        id: id.clone(),
        campaign_id,
        name,
        edition,
        metatype,
        attributes,
        skills: vec![],
        skill_groups: vec![],
        qualities: vec![],
        augmentations: vec![],
        spells: vec![],
        adept_powers: vec![],
        complex_forms: vec![],
        contacts: vec![],
        weapons: vec![],
        armor: vec![],
        gear: vec![],
        vehicles: vec![],
        priority_selection: None,
    };

    // Load ledger events
    let ledger_rows: Vec<(String,)> =
        sqlx::query_as("SELECT payload_json FROM ledger WHERE character_id = ? ORDER BY id")
            .bind(&id)
            .fetch_all(&pool)
            .await?;

    let events: Vec<LedgerEvent> = ledger_rows
        .iter()
        .filter_map(|(json,)| serde_json::from_str(json).ok())
        .collect();

    // Project current state
    let rules = rules_for_edition(&edition);
    let computed = projection::project(&base, &events, rules.as_ref());

    Ok(computed)
}

#[tauri::command]
pub async fn apply_event(
    character_id: String,
    event: LedgerEvent,
    state: State<'_, AppState>,
) -> Result<ComputedCharacter, AppError> {
    let pool = get_pool(&state).await?;

    // Serialize and store the event
    let event_type = format!("{:?}", &event)
        .split('{')
        .next()
        .unwrap_or("Unknown")
        .trim()
        .to_string();
    let payload_json = serde_json::to_string(&event)?;

    let run_id: Option<String> = match &event {
        LedgerEvent::KarmaReceived { run_id, .. } => run_id.clone(),
        LedgerEvent::NuyenReceived { run_id, .. } => run_id.clone(),
        _ => None,
    };

    sqlx::query(
        "INSERT INTO ledger (character_id, event_type, payload_json, run_id) VALUES (?, ?, ?, ?)",
    )
    .bind(&character_id)
    .bind(&event_type)
    .bind(&payload_json)
    .bind(&run_id)
    .execute(&pool)
    .await?;

    // Re-project the character with all events
    get_character(character_id, state).await
}

#[tauri::command]
pub async fn get_ledger(
    character_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<LedgerEvent>, AppError> {
    let pool = get_pool(&state).await?;

    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT payload_json FROM ledger WHERE character_id = ? ORDER BY id")
            .bind(&character_id)
            .fetch_all(&pool)
            .await?;

    let events: Vec<LedgerEvent> = rows
        .iter()
        .filter_map(|(json,)| serde_json::from_str(json).ok())
        .collect();

    Ok(events)
}
