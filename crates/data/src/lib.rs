pub mod sqlite;

use personafix_core::model::{
    augmentations::Augmentation, edition::Edition, gear::*, magic::*, qualities::Quality,
    skills::Skill,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },
}

pub type DataResult<T> = Result<T, DataError>;

/// Trait for querying game data. Implemented for SQLite (desktop) and Postgres (web).
#[allow(async_fn_in_trait)]
pub trait GameDataRepository {
    async fn get_metatypes(&self, edition: Edition) -> DataResult<Vec<String>>;
    async fn get_skills(&self, edition: Edition) -> DataResult<Vec<Skill>>;
    async fn get_qualities(&self, edition: Edition) -> DataResult<Vec<Quality>>;
    async fn get_weapons(&self, edition: Edition) -> DataResult<Vec<Weapon>>;
    async fn get_armor(&self, edition: Edition) -> DataResult<Vec<Armor>>;
    async fn get_gear(&self, edition: Edition) -> DataResult<Vec<GearItem>>;
    async fn get_augmentations(&self, edition: Edition) -> DataResult<Vec<Augmentation>>;
    async fn get_spells(&self, edition: Edition) -> DataResult<Vec<Spell>>;
    async fn get_adept_powers(&self, edition: Edition) -> DataResult<Vec<AdeptPower>>;
    async fn get_complex_forms(&self, edition: Edition) -> DataResult<Vec<ComplexForm>>;
    async fn get_vehicles(&self, edition: Edition) -> DataResult<Vec<Vehicle>>;
}
