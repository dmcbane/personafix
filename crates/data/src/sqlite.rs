use sqlx::SqlitePool;

use personafix_core::model::{
    augmentations::Augmentation, edition::Edition, gear::*, magic::*, qualities::Quality,
    skills::Skill,
};

use crate::{DataResult, GameDataRepository};

/// SQLite implementation of the game data repository.
/// Used by the desktop app (one SQLite file per campaign).
pub struct SqliteGameDataRepository {
    pool: SqlitePool,
}

impl SqliteGameDataRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

impl GameDataRepository for SqliteGameDataRepository {
    async fn get_metatypes(&self, _edition: Edition) -> DataResult<Vec<String>> {
        todo!("query metatypes table")
    }

    async fn get_skills(&self, _edition: Edition) -> DataResult<Vec<Skill>> {
        todo!("query skills_data table")
    }

    async fn get_qualities(&self, _edition: Edition) -> DataResult<Vec<Quality>> {
        todo!("query qualities table")
    }

    async fn get_weapons(&self, _edition: Edition) -> DataResult<Vec<Weapon>> {
        todo!("query weapons table")
    }

    async fn get_armor(&self, _edition: Edition) -> DataResult<Vec<Armor>> {
        todo!("query armor table")
    }

    async fn get_gear(&self, _edition: Edition) -> DataResult<Vec<GearItem>> {
        todo!("query gear table")
    }

    async fn get_augmentations(&self, _edition: Edition) -> DataResult<Vec<Augmentation>> {
        todo!("query augmentations table")
    }

    async fn get_spells(&self, _edition: Edition) -> DataResult<Vec<Spell>> {
        todo!("query spells table")
    }

    async fn get_adept_powers(&self, _edition: Edition) -> DataResult<Vec<AdeptPower>> {
        todo!("query adept_powers table")
    }

    async fn get_complex_forms(&self, _edition: Edition) -> DataResult<Vec<ComplexForm>> {
        todo!("query complex_forms table")
    }

    async fn get_vehicles(&self, _edition: Edition) -> DataResult<Vec<Vehicle>> {
        todo!("query vehicles table")
    }
}
