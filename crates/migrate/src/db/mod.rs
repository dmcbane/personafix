use std::path::Path;

use sqlx::SqlitePool;

use crate::error::MigrateResult;
use crate::xml::ParsedGameData;

/// Seed a SQLite database with parsed game data.
/// Runs migrations first, then inserts all data in a transaction.
pub async fn seed(db_path: &Path, datasets: &[ParsedGameData]) -> MigrateResult<()> {
    // Create parent directory if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&db_url).await?;

    // Run migrations from crates/data/migrations/
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations_dir).await?;
    migrator.run(&pool).await?;

    for data in datasets {
        seed_edition(&pool, data).await?;
    }

    pool.close().await;
    Ok(())
}

async fn seed_edition(pool: &SqlitePool, data: &ParsedGameData) -> MigrateResult<()> {
    let edition = &data.edition;

    // Use a transaction for bulk inserts
    let mut tx = pool.begin().await?;

    // Sourcebooks
    for book in &data.books {
        sqlx::query(
            "INSERT OR REPLACE INTO sourcebooks (id, name, abbreviation, edition) VALUES (?, ?, ?, ?)",
        )
        .bind(&book.id)
        .bind(&book.name)
        .bind(&book.abbreviation)
        .bind(edition)
        .execute(&mut *tx)
        .await?;
    }

    // Metatypes
    for m in &data.metatypes {
        sqlx::query(
            "INSERT OR REPLACE INTO metatypes (id, name, edition, body_min, body_max, agility_min, agility_max, reaction_min, reaction_max, strength_min, strength_max, willpower_min, willpower_max, logic_min, logic_max, intuition_min, intuition_max, charisma_min, charisma_max, edge_min, edge_max, source, page) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&m.id)
        .bind(&m.name)
        .bind(edition)
        .bind(m.body_min)
        .bind(m.body_max)
        .bind(m.agility_min)
        .bind(m.agility_max)
        .bind(m.reaction_min)
        .bind(m.reaction_max)
        .bind(m.strength_min)
        .bind(m.strength_max)
        .bind(m.willpower_min)
        .bind(m.willpower_max)
        .bind(m.logic_min)
        .bind(m.logic_max)
        .bind(m.intuition_min)
        .bind(m.intuition_max)
        .bind(m.charisma_min)
        .bind(m.charisma_max)
        .bind(m.edge_min)
        .bind(m.edge_max)
        .bind(&m.source)
        .bind(&m.page)
        .execute(&mut *tx)
        .await?;
    }

    // Skills
    for s in &data.skills {
        sqlx::query(
            "INSERT OR REPLACE INTO skills_data (id, name, linked_attribute, skill_group, edition, source, page) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&s.id)
        .bind(&s.name)
        .bind(&s.linked_attribute)
        .bind(&s.skill_group)
        .bind(edition)
        .bind(&s.source)
        .bind(&s.page)
        .execute(&mut *tx)
        .await?;
    }

    // Qualities
    for q in &data.qualities {
        sqlx::query(
            "INSERT OR REPLACE INTO qualities (id, name, quality_type, cost, edition, source, page) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&q.id)
        .bind(&q.name)
        .bind(&q.quality_type)
        .bind(q.cost)
        .bind(edition)
        .bind(&q.source)
        .bind(&q.page)
        .execute(&mut *tx)
        .await?;
    }

    // Weapons
    for w in &data.weapons {
        sqlx::query(
            "INSERT OR REPLACE INTO weapons (id, name, category, damage, ap, mode, recoil_comp, ammo, availability, cost, edition, source, page) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&w.id)
        .bind(&w.name)
        .bind(&w.category)
        .bind(&w.damage)
        .bind(&w.ap)
        .bind(&w.mode)
        .bind(&w.recoil_comp)
        .bind(&w.ammo)
        .bind(&w.availability)
        .bind(&w.cost)
        .bind(edition)
        .bind(&w.source)
        .bind(&w.page)
        .execute(&mut *tx)
        .await?;
    }

    // Armor
    for a in &data.armor {
        sqlx::query(
            "INSERT OR REPLACE INTO armor (id, name, armor_value, availability, cost, edition, source, page) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&a.id)
        .bind(&a.name)
        .bind(&a.armor_value)
        .bind(&a.availability)
        .bind(&a.cost)
        .bind(edition)
        .bind(&a.source)
        .bind(&a.page)
        .execute(&mut *tx)
        .await?;
    }

    // Augmentations
    for aug in &data.augmentations {
        sqlx::query(
            "INSERT OR REPLACE INTO augmentations (id, name, augmentation_type, essence_cost, capacity, availability, cost, edition, source, page) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&aug.id)
        .bind(&aug.name)
        .bind(&aug.augmentation_type)
        .bind(&aug.essence_cost)
        .bind(&aug.capacity)
        .bind(&aug.availability)
        .bind(&aug.cost)
        .bind(edition)
        .bind(&aug.source)
        .bind(&aug.page)
        .execute(&mut *tx)
        .await?;
    }

    // Spells
    for s in &data.spells {
        sqlx::query(
            "INSERT OR REPLACE INTO spells (id, name, category, spell_type, range, damage, duration, drain, edition, source, page) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&s.id)
        .bind(&s.name)
        .bind(&s.category)
        .bind(&s.spell_type)
        .bind(&s.range)
        .bind(&s.damage)
        .bind(&s.duration)
        .bind(&s.drain)
        .bind(edition)
        .bind(&s.source)
        .bind(&s.page)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
