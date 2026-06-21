use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use tauri::State;

use personafix_core::ledger::events::LedgerEvent;
use personafix_core::ledger::projection;
use personafix_core::model::{
    attributes::{Attributes, Metatype, RacialLimits},
    character::{CharacterBase, CharacterDraft, CharacterSummary, ComputedCharacter},
    edition::Edition,
    validation::ValidationError,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentCampaign {
    pub path: String,
    pub name: String,
    pub last_opened: String,
}

// -- Helpers --

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

pub fn parse_edition(s: &str) -> Result<Edition, AppError> {
    match s {
        "SR4" => Ok(Edition::SR4),
        "SR5" => Ok(Edition::SR5),
        _ => Err(AppError::validation(format!("Unknown edition: {s}"))),
    }
}

pub fn parse_metatype(s: &str) -> Result<Metatype, AppError> {
    match s {
        "Human" => Ok(Metatype::Human),
        "Elf" => Ok(Metatype::Elf),
        "Dwarf" => Ok(Metatype::Dwarf),
        "Ork" => Ok(Metatype::Ork),
        "Troll" => Ok(Metatype::Troll),
        _ => Err(AppError::validation(format!("Unknown metatype: {s}"))),
    }
}

// ============================================================
// Core logic functions — testable without Tauri runtime
// ============================================================

// -- Recent campaigns helpers (filesystem, no pool needed) --

fn recent_campaigns_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("personafix").join("recent_campaigns.json"))
}

fn unix_now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

pub fn get_recent_campaigns_sync() -> Vec<RecentCampaign> {
    let path = match recent_campaigns_path() {
        Some(p) => p,
        None => return vec![],
    };
    if !path.exists() {
        return vec![];
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn record_recent_campaign_sync(path: &str, name: &str) {
    let json_path = match recent_campaigns_path() {
        Some(p) => p,
        None => return,
    };
    let mut recents = get_recent_campaigns_sync();
    recents.retain(|r| r.path != path);
    recents.insert(
        0,
        RecentCampaign {
            path: path.to_string(),
            name: name.to_string(),
            last_opened: unix_now_string(),
        },
    );
    recents.truncate(10);
    if let Some(parent) = json_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&recents) {
        let _ = std::fs::write(&json_path, json);
    }
}

// -- Open campaign (testable core) --

pub async fn open_campaign_db(pool: &SqlitePool) -> Result<Campaign, AppError> {
    let row: (String, String) = sqlx::query_as("SELECT id, name FROM campaigns LIMIT 1")
        .fetch_one(pool)
        .await?;
    Ok(Campaign { id: row.0, name: row.1 })
}

// -- JSON character backup / restore --

pub async fn export_character_json_db(
    pool: &SqlitePool,
    character_id: &str,
    out_path: &str,
) -> Result<(), AppError> {
    let computed = get_character_db(pool, character_id).await?;
    let events = get_ledger_db(pool, character_id).await?;

    #[derive(Serialize)]
    struct CharacterExport<'a> {
        version: u8,
        character: &'a CharacterBase,
        ledger: &'a [LedgerEvent],
    }

    let export = CharacterExport {
        version: 1,
        character: &computed.base,
        ledger: &events,
    };
    let json = serde_json::to_string_pretty(&export)?;
    std::fs::write(out_path, json).map_err(|e| AppError {
        kind: "io".to_string(),
        message: e.to_string(),
    })?;
    Ok(())
}

pub async fn import_character_json_db(
    pool: &SqlitePool,
    campaign_id: &str,
    file_path: &str,
) -> Result<String, AppError> {
    #[derive(Deserialize)]
    struct CharacterImport {
        #[allow(dead_code)]
        version: u8,
        character: CharacterBase,
        ledger: Vec<LedgerEvent>,
    }

    let json = std::fs::read_to_string(file_path).map_err(|e| AppError {
        kind: "io".to_string(),
        message: e.to_string(),
    })?;
    let import: CharacterImport = serde_json::from_str(&json).map_err(|e| AppError {
        kind: "json_parse".to_string(),
        message: e.to_string(),
    })?;

    let new_id = uuid::Uuid::new_v4().to_string();
    let mut new_base = import.character;

    create_character_db(
        pool,
        &new_id,
        campaign_id,
        &new_base.edition,
        &new_base.name,
        &new_base.metatype,
    )
    .await?;

    new_base.id = new_id.clone();
    new_base.campaign_id = campaign_id.to_string();
    save_character_base_db(pool, &new_base).await?;

    for event in import.ledger {
        apply_event_db(pool, &new_id, &event).await?;
    }

    Ok(new_id)
}

// -- Chummer character import / export --

pub async fn import_chummer_character_db(
    campaign_pool: &SqlitePool,
    game_data_pool: &SqlitePool,
    campaign_id: &str,
    chum_path: &str,
) -> Result<String, AppError> {
    use personafix_import_export::sr5;

    let path = std::path::Path::new(chum_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    let new_id = uuid::Uuid::new_v4().to_string();

    match ext.as_str() {
        "chum5" => {
            let chum = sr5::parse_file(path).map_err(|e| AppError {
                kind: "import".to_string(),
                message: e.to_string(),
            })?;
            // Create a placeholder record first so create_character_db sets up the row,
            // then import_sr5 builds the full CharacterBase and we overwrite with save.
            let metatype = match chum.metatype.as_str() {
                "Elf" => Metatype::Elf,
                "Dwarf" => Metatype::Dwarf,
                "Ork" | "Orc" => Metatype::Ork,
                "Troll" => Metatype::Troll,
                _ => Metatype::Human,
            };
            let char_name = if chum.name.is_empty() { "Imported Character".to_string() } else { chum.name.clone() };
            create_character_db(campaign_pool, &new_id, campaign_id, &Edition::SR5, &char_name, &metatype).await?;

            let base = sr5::map::import_sr5(&chum, game_data_pool, campaign_id, &new_id)
                .await
                .map_err(|e| AppError {
                    kind: "import".to_string(),
                    message: e.to_string(),
                })?;
            save_character_base_db(campaign_pool, &base).await?;
        }
        "chum" => {
            return Err(AppError {
                kind: "unsupported".to_string(),
                message: "SR4 .chum import is not yet supported".to_string(),
            });
        }
        _ => {
            return Err(AppError {
                kind: "unsupported".to_string(),
                message: format!("Unsupported file type: .{ext}"),
            });
        }
    }

    Ok(new_id)
}

pub async fn export_chummer_character_db(
    campaign_pool: &SqlitePool,
    game_data_pool: &SqlitePool,
    character_id: &str,
    out_path: &str,
) -> Result<(), AppError> {
    use personafix_import_export::export::sr5::export_sr5;

    let computed = get_character_db(campaign_pool, character_id).await?;
    let xml = export_sr5(&computed.base, game_data_pool)
        .await
        .map_err(|e| AppError {
            kind: "export".to_string(),
            message: e.to_string(),
        })?;
    std::fs::write(out_path, xml).map_err(|e| AppError {
        kind: "io".to_string(),
        message: e.to_string(),
    })?;
    Ok(())
}

/// Create a new campaign database at the given path, run migrations, insert record.
pub async fn create_campaign_db(
    pool: &SqlitePool,
    id: &str,
    name: &str,
) -> Result<Campaign, AppError> {
    let migrations =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../crates/data/migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations).await?;
    migrator.run(pool).await?;

    sqlx::query("INSERT INTO campaigns (id, name) VALUES (?, ?)")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await?;

    Ok(Campaign {
        id: id.to_string(),
        name: name.to_string(),
    })
}

