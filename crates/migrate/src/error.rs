use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrateError {
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("{0}")]
    Other(String),
}

pub type MigrateResult<T> = Result<T, MigrateError>;
