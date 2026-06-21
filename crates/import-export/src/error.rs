use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Invalid character data: {0}")]
    Invalid(String),
}

pub type ImportResult<T> = Result<T, ImportError>;
