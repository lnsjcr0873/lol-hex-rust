use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    InvalidInput(String),
    LockError(String),
    MigrationError { version: i32, message: String },
    NotFound(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Sqlite(e) => write!(f, "SQLite database error: {}", e),
            StorageError::Io(e) => write!(f, "Storage I/O error: {}", e),
            StorageError::InvalidInput(msg) => write!(f, "Invalid storage input: {}", msg),
            StorageError::LockError(msg) => write!(f, "Storage mutex lock error: {}", msg),
            StorageError::MigrationError { version, message } => {
                write!(f, "Migration v{} failed: {}", version, message)
            }
            StorageError::NotFound(msg) => write!(f, "Storage record not found: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Sqlite(e) => Some(e),
            StorageError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        StorageError::Sqlite(err)
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Io(err)
    }
}

impl serde::Serialize for StorageError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