pub async fn list_characters_db(
    pool: &SqlitePool,
    campaign_id: &str,
) -> Result<Vec<CharacterSummary>, AppError> {
    let rows: Vec<(String, String, String, String)> =
        sqlx::query_as("SELECT c.id, c.name, c.edition, cb.metatype FROM characters c JOIN character_base cb ON c.id = cb.character_id WHERE c.campaign_id = ?")
            .bind(campaign_id)
            .fetch_all(pool)
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

pub async fn create_character_db(
    pool: &SqlitePool,
    id: &str,
    campaign_id: &str,
    edition: &Edition,
    name: &str,
    metatype: &Metatype,
) -> Result<CharacterSummary, AppError> {
    let rules = rules_for_edition(edition);
    let limits = rules.racial_limits(*metatype);

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

    let edition_str = edition.to_string();
    let metatype_str = format!("{metatype:?}");
    let attributes_json = serde_json::to_string(&attributes)?;

    sqlx::query("INSERT INTO characters (id, campaign_id, edition, name) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(campaign_id)
        .bind(&edition_str)
        .bind(name)
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT INTO character_base (character_id, metatype, attributes_json) VALUES (?, ?, ?)",
    )
    .bind(id)
    .bind(&metatype_str)
    .bind(&attributes_json)
    .execute(pool)
    .await?;

    Ok(CharacterSummary {
        id: id.to_string(),
        name: name.to_string(),
        edition: *edition,
        metatype: *metatype,
        total_karma: 0,
    })
}

pub async fn get_character_db(pool: &SqlitePool, id: &str) -> Result<ComputedCharacter, AppError> {
    let (edition_str, name, campaign_id, metatype_str): (String, String, String, String) =
        sqlx::query_as(
            "SELECT c.edition, c.name, c.campaign_id, cb.metatype \
             FROM characters c JOIN character_base cb ON c.id = cb.character_id \
             WHERE c.id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("character", id))?;

    let edition = parse_edition(&edition_str)?;
    let metatype = parse_metatype(&metatype_str)?;

    // Load all JSON columns from character_base
    #[derive(FromRow)]
    struct CharacterBaseRow {
        attributes_json: String,
        skills_json: String,
        skill_groups_json: String,
        knowledge_skills_json: String,
        qualities_json: String,
        augmentations_json: String,
        spells_json: String,
        adept_powers_json: String,
        complex_forms_json: String,
        contacts_json: String,
        weapons_json: String,
        armor_json: String,
        gear_json: String,
        vehicles_json: String,
        priority_selection_json: Option<String>,
        magic_tradition: Option<String>,
        tradition_name: Option<String>,
        notes: String,
    }

    let row: CharacterBaseRow = sqlx::query_as(
        "SELECT attributes_json, skills_json, skill_groups_json, knowledge_skills_json, \
             qualities_json, augmentations_json, spells_json, adept_powers_json, \
             complex_forms_json, contacts_json, weapons_json, armor_json, gear_json, \
             vehicles_json, priority_selection_json, magic_tradition, tradition_name, notes \
             FROM character_base WHERE character_id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    let base = CharacterBase {
        id: id.to_string(),
        campaign_id,
        name,
        edition,
        metatype,
        attributes: serde_json::from_str(&row.attributes_json)?,
        skills: serde_json::from_str(&row.skills_json)?,
        skill_groups: serde_json::from_str(&row.skill_groups_json)?,
        knowledge_skills: serde_json::from_str(&row.knowledge_skills_json)?,
        qualities: serde_json::from_str(&row.qualities_json)?,
        augmentations: serde_json::from_str(&row.augmentations_json)?,
        spells: serde_json::from_str(&row.spells_json)?,
        adept_powers: serde_json::from_str(&row.adept_powers_json)?,
        complex_forms: serde_json::from_str(&row.complex_forms_json)?,
        contacts: serde_json::from_str(&row.contacts_json)?,
        weapons: serde_json::from_str(&row.weapons_json)?,
        armor: serde_json::from_str(&row.armor_json)?,
        gear: serde_json::from_str(&row.gear_json)?,
        vehicles: serde_json::from_str(&row.vehicles_json)?,
        priority_selection: row.priority_selection_json.as_deref().and_then(|s| serde_json::from_str(s).ok()),
        magic_tradition: row.magic_tradition.as_deref().and_then(|s| serde_json::from_str(s).ok()),
        tradition_name: row.tradition_name,
        notes: row.notes,
    };

    let ledger_rows: Vec<(String,)> =
        sqlx::query_as("SELECT payload_json FROM ledger WHERE character_id = ? ORDER BY id")
            .bind(id)
            .fetch_all(pool)
            .await?;

    let events: Vec<LedgerEvent> = ledger_rows
        .iter()
        .filter_map(|(json,)| serde_json::from_str(json).ok())
        .collect();

    let rules = rules_for_edition(&edition);
    let computed = projection::project(&base, &events, rules.as_ref());

    Ok(computed)
}

pub async fn apply_event_db(
    pool: &SqlitePool,
    character_id: &str,
    event: &LedgerEvent,
) -> Result<(), AppError> {
    let event_type = format!("{event:?}")
        .split('{')
        .next()
        .unwrap_or("Unknown")
        .trim()
        .to_string();
    let payload_json = serde_json::to_string(event)?;

    let run_id: Option<String> = match event {
        LedgerEvent::KarmaReceived { run_id, .. } => run_id.clone(),
        LedgerEvent::NuyenReceived { run_id, .. } => run_id.clone(),
        _ => None,
    };

    sqlx::query(
        "INSERT INTO ledger (character_id, event_type, payload_json, run_id) VALUES (?, ?, ?, ?)",
    )
    .bind(character_id)
    .bind(&event_type)
    .bind(&payload_json)
    .bind(&run_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_ledger_db(
    pool: &SqlitePool,
    character_id: &str,
) -> Result<Vec<LedgerEvent>, AppError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT payload_json FROM ledger WHERE character_id = ? ORDER BY id")
            .bind(character_id)
            .fetch_all(pool)
            .await?;

    let events: Vec<LedgerEvent> = rows
        .iter()
        .filter_map(|(json,)| serde_json::from_str(json).ok())
        .collect();

    Ok(events)
}

/// Get racial attribute limits for a metatype in a given edition.
pub fn get_racial_limits_for(edition: &Edition, metatype: &Metatype) -> RacialLimits {
    let rules = rules_for_edition(edition);
    rules.racial_limits(*metatype)
}

/// Validate a character draft using the appropriate edition's rules.
pub fn validate_draft_with_rules(draft: &CharacterDraft) -> Vec<ValidationError> {
    let rules = rules_for_edition(&draft.edition);
    rules.validate_creation(draft)
}

/// Save a finalized character base to the database (overwriting any existing base).
pub async fn save_character_base_db(
    pool: &SqlitePool,
    base: &CharacterBase,
) -> Result<(), AppError> {
    let metatype_str = format!("{:?}", base.metatype);
    let attributes_json = serde_json::to_string(&base.attributes)?;
    let skills_json = serde_json::to_string(&base.skills)?;
    let skill_groups_json = serde_json::to_string(&base.skill_groups)?;
    let knowledge_skills_json = serde_json::to_string(&base.knowledge_skills)?;
    let qualities_json = serde_json::to_string(&base.qualities)?;
    let augmentations_json = serde_json::to_string(&base.augmentations)?;
    let spells_json = serde_json::to_string(&base.spells)?;
    let adept_powers_json = serde_json::to_string(&base.adept_powers)?;
    let complex_forms_json = serde_json::to_string(&base.complex_forms)?;
    let contacts_json = serde_json::to_string(&base.contacts)?;
    let weapons_json = serde_json::to_string(&base.weapons)?;
    let armor_json = serde_json::to_string(&base.armor)?;
    let gear_json = serde_json::to_string(&base.gear)?;
    let vehicles_json = serde_json::to_string(&base.vehicles)?;
    let priority_json = base
        .priority_selection
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    let magic_tradition_json = base
        .magic_tradition
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;

    sqlx::query(
        "UPDATE character_base SET \
         metatype = ?, attributes_json = ?, skills_json = ?, skill_groups_json = ?, \
         knowledge_skills_json = ?, \
         qualities_json = ?, augmentations_json = ?, spells_json = ?, adept_powers_json = ?, \
         complex_forms_json = ?, contacts_json = ?, weapons_json = ?, armor_json = ?, \
         gear_json = ?, vehicles_json = ?, priority_selection_json = ?, magic_tradition = ?, \
         tradition_name = ?, notes = ? \
         WHERE character_id = ?",
    )
    .bind(&metatype_str)
    .bind(&attributes_json)
    .bind(&skills_json)
    .bind(&skill_groups_json)
    .bind(&knowledge_skills_json)
    .bind(&qualities_json)
    .bind(&augmentations_json)
    .bind(&spells_json)
    .bind(&adept_powers_json)
    .bind(&complex_forms_json)
    .bind(&contacts_json)
    .bind(&weapons_json)
    .bind(&armor_json)
    .bind(&gear_json)
    .bind(&vehicles_json)
    .bind(&priority_json)
    .bind(&magic_tradition_json)
    .bind(&base.tradition_name)
    .bind(&base.notes)
    .bind(&base.id)
    .execute(pool)
    .await?;

    Ok(())
}

// ============================================================
// Notes update
// ============================================================

pub async fn update_notes_db(
    pool: &SqlitePool,
    character_id: &str,
    notes: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE character_base SET notes = ? WHERE character_id = ?")
        .bind(notes)
        .bind(character_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn update_notes(
    character_id: &str,
    notes: &str,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let pool = get_pool(&state).await?;
    update_notes_db(&pool, character_id, notes).await
}

pub async fn update_tradition_name_db(
    pool: &SqlitePool,
    character_id: &str,
    tradition_name: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query("UPDATE character_base SET tradition_name = ? WHERE character_id = ?")
        .bind(tradition_name)
        .bind(character_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn update_tradition_name(
    character_id: &str,
    tradition_name: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let pool = get_pool(&state).await?;
    update_tradition_name_db(&pool, character_id, tradition_name.as_deref()).await
}

// ============================================================
// Game data query functions
// ============================================================

/// Skill record from the game data DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSkill {
    pub id: String,
    pub name: String,
    pub linked_attribute: String,
    pub skill_group: Option<String>,
    pub source: String,
    pub page: String,
}

/// Quality record from the game data DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameQuality {
    pub id: String,
    pub name: String,
    pub quality_type: String,
    pub cost: i32,
    pub source: String,
    pub page: String,
    pub incompatible_with: Vec<String>,
}

/// Weapon record from the game data DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWeapon {
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

/// Augmentation record from the game data DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAugmentation {
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

pub async fn query_skills_db(pool: &SqlitePool, edition: &str) -> Result<Vec<GameSkill>, AppError> {
    let rows: Vec<(String, String, String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT id, name, linked_attribute, skill_group, source, page \
         FROM skills_data WHERE edition = ? ORDER BY name",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, name, linked_attribute, skill_group, source, page)| GameSkill {
                id,
                name,
                linked_attribute,
                skill_group,
                source,
                page,
            },
        )
        .collect())
}

pub async fn query_qualities_db(
    pool: &SqlitePool,
    edition: &str,
) -> Result<Vec<GameQuality>, AppError> {
    let rows: Vec<(String, String, String, i32, String, String, String)> = sqlx::query_as(
        "SELECT id, name, quality_type, cost, source, page, incompatible_with_json \
         FROM qualities WHERE edition = ? ORDER BY quality_type, name",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, name, quality_type, cost, source, page, incompat_json)| {
                let incompatible_with: Vec<String> =
                    serde_json::from_str(&incompat_json).unwrap_or_default();
                GameQuality {
                    id,
                    name,
                    quality_type,
                    cost,
                    source,
                    page,
                    incompatible_with,
                }
            },
        )
        .collect())
}

pub async fn query_weapons_db(
    pool: &SqlitePool,
    edition: &str,
) -> Result<Vec<GameWeapon>, AppError> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, String, String, String, String, String, String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, category, damage, ap, mode, recoil_comp, ammo, availability, cost, source, page \
         FROM weapons WHERE edition = ? ORDER BY category, name",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                name,
                category,
                damage,
                ap,
                mode,
                recoil_comp,
                ammo,
                availability,
                cost,
                source,
                page,
            )| GameWeapon {
                id,
                name,
                category,
                damage,
                ap,
                mode,
                recoil_comp,
                ammo,
                availability,
                cost,
                source,
                page,
            },
        )
        .collect())
}

