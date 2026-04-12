use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::RwLock;

/// Application state managed by Tauri.
/// Holds database connections for the currently open campaign.
pub struct AppState {
    /// The currently open campaign database (one SQLite file per campaign).
    /// None when no campaign is open.
    pub campaign_pool: Arc<RwLock<Option<SqlitePool>>>,
    /// Path to the currently open campaign file.
    pub campaign_path: Arc<RwLock<Option<PathBuf>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            campaign_pool: Arc::new(RwLock::new(None)),
            campaign_path: Arc::new(RwLock::new(None)),
        }
    }
}
