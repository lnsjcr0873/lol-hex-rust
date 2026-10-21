use std::path::Path;
use crate::storage::error::StorageError;

/// Applies mandatory performance and consistency pragmas to an active SQLite connection.
pub fn apply_pragmas(conn: &rusqlite::Connection) -> Result<(), StorageError> {
    // Basic pragmas
    conn.execute_batch(
        r#"
        PRAGMA busy_timeout = 5000;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA cache_size = -8000;
        "#
    )?;

    // Attempt WAL mode (supported on disk databases, no-op or memory in :memory:)
    let _ = conn.pragma_update(None, "journal_mode", "WAL");

    // Perform quick integrity check
    let check: String = conn.query_row("PRAGMA quick_check;", [], |r| r.get(0))?;
    if check != "ok" {
        return Err(StorageError::Sqlite(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
            Some(format!("SQLite quick_check failed: {}", check)),
        )));
    }

    Ok(())
}

/// Detects whether a StorageError is caused by database corruption or invalid header.
pub fn is_corrupt_error(err: &StorageError) -> bool {
    match err {
        StorageError::Sqlite(rusqlite::Error::SqliteFailure(code, _)) => {
            code.code == rusqlite::ffi::ErrorCode::DatabaseCorrupt
                || code.code == rusqlite::ffi::ErrorCode::NotADatabase
                || code.extended_code == rusqlite::ffi::SQLITE_CORRUPT
                || code.extended_code == rusqlite::ffi::SQLITE_NOTADB
        }
        StorageError::Sqlite(e) => {
            let msg = e.to_string().to_lowercase();
            msg.contains("corrupt")
                || msg.contains("malformed")
                || msg.contains("notadb")
                || msg.contains("file is not a database")
                || msg.contains("disk i/o error")
        }
        _ => false,
    }
}

/// Moves a corrupted database file to a timestamped backup copy.
pub fn recover_corrupt_file(path: &Path) -> Result<(), StorageError> {
    if path.exists() && path.to_str() != Some(":memory:") {
        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("HexBrain");
        let corrupt_path = path.with_file_name(format!("{}.db.corrupted.{}", file_stem, now_sec));
        
        tracing::warn!(
            "SQLite database at {:?} is corrupted. Backing up to {:?} and recreating fresh DB",
            path,
            corrupt_path
        );
        let _ = std::fs::rename(path, corrupt_path);
        
        // Also cleanup dangling WAL/SHM companion files if present
        let parent = path.parent().unwrap_or(Path::new(""));
        let wal = parent.join(format!("{}.db-wal", file_stem));
        let shm = parent.join(format!("{}.db-shm", file_stem));
        let _ = std::fs::remove_file(wal);
        let _ = std::fs::remove_file(shm);
    }
    Ok(())
}

/// Opens a SQLite connection with automated corruption detection and recovery.
pub fn open_connection_with_recovery(path: &Path) -> Result<rusqlite::Connection, StorageError> {
    if path.to_str() == Some(":memory:") {
        let conn = rusqlite::Connection::open_in_memory()?;
        apply_pragmas(&conn)?;
        return Ok(conn);
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    let open_result = rusqlite::Connection::open(path);
    match open_result {
        Ok(conn) => {
            if let Err(e) = apply_pragmas(&conn) {
                if is_corrupt_error(&e) {
                    drop(conn);
                    recover_corrupt_file(path)?;
                    let fresh_conn = rusqlite::Connection::open(path)?;
                    apply_pragmas(&fresh_conn)?;
                    return Ok(fresh_conn);
                }
                return Err(e);
            }
            Ok(conn)
        }
        Err(e) => {
            let storage_err = StorageError::from(e);
            if is_corrupt_error(&storage_err) {
                recover_corrupt_file(path)?;
                let fresh_conn = rusqlite::Connection::open(path)?;
                apply_pragmas(&fresh_conn)?;
                Ok(fresh_conn)
            } else {
                Err(storage_err)
            }
        }
    }
}