pub async fn query_augmentations_db(
    pool: &SqlitePool,
    edition: &str,
) -> Result<Vec<GameAugmentation>, AppError> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, String, String, String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, augmentation_type, essence_cost, capacity, availability, cost, source, page \
         FROM augmentations WHERE edition = ? ORDER BY augmentation_type, name",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(
                id,
                name,
                augmentation_type,
                essence_cost,
                capacity,
                availability,
                cost,
                source,
                page,
            )| GameAugmentation {
                id,
                name,
                augmentation_type,
                essence_cost,
                capacity,
                availability,
                cost,
                source,
                page,
            },
        )
        .collect())
}

// ============================================================
// Tauri command wrappers — thin layer over core logic
// ============================================================

#[tauri::command]
pub async fn create_campaign(
    name: String,
    state: State<'_, AppState>,
) -> Result<Campaign, AppError> {
    let id = uuid::Uuid::new_v4().to_string();

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

    let campaign = create_campaign_db(&pool, &id, &name).await?;

    record_recent_campaign_sync(&db_path.to_string_lossy(), &campaign.name);

    *state.campaign_pool.write().await = Some(pool);
    *state.campaign_path.write().await = Some(db_path);

    Ok(campaign)
}

#[tauri::command]
pub async fn open_campaign(path: String, state: State<'_, AppState>) -> Result<Campaign, AppError> {
    let db_path = PathBuf::from(&path);
    let db_url = format!("sqlite:{}?mode=rw", db_path.display());
    let pool = SqlitePool::connect(&db_url).await?;

    let row: (String, String) = sqlx::query_as("SELECT id, name FROM campaigns LIMIT 1")
        .fetch_one(&pool)
        .await?;

    record_recent_campaign_sync(&path, &row.1);

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
    list_characters_db(&pool, &campaign_id).await
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
    create_character_db(
        &pool,
        &id,
        &campaign_id,
        &edition_enum,
        &name,
        &metatype_enum,
    )
    .await
}

#[tauri::command]
pub async fn get_character(
    id: String,
    state: State<'_, AppState>,
) -> Result<ComputedCharacter, AppError> {
    let pool = get_pool(&state).await?;
    get_character_db(&pool, &id).await
}

#[tauri::command]
pub async fn apply_event(
    character_id: String,
    event: LedgerEvent,
    state: State<'_, AppState>,
) -> Result<ComputedCharacter, AppError> {
    let pool = get_pool(&state).await?;
    apply_event_db(&pool, &character_id, &event).await?;
    get_character_db(&pool, &character_id).await
}

#[tauri::command]
pub async fn get_ledger(
    character_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<LedgerEvent>, AppError> {
    let pool = get_pool(&state).await?;
    get_ledger_db(&pool, &character_id).await
}

#[tauri::command]
pub fn get_racial_limits(edition: String, metatype: String) -> Result<RacialLimits, AppError> {
    let edition_enum = parse_edition(&edition)?;
    let metatype_enum = parse_metatype(&metatype)?;
    Ok(get_racial_limits_for(&edition_enum, &metatype_enum))
}

#[tauri::command]
pub fn validate_draft(draft: CharacterDraft) -> Vec<ValidationError> {
    validate_draft_with_rules(&draft)
}

#[tauri::command]
pub async fn save_character_base(
    base: CharacterBase,
    state: State<'_, AppState>,
) -> Result<ComputedCharacter, AppError> {
    let pool = get_pool(&state).await?;
    save_character_base_db(&pool, &base).await?;
    get_character_db(&pool, &base.id).await
}

async fn get_game_pool(state: &State<'_, AppState>) -> Result<SqlitePool, AppError> {
    let guard = state.game_data_pool.read().await;
    guard.clone().ok_or_else(|| AppError {
        kind: "no_game_data".to_string(),
        message: "Game data not loaded. Run the migration tool first.".to_string(),
    })
}

#[tauri::command]
pub async fn load_game_data(path: String, state: State<'_, AppState>) -> Result<String, AppError> {
    let abs_path = resolve_path(&path);

    if !abs_path.exists() {
        return Err(AppError {
            kind: "file_not_found".to_string(),
            message: format!(
                "Game data file not found at: {}\nProject root: {}\nOriginal path: {}",
                abs_path.display(),
                project_root().display(),
                path
            ),
        });
    }

    let db_url = format!("sqlite:{}?mode=ro", abs_path.display());
    let pool = SqlitePool::connect(&db_url).await.map_err(|e| AppError {
        kind: "database".to_string(),
        message: format!("Failed to open {}: {}", abs_path.display(), e),
    })?;

    // Verify it has game data by checking a table
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM skills_data")
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError {
            kind: "database".to_string(),
            message: format!("File opened but doesn't look like a game data DB: {e}"),
        })?;

    *state.game_data_pool.write().await = Some(pool);

    Ok(format!(
        "Loaded game data from {} ({} skills)",
        abs_path.display(),
        count.0
    ))
}

