use serde::Serialize;

/// Application error type for IPC command results.
/// Must implement Serialize for Tauri to return it to the frontend.
#[derive(Debug, Serialize)]
pub struct AppError {
    pub kind: String,
    pub message: String,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError {
            kind: "database".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        AppError {
            kind: "migration".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError {
            kind: "serialization".to_string(),
            message: err.to_string(),
        }
    }
}

impl AppError {
    pub fn not_found(entity: &str, id: &str) -> Self {
        AppError {
            kind: "not_found".to_string(),
            message: format!("{entity} with id '{id}' not found"),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        AppError {
            kind: "validation".to_string(),
            message: message.into(),
        }
    }
}
