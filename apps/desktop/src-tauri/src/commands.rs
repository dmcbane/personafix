use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
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
    #[allow(clippy::type_complexity)]
    let row: (
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
        String,
        String,
        String,
        Option<String>,
    ) = sqlx::query_as(
        "SELECT attributes_json, skills_json, skill_groups_json, qualities_json, \
             augmentations_json, spells_json, adept_powers_json, complex_forms_json, \
             contacts_json, weapons_json, armor_json, gear_json, vehicles_json, \
             priority_selection_json \
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
        attributes: serde_json::from_str(&row.0)?,
        skills: serde_json::from_str(&row.1)?,
        skill_groups: serde_json::from_str(&row.2)?,
        qualities: serde_json::from_str(&row.3)?,
        augmentations: serde_json::from_str(&row.4)?,
        spells: serde_json::from_str(&row.5)?,
        adept_powers: serde_json::from_str(&row.6)?,
        complex_forms: serde_json::from_str(&row.7)?,
        contacts: serde_json::from_str(&row.8)?,
        weapons: serde_json::from_str(&row.9)?,
        armor: serde_json::from_str(&row.10)?,
        gear: serde_json::from_str(&row.11)?,
        vehicles: serde_json::from_str(&row.12)?,
        priority_selection: row.13.as_deref().and_then(|s| serde_json::from_str(s).ok()),
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

    sqlx::query(
        "UPDATE character_base SET \
         metatype = ?, attributes_json = ?, skills_json = ?, skill_groups_json = ?, \
         qualities_json = ?, augmentations_json = ?, spells_json = ?, adept_powers_json = ?, \
         complex_forms_json = ?, contacts_json = ?, weapons_json = ?, armor_json = ?, \
         gear_json = ?, vehicles_json = ?, priority_selection_json = ? \
         WHERE character_id = ?",
    )
    .bind(&metatype_str)
    .bind(&attributes_json)
    .bind(&skills_json)
    .bind(&skill_groups_json)
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
    .bind(&base.id)
    .execute(pool)
    .await?;

    Ok(())
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
            priority_selection: None,
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
            priority_selection: None,
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
            priority_selection: None,
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
            priority_selection: None,
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
            priority_selection: Some(PrioritySelection {
                metatype: PriorityLevel::D,
                attributes: PriorityLevel::A,
                magic_or_resonance: PriorityLevel::B,
                skills: PriorityLevel::C,
                resources: PriorityLevel::E,
            }),
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
}