/// Project root directory (workspace root, not src-tauri/).
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
}

/// Resolve a path: absolute paths used as-is, relative paths resolved
/// against the project root (not the working directory, which Tauri
/// sets to src-tauri/).
fn resolve_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        project_root().join(&p)
    }
}

/// Debug command: check if a file exists and return info about paths.
#[tauri::command]
pub fn debug_check_file(path: String) -> Result<String, AppError> {
    let abs_path = resolve_path(&path);
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let exists = abs_path.exists();
    let is_file = abs_path.is_file();
    let size = if exists {
        std::fs::metadata(&abs_path)
            .map(|m| format!("{} bytes", m.len()))
            .unwrap_or_else(|e| format!("error: {e}"))
    } else {
        "N/A".to_string()
    };

    Ok(format!(
        "Input path: {}\nProject root: {}\nWorking dir: {}\nResolved to: {}\nExists: {}\nIs file: {}\nSize: {}",
        path,
        project_root().display(),
        cwd,
        abs_path.display(),
        exists,
        is_file,
        size
    ))
}

#[tauri::command]
pub async fn get_skills(
    edition: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameSkill>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_skills_db(&pool, &edition).await
}

#[tauri::command]
pub async fn get_qualities(
    edition: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameQuality>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_qualities_db(&pool, &edition).await
}

#[tauri::command]
pub async fn get_weapons(
    edition: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameWeapon>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_weapons_db(&pool, &edition).await
}

#[tauri::command]
pub async fn get_augmentations(
    edition: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameAugmentation>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_augmentations_db(&pool, &edition).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameArmor {
    pub id: String,
    pub name: String,
    pub armor_value: String,
    pub availability: String,
    pub cost: String,
    pub source: String,
    pub page: String,
}

pub async fn query_armor_db(
    pool: &SqlitePool,
    edition: &str,
) -> Result<Vec<GameArmor>, AppError> {
    let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, armor_value, availability, cost, source, page \
         FROM armor WHERE edition = ? ORDER BY name",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, armor_value, availability, cost, source, page)| GameArmor {
            id,
            name,
            armor_value,
            availability,
            cost,
            source,
            page,
        })
        .collect())
}

#[tauri::command]
pub async fn get_armor(
    edition: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameArmor>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_armor_db(&pool, &edition).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSpell {
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

type SpellRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

pub async fn query_spells_db(
    pool: &SqlitePool,
    edition: &str,
) -> Result<Vec<GameSpell>, AppError> {
    let rows: Vec<SpellRow> = sqlx::query_as(
        "SELECT id, name, category, spell_type, range, damage, duration, drain, source, page \
         FROM spells WHERE edition = ? ORDER BY category, name",
    )
    .bind(edition)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, name, category, spell_type, range, damage, duration, drain, source, page)| {
                GameSpell {
                    id,
                    name,
                    category,
                    spell_type,
                    range,
                    damage,
                    duration,
                    drain,
                    source,
                    page,
                }
            },
        )
        .collect())
}

#[tauri::command]
pub async fn get_spells(
    edition: String,
    state: State<'_, AppState>,
) -> Result<Vec<GameSpell>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_spells_db(&pool, &edition).await
}

/// Adept power record from the game data DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAdeptPower {
    pub id: String,
    pub name: String,
    /// Power point cost as decimal string (e.g. "0.25", "1.00").
    pub cost: String,
    pub levels: bool,
    pub source: String,
    pub page: String,
}

pub async fn query_adept_powers_db(
    pool: &SqlitePool,
) -> Result<Vec<GameAdeptPower>, AppError> {
    let rows: Vec<(String, String, String, i32, String, String)> = sqlx::query_as(
        "SELECT id, name, cost, levels, source, page \
         FROM adept_powers WHERE edition = 'SR5' ORDER BY name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, cost, levels, source, page)| GameAdeptPower {
            id,
            name,
            cost,
            levels: levels != 0,
            source,
            page,
        })
        .collect())
}

#[tauri::command]
pub async fn get_adept_powers(
    state: State<'_, AppState>,
) -> Result<Vec<GameAdeptPower>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_adept_powers_db(&pool).await
}

/// Complex form record from the game data DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameComplexForm {
    pub id: String,
    pub name: String,
    pub target: String,
    pub duration: String,
    /// Fading value (e.g. "L-2").
    pub fading: String,
    pub source: String,
    pub page: String,
}

pub async fn query_complex_forms_db(
    pool: &SqlitePool,
) -> Result<Vec<GameComplexForm>, AppError> {
    let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, target, duration, fading, source, page \
         FROM complex_forms WHERE edition = 'SR5' ORDER BY name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, target, duration, fading, source, page)| GameComplexForm {
            id,
            name,
            target,
            duration,
            fading,
            source,
            page,
        })
        .collect())
}

#[tauri::command]
pub async fn get_complex_forms(
    state: State<'_, AppState>,
) -> Result<Vec<GameComplexForm>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_complex_forms_db(&pool).await
}

/// Vehicle record from the game data DB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVehicle {
    pub id: String,
    pub name: String,
    pub handling: String,
    pub speed: String,
    pub acceleration: String,
    pub body: String,
    pub armor: String,
    pub pilot: String,
    pub sensor: String,
    pub availability: String,
    pub cost: String,
    pub edition: String,
    pub source: String,
    pub page: String,
}

pub async fn query_vehicles_db(pool: &SqlitePool) -> Result<Vec<GameVehicle>, AppError> {
    type VehicleRow = (String, String, String, String, String, String, String, String, String, String, String, String, String);
    let rows: Vec<VehicleRow> = sqlx::query_as(
        "SELECT id, name, handling, speed, acceleration, body, armor, pilot, sensor, \
         availability, cost, source, page FROM vehicles ORDER BY name",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, name, handling, speed, acceleration, body, armor, pilot, sensor, availability, cost, source, page)| {
                GameVehicle { id, name, handling, speed, acceleration, body, armor, pilot, sensor, availability, cost, edition: String::new(), source, page }
            },
        )
        .collect())
}

#[tauri::command]
pub async fn get_vehicles(
    state: State<'_, AppState>,
) -> Result<Vec<GameVehicle>, AppError> {
    let pool = get_game_pool(&state).await?;
    query_vehicles_db(&pool).await
}

#[tauri::command]
pub async fn get_recent_campaigns() -> Result<Vec<RecentCampaign>, AppError> {
    Ok(get_recent_campaigns_sync())
}

#[tauri::command]
pub async fn record_recent_campaign(path: String, name: String) -> Result<(), AppError> {
    record_recent_campaign_sync(&path, &name);
    Ok(())
}

#[tauri::command]
pub async fn export_character_json(
    character_id: String,
    out_path: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let pool = get_pool(&state).await?;
    export_character_json_db(&pool, &character_id, &out_path).await
}

#[tauri::command]
pub async fn import_character_json(
    campaign_id: String,
    file_path: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let pool = get_pool(&state).await?;
    import_character_json_db(&pool, &campaign_id, &file_path).await
}

#[tauri::command]
pub async fn import_chummer_character(
    campaign_id: String,
    chum_path: String,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let pool = get_pool(&state).await?;
    let game_pool = get_game_pool(&state).await?;
    import_chummer_character_db(&pool, &game_pool, &campaign_id, &chum_path).await
}

#[tauri::command]
pub async fn export_chummer_character(
    character_id: String,
    out_path: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let pool = get_pool(&state).await?;
    let game_pool = get_game_pool(&state).await?;
    export_chummer_character_db(&pool, &game_pool, &character_id, &out_path).await
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use personafix_core::model::edition::Edition;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let migrations =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../crates/data/migrations");
        let migrator = sqlx::migrate::Migrator::new(migrations).await.unwrap();
        migrator.run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_create_campaign_db() {
        let pool = setup_test_db().await;
        let campaign = create_campaign_db(&pool, "c1", "Test Campaign")
            .await
            .unwrap();
        assert_eq!(campaign.id, "c1");
        assert_eq!(campaign.name, "Test Campaign");

        // Verify it's in the database
        let (name,): (String,) = sqlx::query_as("SELECT name FROM campaigns WHERE id = 'c1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(name, "Test Campaign");
    }

    #[tokio::test]
    async fn test_create_and_list_characters() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();

        // Create two characters
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Runner",
            &Metatype::Human,
        )
        .await
        .unwrap();
        create_character_db(&pool, "ch2", "c1", &Edition::SR5, "Adept", &Metatype::Elf)
            .await
            .unwrap();

        let chars = list_characters_db(&pool, "c1").await.unwrap();
        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0].name, "Runner");
        assert_eq!(chars[0].edition, Edition::SR4);
        assert_eq!(chars[1].name, "Adept");
        assert_eq!(chars[1].metatype, Metatype::Elf);
    }

    #[tokio::test]
    async fn test_create_character_sets_racial_minimum_attributes() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();

        // Create an Ork (SR4: BOD 3-8, STR 3-8)
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Ork Runner",
            &Metatype::Ork,
        )
        .await
        .unwrap();

        let computed = get_character_db(&pool, "ch1").await.unwrap();
        assert_eq!(computed.computed_attributes.body, 3); // Ork minimum
        assert_eq!(computed.computed_attributes.strength, 3); // Ork minimum
        assert_eq!(computed.computed_attributes.charisma, 1); // Ork CHA min
    }

    #[tokio::test]
    async fn test_get_character_computes_derived_stats() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Runner",
            &Metatype::Human,
        )
        .await
        .unwrap();

        let computed = get_character_db(&pool, "ch1").await.unwrap();

        // Human minimums: all 1, Edge 2. Body 1 → physical CM = 8 + ceil(1/2) = 9
        assert_eq!(computed.physical_condition_monitor, 9);
        // Willpower 1 → stun CM = 9
        assert_eq!(computed.stun_condition_monitor, 9);
        // Initiative: REA 1 + INT 1 = 2
        assert_eq!(computed.initiative, 2);
        // Full essence (no augmentations)
        assert_eq!(computed.computed_attributes.essence, 600);
    }

    #[tokio::test]
    async fn test_get_character_not_found() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();

        let result = get_character_db(&pool, "nonexistent").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, "not_found");
    }

    #[tokio::test]
    async fn test_apply_event_and_get_character() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Runner",
            &Metatype::Human,
        )
        .await
        .unwrap();

        // Apply karma received event
        let event = LedgerEvent::KarmaReceived {
            amount: 10,
            reason: "Run reward".to_string(),
            run_id: Some("run1".to_string()),
        };
        apply_event_db(&pool, "ch1", &event).await.unwrap();

        // Apply karma spent event
        let event2 = LedgerEvent::KarmaSpent {
            amount: 6,
            description: "Skill improvement".to_string(),
        };
        apply_event_db(&pool, "ch1", &event2).await.unwrap();

        // Get character — should have projected karma totals
        let computed = get_character_db(&pool, "ch1").await.unwrap();
        assert_eq!(computed.total_karma_earned, 10);
        assert_eq!(computed.total_karma_spent, 6);
    }

    #[tokio::test]
    async fn test_get_ledger_returns_events_in_order() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Runner",
            &Metatype::Human,
        )
        .await
        .unwrap();

        apply_event_db(
            &pool,
            "ch1",
            &LedgerEvent::KarmaReceived {
                amount: 5,
                reason: "Run 1".to_string(),
                run_id: None,
            },
        )
        .await
        .unwrap();

        apply_event_db(
            &pool,
            "ch1",
            &LedgerEvent::NuyenReceived {
                amount: 10_000,
                reason: "Payment".to_string(),
                run_id: None,
            },
        )
        .await
        .unwrap();

        let events = get_ledger_db(&pool, "ch1").await.unwrap();
        assert_eq!(events.len(), 2);
        matches!(&events[0], LedgerEvent::KarmaReceived { amount: 5, .. });
        matches!(
            &events[1],
            LedgerEvent::NuyenReceived { amount: 10_000, .. }
        );
    }

    #[tokio::test]
    async fn test_apply_event_persists_and_projects() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Runner",
            &Metatype::Human,
        )
        .await
        .unwrap();

        // Apply attribute improvement
        apply_event_db(
            &pool,
            "ch1",
            &LedgerEvent::AttributeImproved {
                attribute: "body".to_string(),
                from: 1,
                to: 4,
                karma_cost: 50,
            },
        )
        .await
        .unwrap();

        let computed = get_character_db(&pool, "ch1").await.unwrap();
        // Body improved from 1 to 4 → physical CM = 8 + ceil(4/2) = 10
        assert_eq!(computed.computed_attributes.body, 4);
        assert_eq!(computed.physical_condition_monitor, 10);
        assert_eq!(computed.total_karma_spent, 50);
    }

    #[tokio::test]
    async fn test_full_career_round_trip() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Street Sam",
            &Metatype::Human,
        )
        .await
        .unwrap();

        // Run 1
        for event in [
            LedgerEvent::RunCompleted {
                run_id: "r1".to_string(),
                name: "Milk Run".to_string(),
                date: "2078-01-15".to_string(),
                notes: String::new(),
            },
            LedgerEvent::KarmaReceived {
                amount: 5,
                reason: "Run reward".to_string(),
                run_id: Some("r1".to_string()),
            },
            LedgerEvent::NuyenReceived {
                amount: 8_000,
                reason: "Payment".to_string(),
                run_id: Some("r1".to_string()),
            },
        ] {
            apply_event_db(&pool, "ch1", &event).await.unwrap();
        }

        // Run 2
        for event in [
            LedgerEvent::KarmaReceived {
                amount: 8,
                reason: "Run 2".to_string(),
                run_id: None,
            },
            LedgerEvent::NuyenReceived {
                amount: 15_000,
                reason: "Run 2".to_string(),
                run_id: None,
            },
            LedgerEvent::NuyenSpent {
                amount: 3_000,
                description: "Gear".to_string(),
            },
        ] {
            apply_event_db(&pool, "ch1", &event).await.unwrap();
        }

        // Run 3
        apply_event_db(
            &pool,
            "ch1",
            &LedgerEvent::KarmaReceived {
                amount: 10,
                reason: "Run 3".to_string(),
                run_id: None,
            },
        )
        .await
        .unwrap();

        let computed = get_character_db(&pool, "ch1").await.unwrap();
        assert_eq!(computed.total_karma_earned, 23); // 5+8+10
        assert_eq!(computed.nuyen, 20_000); // 8k+15k-3k

        let events = get_ledger_db(&pool, "ch1").await.unwrap();
        assert_eq!(events.len(), 7);
    }

    // -- Tests for new builder commands --

    #[test]
    fn test_get_racial_limits_sr4_human() {
        let limits = get_racial_limits_for(&Edition::SR4, &Metatype::Human);
        assert_eq!(limits.body, (1, 6));
        assert_eq!(limits.edge, (2, 7));
    }

    #[test]
    fn test_get_racial_limits_sr5_ork() {
        let limits = get_racial_limits_for(&Edition::SR5, &Metatype::Ork);
        assert_eq!(limits.body, (4, 9));
        assert_eq!(limits.strength, (3, 8));
    }

    #[test]
    fn test_validate_draft_legal_sr4() {
        let draft = CharacterDraft {
            name: "Test".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 3,
                agility: 3,
                reaction: 3,
                strength: 3,
                willpower: 3,
                logic: 3,
                intuition: 3,
                charisma: 3,
                edge: 3,
                essence: 600,
                magic: None,
                resonance: None,
            },
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
            knowledge_skills: vec![],
            priority_selection: None,
            magic_tradition: None,
            tradition_name: None,
            creation_points_spent: 0,
            nuyen_spent: 0,
        };
        let errors = validate_draft_with_rules(&draft);
        let real_errors: Vec<_> = errors
            .iter()
            .filter(|e| e.severity == personafix_core::model::validation::ValidationSeverity::Error)
            .collect();
        assert!(
            real_errors.is_empty(),
            "Expected no errors, got: {real_errors:?}"
        );
    }

    #[test]
    fn test_validate_draft_catches_attribute_over_max() {
        let draft = CharacterDraft {
            name: "Test".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 8, // Over max of 6
                agility: 1,
                reaction: 1,
                strength: 1,
                willpower: 1,
                logic: 1,
                intuition: 1,
                charisma: 1,
                edge: 2,
                essence: 600,
                magic: None,
                resonance: None,
            },
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
            knowledge_skills: vec![],
            priority_selection: None,
            magic_tradition: None,
            tradition_name: None,
            creation_points_spent: 0,
            nuyen_spent: 0,
        };
        let errors = validate_draft_with_rules(&draft);
        assert!(errors.iter().any(|e| e.field == "attributes.body"));
    }

    #[tokio::test]
    async fn test_save_character_base_persists_skills() {
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Runner",
            &Metatype::Human,
        )
        .await
        .unwrap();

        // Build a base with skills
        use personafix_core::model::skills::Skill;
        let base = CharacterBase {
            id: "ch1".to_string(),
            campaign_id: "c1".to_string(),
            name: "Runner".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 4,
                agility: 5,
                reaction: 3,
                strength: 3,
                willpower: 3,
                logic: 2,
                intuition: 4,
                charisma: 2,
                edge: 3,
                essence: 600,
                magic: None,
                resonance: None,
            },
            skills: vec![Skill {
                id: "pistols".to_string(),
                name: "Pistols".to_string(),
                linked_attribute: "AGI".to_string(),
                group: None,
                rating: 5,
                specializations: vec![],
            }],
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
            knowledge_skills: vec![],
            priority_selection: None,
            magic_tradition: None,
            tradition_name: None,
            notes: String::new(),
        };

        save_character_base_db(&pool, &base).await.unwrap();

        // Retrieve and verify
        let computed = get_character_db(&pool, "ch1").await.unwrap();
        assert_eq!(computed.computed_attributes.body, 4);
        assert_eq!(computed.computed_attributes.agility, 5);
        assert_eq!(computed.physical_condition_monitor, 10); // 8 + ceil(4/2)
        assert_eq!(computed.initiative, 7); // REA 3 + INT 4
    }

    #[tokio::test]
    async fn test_save_full_character_with_qualities_round_trips() {
        use personafix_core::model::qualities::{Quality, QualityType};
        use personafix_core::model::skills::Skill;

        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(
            &pool,
            "ch1",
            "c1",
            &Edition::SR4,
            "Street Sam",
            &Metatype::Human,
        )
        .await
        .unwrap();

        let base = CharacterBase {
            id: "ch1".to_string(),
            campaign_id: "c1".to_string(),
            name: "Street Sam".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 5,
                agility: 5,
                reaction: 4,
                strength: 4,
                willpower: 3,
                logic: 2,
                intuition: 4,
                charisma: 2,
                edge: 3,
                essence: 600,
                magic: None,
                resonance: None,
            },
            skills: vec![
                Skill {
                    id: "pistols".to_string(),
                    name: "Pistols".to_string(),
                    linked_attribute: "AGI".to_string(),
                    group: None,
                    rating: 5,
                    specializations: vec![],
                },
                Skill {
                    id: "dodge".to_string(),
                    name: "Dodge".to_string(),
                    linked_attribute: "REA".to_string(),
                    group: None,
                    rating: 4,
                    specializations: vec![],
                },
            ],
            skill_groups: vec![],
            qualities: vec![
                Quality {
                    id: "ambidextrous".to_string(),
                    name: "Ambidextrous".to_string(),
                    quality_type: QualityType::Positive,
                    cost: 5,
                    source: "SR4".to_string(),
                    page: "90".to_string(),
                    improvements: vec![],
                    incompatible_with: vec![],
                },
                Quality {
                    id: "sinner".to_string(),
                    name: "SINner".to_string(),
                    quality_type: QualityType::Negative,
                    cost: 5,
                    source: "SR4".to_string(),
                    page: "91".to_string(),
                    improvements: vec![],
                    incompatible_with: vec![],
                },
            ],
            augmentations: vec![],
            spells: vec![],
            adept_powers: vec![],
            complex_forms: vec![],
            contacts: vec![],
            weapons: vec![],
            armor: vec![],
            gear: vec![],
            vehicles: vec![],
            knowledge_skills: vec![],
            priority_selection: None,
            magic_tradition: None,
            tradition_name: None,
            notes: String::new(),
        };

        save_character_base_db(&pool, &base).await.unwrap();

        // Read back and verify everything persisted
        let computed = get_character_db(&pool, "ch1").await.unwrap();
        assert_eq!(computed.base.skills.len(), 2);
        assert_eq!(computed.base.skills[0].name, "Pistols");
        assert_eq!(computed.base.skills[0].rating, 5);
        assert_eq!(computed.base.skills[1].name, "Dodge");
        assert_eq!(computed.base.qualities.len(), 2);
        assert_eq!(computed.base.qualities[0].name, "Ambidextrous");
        assert_eq!(
            computed.base.qualities[1].quality_type,
            QualityType::Negative
        );
        assert_eq!(computed.computed_attributes.body, 5);
        assert_eq!(computed.physical_condition_monitor, 11); // 8 + ceil(5/2)
    }

    #[tokio::test]
    async fn test_save_sr5_character_with_priority_selection() {
        use personafix_core::model::priority::{PriorityLevel, PrioritySelection};
        use personafix_core::model::skills::Skill;

        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(&pool, "ch1", "c1", &Edition::SR5, "Adept", &Metatype::Human)
            .await
            .unwrap();

        let base = CharacterBase {
            id: "ch1".to_string(),
            campaign_id: "c1".to_string(),
            name: "Adept".to_string(),
            edition: Edition::SR5,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 4,
                agility: 5,
                reaction: 4,
                strength: 3,
                willpower: 3,
                logic: 2,
                intuition: 5,
                charisma: 3,
                edge: 3,
                essence: 600,
                magic: Some(6),
                resonance: None,
            },
            skills: vec![Skill {
                id: "unarmed".to_string(),
                name: "Unarmed Combat".to_string(),
                linked_attribute: "AGI".to_string(),
                group: None,
                rating: 6,
                specializations: vec![],
            }],
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
            knowledge_skills: vec![],
            priority_selection: Some(PrioritySelection {
                metatype: PriorityLevel::D,
                attributes: PriorityLevel::A,
                magic_or_resonance: PriorityLevel::B,
                skills: PriorityLevel::C,
                resources: PriorityLevel::E,
            }),
            magic_tradition: None,
            tradition_name: None,
            notes: String::new(),
        };

        save_character_base_db(&pool, &base).await.unwrap();

        // Read back — priority selection should persist
        let computed = get_character_db(&pool, "ch1").await.unwrap();
        let priority = computed.base.priority_selection.unwrap();
        assert_eq!(priority.metatype, PriorityLevel::D);
        assert_eq!(priority.attributes, PriorityLevel::A);
        assert_eq!(priority.magic_or_resonance, PriorityLevel::B);
        assert_eq!(priority.skills, PriorityLevel::C);
        assert_eq!(priority.resources, PriorityLevel::E);
        assert_eq!(computed.computed_attributes.magic, Some(6));
    }

    // -- Game data query tests --

    async fn setup_game_data_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let migrations =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../crates/data/migrations");
        let migrator = sqlx::migrate::Migrator::new(migrations).await.unwrap();
        migrator.run(&pool).await.unwrap();

        // Seed sample skills
        for (id, name, attr, group, edition) in [
            ("s1", "Pistols", "AGI", Some("Firearms"), "SR4"),
            ("s2", "Automatics", "AGI", Some("Firearms"), "SR4"),
            ("s3", "Perception", "INT", None, "SR4"),
            ("s4", "Pistols", "AGI", Some("Firearms"), "SR5"),
            ("s5", "Sneaking", "AGI", Some("Stealth"), "SR5"),
        ] {
            sqlx::query(
                "INSERT INTO skills_data (id, name, linked_attribute, skill_group, edition, source, page) VALUES (?, ?, ?, ?, ?, 'SR', '1')"
            )
            .bind(id).bind(name).bind(attr).bind(group).bind(edition)
            .execute(&pool).await.unwrap();
        }

        // Seed sample qualities
        for (id, name, qtype, cost, edition) in [
            ("q1", "Ambidextrous", "Positive", 5, "SR4"),
            ("q2", "Toughness", "Positive", 10, "SR4"),
            ("q3", "SINner", "Negative", 5, "SR4"),
            ("q4", "Analytical Mind", "Positive", 5, "SR5"),
            ("q5", "Bad Luck", "Negative", 20, "SR5"),
        ] {
            sqlx::query(
                "INSERT INTO qualities (id, name, quality_type, cost, edition, source, page) VALUES (?, ?, ?, ?, ?, 'SR', '1')"
            )
            .bind(id).bind(name).bind(qtype).bind(cost).bind(edition)
            .execute(&pool).await.unwrap();
        }

        // Seed sample weapons
        sqlx::query(
            "INSERT INTO weapons (id, name, category, damage, ap, mode, recoil_comp, ammo, availability, cost, edition, source, page) \
             VALUES ('w1', 'Ares Predator V', 'Heavy Pistols', '8P', '-1', 'SA', '0', '15(c)', '5R', '725', 'SR5', 'SR5', '425')"
        ).execute(&pool).await.unwrap();

        // Seed sample augmentation
        sqlx::query(
            "INSERT INTO augmentations (id, name, augmentation_type, essence_cost, capacity, availability, cost, edition, source, page) \
             VALUES ('a1', 'Wired Reflexes 1', 'Cyberware', '2', '[0]', '8R', '39000', 'SR4', 'SR4', '340')"
        ).execute(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn test_query_skills_by_edition() {
        let pool = setup_game_data_db().await;
        let sr4_skills = query_skills_db(&pool, "SR4").await.unwrap();
        assert_eq!(sr4_skills.len(), 3);
        assert!(sr4_skills.iter().any(|s| s.name == "Pistols"));
        assert!(sr4_skills.iter().any(|s| s.name == "Perception"));

        let sr5_skills = query_skills_db(&pool, "SR5").await.unwrap();
        assert_eq!(sr5_skills.len(), 2);
        assert!(sr5_skills.iter().any(|s| s.name == "Sneaking"));
    }

    #[tokio::test]
    async fn test_query_skills_returns_attributes_and_groups() {
        let pool = setup_game_data_db().await;
        let skills = query_skills_db(&pool, "SR4").await.unwrap();
        let pistols = skills.iter().find(|s| s.name == "Pistols").unwrap();
        assert_eq!(pistols.linked_attribute, "AGI");
        assert_eq!(pistols.skill_group.as_deref(), Some("Firearms"));

        let perception = skills.iter().find(|s| s.name == "Perception").unwrap();
        assert!(perception.skill_group.is_none());
    }

    #[tokio::test]
    async fn test_query_qualities_by_edition() {
        let pool = setup_game_data_db().await;
        let sr4_quals = query_qualities_db(&pool, "SR4").await.unwrap();
        assert_eq!(sr4_quals.len(), 3);

        let positives: Vec<_> = sr4_quals
            .iter()
            .filter(|q| q.quality_type == "Positive")
            .collect();
        let negatives: Vec<_> = sr4_quals
            .iter()
            .filter(|q| q.quality_type == "Negative")
            .collect();
        assert_eq!(positives.len(), 2);
        assert_eq!(negatives.len(), 1);
        assert_eq!(negatives[0].name, "SINner");
        assert_eq!(negatives[0].cost, 5);
    }

    #[tokio::test]
    async fn test_query_weapons_by_edition() {
        let pool = setup_game_data_db().await;
        let weapons = query_weapons_db(&pool, "SR5").await.unwrap();
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].name, "Ares Predator V");
        assert_eq!(weapons[0].damage, "8P");
    }

    #[tokio::test]
    async fn test_query_augmentations_by_edition() {
        let pool = setup_game_data_db().await;
        let augs = query_augmentations_db(&pool, "SR4").await.unwrap();
        assert_eq!(augs.len(), 1);
        assert_eq!(augs[0].name, "Wired Reflexes 1");
        assert_eq!(augs[0].augmentation_type, "Cyberware");
    }

    #[tokio::test]
    async fn test_query_empty_edition_returns_empty() {
        let pool = setup_game_data_db().await;
        let skills = query_skills_db(&pool, "SR6").await.unwrap();
        assert!(skills.is_empty());
    }

    // -- L3a data smoke (requires a real game_data.db; blocked on backlog P1-1) --

    #[tokio::test]
    async fn data_smoke_real_game_data_db() {
        let db_path = std::env::var("GAME_DATA_DB").unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../game_data.db")
                .to_string_lossy()
                .to_string()
        });
        let db_url = format!("sqlite:{db_path}");
        let pool = SqlitePool::connect(&db_url).await.unwrap_or_else(|e| {
            panic!(
                "Cannot open game_data.db at '{db_path}': {e}\n\
                 Set GAME_DATA_DB env var or run `make migrate` to generate the file."
            )
        });

        let sr4_skills = query_skills_db(&pool, "SR4").await.unwrap();
        assert!(
            !sr4_skills.is_empty(),
            "Expected non-empty SR4 skills from real game_data.db, got 0 rows"
        );
        for skill in &sr4_skills {
            assert!(!skill.name.is_empty(), "Skill has empty name: {skill:?}");
            assert!(
                !skill.linked_attribute.is_empty(),
                "Skill has empty linked_attribute: {skill:?}"
            );
        }

        let sr4_qualities = query_qualities_db(&pool, "SR4").await.unwrap();
        assert!(
            !sr4_qualities.is_empty(),
            "Expected non-empty SR4 qualities from real game_data.db, got 0 rows"
        );
        assert!(
            sr4_qualities.iter().any(|q| q.quality_type == "Positive"),
            "Expected at least one Positive quality in SR4 data"
        );
        assert!(
            sr4_qualities.iter().any(|q| q.quality_type == "Negative"),
            "Expected at least one Negative quality in SR4 data"
        );

        let sr5_powers = query_adept_powers_db(&pool).await.unwrap();
        assert!(
            !sr5_powers.is_empty(),
            "Expected non-empty SR5 adept powers from real game_data.db, got 0 rows"
        );
        for p in &sr5_powers {
            assert!(!p.name.is_empty(), "Adept power has empty name: {p:?}");
            let cost: f64 = p.cost.parse().unwrap_or(-1.0);
            assert!(cost >= 0.0, "Adept power '{}' has invalid cost: '{}'", p.name, p.cost);
        }

        let sr5_complex_forms = query_complex_forms_db(&pool).await.unwrap();
        assert!(
            !sr5_complex_forms.is_empty(),
            "Expected non-empty SR5 complex forms from real game_data.db, got 0 rows"
        );
        for f in &sr5_complex_forms {
            assert!(!f.name.is_empty(), "Complex form has empty name: {f:?}");
            assert!(!f.fading.is_empty(), "Complex form '{}' has empty fading value", f.name);
        }

        let vehicles = query_vehicles_db(&pool).await.unwrap();
        assert!(
            !vehicles.is_empty(),
            "Expected non-empty vehicles from real game_data.db, got 0 rows.\n\
             Run `make migrate` to re-seed the DB with vehicle data."
        );
        for v in &vehicles {
            assert!(!v.name.is_empty(), "Vehicle has empty name: {v:?}");
        }
    }

    #[tokio::test]
    async fn test_skill_specialization_roundtrip() {
        use personafix_core::model::skills::{Skill, Specialization};

        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(&pool, "ch1", "c1", &Edition::SR4, "Sniper", &Metatype::Human)
            .await
            .unwrap();

        let base = CharacterBase {
            id: "ch1".to_string(),
            campaign_id: "c1".to_string(),
            name: "Sniper".to_string(),
            edition: Edition::SR4,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 4,
                agility: 5,
                reaction: 4,
                strength: 3,
                willpower: 3,
                logic: 2,
                intuition: 4,
                charisma: 2,
                edge: 3,
                essence: 600,
                magic: None,
                resonance: None,
            },
            skills: vec![Skill {
                id: "longarms".to_string(),
                name: "Longarms".to_string(),
                linked_attribute: "AGI".to_string(),
                group: Some("Firearms".to_string()),
                rating: 5,
                specializations: vec![Specialization {
                    name: "Sniper Rifles".to_string(),
                    bonus: 2,
                }],
            }],
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
            knowledge_skills: vec![],
            priority_selection: None,
            magic_tradition: None,
            tradition_name: None,
            notes: String::new(),
        };

        save_character_base_db(&pool, &base).await.unwrap();
        let loaded = get_character_db(&pool, "ch1").await.unwrap();
        let skill = loaded.base.skills.iter().find(|s| s.id == "longarms").unwrap();
        assert_eq!(skill.specializations.len(), 1);
        assert_eq!(skill.specializations[0].name, "Sniper Rifles");
        assert_eq!(skill.specializations[0].bonus, 2);
    }

    #[tokio::test]
    async fn test_tradition_name_roundtrip() {
        use personafix_core::model::magic::MagicTradition;
        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(&pool, "ch1", "c1", &Edition::SR5, "Magician", &Metatype::Human)
            .await
            .unwrap();

        let base = CharacterBase {
            id: "ch1".to_string(),
            campaign_id: "c1".to_string(),
            name: "Magician".to_string(),
            edition: Edition::SR5,
            metatype: Metatype::Human,
            attributes: Attributes {
                body: 3,
                agility: 3,
                reaction: 3,
                strength: 2,
                willpower: 4,
                logic: 4,
                intuition: 4,
                charisma: 3,
                edge: 2,
                essence: 600,
                magic: Some(5),
                resonance: None,
            },
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
            knowledge_skills: vec![],
            priority_selection: None,
            magic_tradition: Some(MagicTradition::Magician),
            tradition_name: Some("Hermetic".to_string()),
            notes: String::new(),
        };

        save_character_base_db(&pool, &base).await.unwrap();
        let loaded = get_character_db(&pool, "ch1").await.unwrap();
        assert_eq!(
            loaded.base.tradition_name.as_deref(),
            Some("Hermetic"),
            "tradition_name should roundtrip through save/load"
        );

        // update via update_tradition_name_db
        update_tradition_name_db(&pool, "ch1", Some("Shaman"))
            .await
            .unwrap();
        let updated = get_character_db(&pool, "ch1").await.unwrap();
        assert_eq!(
            updated.base.tradition_name.as_deref(),
            Some("Shaman"),
            "tradition_name should be updatable"
        );

        // clear tradition name
        update_tradition_name_db(&pool, "ch1", None)
            .await
            .unwrap();
        let cleared = get_character_db(&pool, "ch1").await.unwrap();
        assert!(
            cleared.base.tradition_name.is_none(),
            "tradition_name should be clearable to None"
        );
    }

    #[tokio::test]
    async fn test_open_campaign_roundtrip() {
        let tmp_path = std::env::temp_dir()
            .join(format!("pf_test_{}.srx", uuid::Uuid::new_v4()));
        let db_url = format!("sqlite:{}?mode=rwc", tmp_path.display());

        {
            let pool = SqlitePool::connect(&db_url).await.unwrap();
            create_campaign_db(&pool, "c1", "My Campaign").await.unwrap();
            pool.close().await;
        }

        let reopen_url = format!("sqlite:{}?mode=rw", tmp_path.display());
        let pool2 = SqlitePool::connect(&reopen_url).await.unwrap();
        let campaign = open_campaign_db(&pool2).await.unwrap();
        assert_eq!(campaign.name, "My Campaign");
        assert_eq!(campaign.id, "c1");
        pool2.close().await;
        let _ = std::fs::remove_file(&tmp_path);
    }

    #[tokio::test]
    async fn test_export_import_json_roundtrip() {
        use personafix_core::model::skills::Skill;

        let pool = setup_test_db().await;
        create_campaign_db(&pool, "c1", "Campaign").await.unwrap();
        create_character_db(&pool, "ch1", "c1", &Edition::SR5, "Razor", &Metatype::Elf)
            .await
            .unwrap();

        let mut computed = get_character_db(&pool, "ch1").await.unwrap();
        computed.base.skills.push(Skill {
            id: "pistols".to_string(),
            name: "Pistols".to_string(),
            linked_attribute: "AGI".to_string(),
            group: None,
            rating: 6,
            specializations: vec![],
        });
        save_character_base_db(&pool, &computed.base).await.unwrap();

        apply_event_db(
            &pool,
            "ch1",
            &LedgerEvent::KarmaReceived {
                amount: 10,
                reason: "Run".to_string(),
                run_id: None,
            },
        )
        .await
        .unwrap();

        let tmp_path = std::env::temp_dir()
            .join(format!("pf_test_{}.json", uuid::Uuid::new_v4()));
        export_character_json_db(&pool, "ch1", &tmp_path.to_string_lossy())
            .await
            .unwrap();

        let new_id = import_character_json_db(&pool, "c1", &tmp_path.to_string_lossy())
            .await
            .unwrap();
        assert_ne!(new_id, "ch1");

        let imported = get_character_db(&pool, &new_id).await.unwrap();
        assert_eq!(imported.base.name, "Razor");
        assert_eq!(imported.base.metatype, Metatype::Elf);
        assert_eq!(imported.base.edition, Edition::SR5);
        assert!(
            imported.base.skills.iter().any(|s| s.name == "Pistols" && s.rating == 6),
            "imported character should have Pistols at rating 6"
        );
        assert_eq!(imported.total_karma_earned, 10);

        let _ = std::fs::remove_file(&tmp_path);
    }

    #[tokio::test]
    async fn test_import_chummer_sr5_minimal() {
        use personafix_import_export::sr5;

        const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<character>
  <name>Street Samurai</name>
  <metatype>Human</metatype>
  <adept>false</adept>
  <magician>false</magician>
  <technomancer>false</technomancer>
  <attributes>
    <attribute><name>BOD</name><base>4</base><karma>0</karma></attribute>
    <attribute><name>AGI</name><base>5</base><karma>0</karma></attribute>
  </attributes>
  <newskills>
    <skills>
      <skill>
        <suid></suid>
        <name>Pistols</name>
        <base>5</base>
        <karma>0</karma>
      </skill>
    </skills>
    <knoskills></knoskills>
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

        let chum = sr5::parse_str(XML).expect("minimal XML should parse");
        let game_pool = setup_game_data_db().await;
        let base = sr5::map::import_sr5(&chum, &game_pool, "c1", "ch1")
            .await
            .expect("import_sr5 should succeed with seeded game data");

        assert_eq!(base.name, "Street Samurai");
        assert_eq!(base.metatype, Metatype::Human);
        assert!(
            base.skills.iter().any(|s| s.rating > 0),
            "should have at least one skill with rating > 0"
        );
    }
}
