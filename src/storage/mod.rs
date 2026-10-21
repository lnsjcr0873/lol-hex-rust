pub mod error;
pub mod migrations;
pub mod models;
pub mod schema;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use rusqlite::params;

use self::error::StorageError;
use self::migrations::run_migrations;
use self::models::*;
use self::schema::open_connection_with_recovery;

/// Resolves the absolute filesystem path for the SQLite database.
pub fn resolve_database_path() -> PathBuf {
    // 1. Explicit environment variable override (for unit tests / CI isolation)
    if let Ok(override_path) = std::env::var("HEXBRAIN_DB_PATH") {
        let trimmed = override_path.trim();
        if !trimmed.is_empty() {
            let p = PathBuf::from(trimmed);
            if trimmed != ":memory:" {
                if let Some(parent) = p.parent() {
                    if !parent.as_os_str().is_empty() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                }
            }
            return p;
        }
    }

    // 2. Standard Windows %APPDATA%\HexBrain\HexBrain.db
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let dir = PathBuf::from(appdata).join("HexBrain");
            let _ = std::fs::create_dir_all(&dir);
            return dir.join("HexBrain.db");
        }
    }

    // 3. Unix / macOS fallback ~/.hexbrain/HexBrain.db
    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(home).join(".hexbrain");
            let _ = std::fs::create_dir_all(&dir);
            return dir.join("HexBrain.db");
        }
    }

    // 4. Local fallback: ./data/HexBrain.db
    let local_dir = PathBuf::from("data");
    let _ = std::fs::create_dir_all(&local_dir);
    local_dir.join("HexBrain.db")
}

#[derive(Clone)]
pub struct StorageManager {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl StorageManager {
    /// Opens the default database file at %APPDATA%\HexBrain\HexBrain.db or HEXBRAIN_DB_PATH override.
    pub fn new() -> Result<Self, StorageError> {
        let path = resolve_database_path();
        Self::open(&path)
    }

    /// Opens a database at a specific path (supports :memory:).
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let mut conn = open_connection_with_recovery(path.as_ref())?;
        run_migrations(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Opens an isolated in-memory database for testing.
    pub fn open_in_memory() -> Result<Self, StorageError> {
        Self::open(":memory:")
    }

    /// Returns the underlying raw connection mutex.
    pub fn raw_connection(&self) -> Arc<Mutex<rusqlite::Connection>> {
        self.conn.clone()
    }

    // =========================================================================
    // Encountered Games API (F16)
    // =========================================================================

    /// Saves a single encountered peer record. Returns the generated record ID.
    pub fn save_encountered_game(&self, dto: &EncounteredGameSaveDto) -> Result<i64, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let now = now_ms();
        let queue_type = dto.queue_type.as_deref().unwrap_or("");

        conn.execute(
            r#"
            INSERT INTO encountered_games (game_id, puuid, self_puuid, region, rso_platform_id, update_at, queue_type)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
            "#,
            params![
                dto.game_id,
                dto.puuid,
                dto.self_puuid,
                dto.region,
                dto.rso_platform_id,
                now,
                queue_type
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Saves a batch of encountered peer records inside a single transaction.
    pub fn save_encountered_games_batch(&self, dtos: &[EncounteredGameSaveDto]) -> Result<Vec<i64>, StorageError> {
        let mut conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let tx = conn.transaction()?;
        let now = now_ms();
        let mut ids = Vec::with_capacity(dtos.len());

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO encountered_games (game_id, puuid, self_puuid, region, rso_platform_id, update_at, queue_type)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
                "#
            )?;

            for dto in dtos {
                let queue_type = dto.queue_type.as_deref().unwrap_or("");
                stmt.execute(params![
                    dto.game_id,
                    dto.puuid,
                    dto.self_puuid,
                    dto.region,
                    dto.rso_platform_id,
                    now,
                    queue_type
                ])?;
                ids.push(tx.last_insert_rowid());
            }
        }

        tx.commit()?;
        Ok(ids)
    }

    /// Records an encountered game with all its peer participants in a single atomic transaction.
    /// Inserts 9 peers into `encountered_games` (with `INSERT OR IGNORE`) and touches `saved_player_tags.last_met_at`.
    pub fn record_encountered_game(
        &self,
        game_id: i64,
        self_puuid: &str,
        peer_puuids: &[String],
        region: &str,
        rso_platform_id: &str,
        queue_type: &str,
    ) -> Result<(), StorageError> {
        if peer_puuids.is_empty() || self_puuid.is_empty() || game_id <= 0 {
            return Ok(());
        }

        let now = now_ms();
        let mut conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let tx = conn.transaction()?;

        {
            let mut enc_stmt = tx.prepare_cached(
                r#"
                INSERT OR IGNORE INTO encountered_games 
                (game_id, puuid, self_puuid, region, rso_platform_id, update_at, queue_type)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);
                "#
            )?;

            let mut tag_stmt = tx.prepare_cached(
                r#"
                INSERT INTO saved_player_tags 
                (puuid, self_puuid, region, rso_platform_id, tag, update_at, last_met_at)
                VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?5)
                ON CONFLICT(puuid, self_puuid, region, rso_platform_id) DO UPDATE SET 
                    last_met_at = excluded.last_met_at,
                    update_at = excluded.update_at;
                "#
            )?;

            for peer in peer_puuids {
                if peer.is_empty() || peer == self_puuid {
                    continue;
                }

                enc_stmt.execute(params![
                    game_id,
                    peer,
                    self_puuid,
                    region,
                    rso_platform_id,
                    now,
                    queue_type
                ])?;

                tag_stmt.execute(params![
                    peer,
                    self_puuid,
                    region,
                    rso_platform_id,
                    now
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Queries paginated encountered games between self and target player.
    pub fn query_encountered_games(&self, query: &EncounteredGamesQueryDto) -> Result<PaginatedEncounteredGames, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(40).max(1);
        let offset = (page - 1) * page_size;
        let is_asc = query.time_order.as_deref().unwrap_or("desc").eq_ignore_ascii_case("asc");
        let order_sql = if is_asc { "ASC" } else { "DESC" };

        let (data, total) = match &query.queue_type {
            Some(qt) if !qt.is_empty() => {
                let count_sql = "SELECT COUNT(*) FROM encountered_games WHERE self_puuid = ?1 AND puuid = ?2 AND queue_type = ?3;";
                let total: u64 = conn.query_row(count_sql, params![query.self_puuid, query.puuid, qt], |r| r.get(0))?;

                let select_sql = format!(
                    r#"
                    SELECT id, game_id, puuid, self_puuid, region, rso_platform_id, update_at, queue_type
                    FROM encountered_games
                    WHERE self_puuid = ?1 AND puuid = ?2 AND queue_type = ?3
                    ORDER BY update_at {}
                    LIMIT ?4 OFFSET ?5;
                    "#,
                    order_sql
                );

                let mut stmt = conn.prepare(&select_sql)?;
                let rows = stmt.query_map(params![query.self_puuid, query.puuid, qt, page_size, offset], |row| {
                    Ok(EncounteredGameRecord {
                        id: row.get(0)?,
                        game_id: row.get(1)?,
                        puuid: row.get(2)?,
                        self_puuid: row.get(3)?,
                        region: row.get(4)?,
                        rso_platform_id: row.get(5)?,
                        update_at: row.get(6)?,
                        queue_type: row.get(7)?,
                    })
                })?;

                let mut list = Vec::new();
                for r in rows {
                    list.push(r?);
                }
                (list, total)
            }
            _ => {
                let count_sql = "SELECT COUNT(*) FROM encountered_games WHERE self_puuid = ?1 AND puuid = ?2;";
                let total: u64 = conn.query_row(count_sql, params![query.self_puuid, query.puuid], |r| r.get(0))?;

                let select_sql = format!(
                    r#"
                    SELECT id, game_id, puuid, self_puuid, region, rso_platform_id, update_at, queue_type
                    FROM encountered_games
                    WHERE self_puuid = ?1 AND puuid = ?2
                    ORDER BY update_at {}
                    LIMIT ?3 OFFSET ?4;
                    "#,
                    order_sql
                );

                let mut stmt = conn.prepare(&select_sql)?;
                let rows = stmt.query_map(params![query.self_puuid, query.puuid, page_size, offset], |row| {
                    Ok(EncounteredGameRecord {
                        id: row.get(0)?,
                        game_id: row.get(1)?,
                        puuid: row.get(2)?,
                        self_puuid: row.get(3)?,
                        region: row.get(4)?,
                        rso_platform_id: row.get(5)?,
                        update_at: row.get(6)?,
                        queue_type: row.get(7)?,
                    })
                })?;

                let mut list = Vec::new();
                for r in rows {
                    list.push(r?);
                }
                (list, total)
            }
        };

        let total_pages = if total == 0 {
            0
        } else {
            ((total + page_size as u64 - 1) / page_size as u64) as u32
        };

        Ok(PaginatedEncounteredGames {
            data,
            page,
            page_size,
            total,
            total_pages,
        })
    }

    /// Deletes an encounter record by its primary key ID.
    pub fn delete_encountered_game(&self, id: i64) -> Result<bool, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let count = conn.execute("DELETE FROM encountered_games WHERE id = ?1;", params![id])?;
        Ok(count > 0)
    }

    /// Fast count of total times target player was encountered.
    pub fn count_encountered_games(&self, self_puuid: &str, puuid: &str) -> Result<u64, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let count: u64 = conn.query_row(
            "SELECT COUNT(*) FROM encountered_games WHERE self_puuid = ?1 AND puuid = ?2;",
            params![self_puuid, puuid],
            |r| r.get(0)
        )?;
        Ok(count)
    }

    // =========================================================================
    // Saved Player Tags API (F17)
    // =========================================================================

    /// Retrieves a single saved player record.
    pub fn get_saved_player(&self, puuid: &str, self_puuid: &str) -> Result<Option<SavedPlayerTagRecord>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            r#"
            SELECT puuid, self_puuid, region, rso_platform_id, tag, update_at, last_met_at
            FROM saved_player_tags
            WHERE puuid = ?1 AND self_puuid = ?2
            LIMIT 1;
            "#
        )?;

        let mut rows = stmt.query_map(params![puuid, self_puuid], |row| {
            Ok(SavedPlayerTagRecord {
                puuid: row.get(0)?,
                self_puuid: row.get(1)?,
                region: row.get(2)?,
                rso_platform_id: row.get(3)?,
                tag: row.get(4)?,
                update_at: row.get(5)?,
                last_met_at: row.get(6)?,
            })
        })?;

        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    /// Returns all tags associated with target PUUID across markers (evaluating marked_by_self).
    pub fn get_player_tags(&self, puuid: &str, self_puuid: &str) -> Result<Vec<SavedPlayerTagItem>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare(
            r#"
            SELECT puuid, self_puuid, region, rso_platform_id, tag, update_at, last_met_at
            FROM saved_player_tags
            WHERE puuid = ?1 AND tag IS NOT NULL AND trim(tag) != '';
            "#
        )?;

        let rows = stmt.query_map(params![puuid], |row| {
            let p_self_puuid: String = row.get(1)?;
            let tag: String = row.get(4)?;
            Ok(SavedPlayerTagItem {
                puuid: row.get(0)?,
                self_puuid: p_self_puuid.clone(),
                region: row.get(2)?,
                rso_platform_id: row.get(3)?,
                marked_by_self: p_self_puuid == self_puuid,
                tag,
                update_at: row.get(5)?,
                last_met_at: row.get(6)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Upserts custom player tag notes.
    pub fn update_player_tag(&self, dto: &UpdatePlayerTagDto) -> Result<SavedPlayerTagRecord, StorageError> {
        let now = now_ms();
        let region = dto.region.as_deref().unwrap_or("");
        let rso_platform_id = dto.rso_platform_id.as_deref().unwrap_or("");

        {
            let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
            conn.execute(
                r#"
                INSERT INTO saved_player_tags (puuid, self_puuid, region, rso_platform_id, tag, update_at, last_met_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL)
                ON CONFLICT(puuid, self_puuid, region, rso_platform_id) DO UPDATE SET
                    tag = excluded.tag,
                    update_at = excluded.update_at;
                "#,
                params![
                    dto.puuid,
                    dto.self_puuid,
                    region,
                    rso_platform_id,
                    dto.tag,
                    now
                ],
            )?;
        }

        self.get_saved_player(&dto.puuid, &dto.self_puuid)?
            .ok_or_else(|| StorageError::NotFound("Failed to retrieve upserted tag".to_string()))
    }

    /// Records that this player was encountered, updating last_met_at and update_at.
    pub fn record_player_met(&self, puuid: &str, self_puuid: &str, region: &str, rso_platform_id: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let now = now_ms();

        conn.execute(
            r#"
            INSERT INTO saved_player_tags (puuid, self_puuid, region, rso_platform_id, tag, update_at, last_met_at)
            VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?5)
            ON CONFLICT(puuid, self_puuid, region, rso_platform_id) DO UPDATE SET
                last_met_at = excluded.last_met_at,
                update_at = excluded.update_at;
            "#,
            params![puuid, self_puuid, region, rso_platform_id, now],
        )?;

        Ok(())
    }

    /// Deletes a saved player tag.
    pub fn delete_saved_player(&self, puuid: &str, self_puuid: &str) -> Result<bool, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let count = conn.execute(
            "DELETE FROM saved_player_tags WHERE puuid = ?1 AND self_puuid = ?2;",
            params![puuid, self_puuid],
        )?;
        Ok(count > 0)
    }

    /// Queries all players with active tags.
    pub fn query_all_player_tags(&self, query: &QueryAllPlayerTagsDto) -> Result<PaginatedPlayerTags, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(40).max(1);
        let offset = (page - 1) * page_size;
        let is_asc = query.time_order.as_deref().unwrap_or("desc").eq_ignore_ascii_case("asc");
        let order_sql = if is_asc { "ASC" } else { "DESC" };

        let total: u64 = conn.query_row(
            "SELECT COUNT(*) FROM saved_player_tags WHERE tag IS NOT NULL AND trim(tag) != '';",
            [],
            |r| r.get(0)
        )?;

        let select_sql = format!(
            r#"
            SELECT puuid, self_puuid, region, rso_platform_id, tag, update_at, last_met_at
            FROM saved_player_tags
            WHERE tag IS NOT NULL AND trim(tag) != ''
            ORDER BY update_at {}
            LIMIT ?1 OFFSET ?2;
            "#,
            order_sql
        );

        let mut stmt = conn.prepare(&select_sql)?;
        let rows = stmt.query_map(params![page_size, offset], |row| {
            Ok(SavedPlayerTagRecord {
                puuid: row.get(0)?,
                self_puuid: row.get(1)?,
                region: row.get(2)?,
                rso_platform_id: row.get(3)?,
                tag: row.get(4)?,
                update_at: row.get(5)?,
                last_met_at: row.get(6)?,
            })
        })?;

        let mut data = Vec::new();
        for r in rows {
            data.push(r?);
        }

        Ok(PaginatedPlayerTags {
            data,
            page,
            page_size,
            total,
        })
    }

    /// Combined query returning player tag record, tags list, and paginated encounters.
    pub fn query_saved_player_with_games(&self, puuid: &str, self_puuid: &str, queue_type: Option<&str>) -> Result<SavedPlayerWithGamesResult, StorageError> {
        let player = self.get_saved_player(puuid, self_puuid)?;
        let tags = self.get_player_tags(puuid, self_puuid)?;
        let encountered_games = self.query_encountered_games(&EncounteredGamesQueryDto {
            puuid: puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: queue_type.map(|s| s.to_string()),
            page: Some(1),
            page_size: Some(10),
            time_order: Some("desc".to_string()),
        })?;

        Ok(SavedPlayerWithGamesResult {
            player,
            tags,
            encountered_games,
        })
    }

    /// Champ-Select Hook: Check which players in lobby have active custom tags.
    pub fn check_tagged_players(&self, self_puuid: &str, puuids: &[String]) -> Result<Vec<SavedPlayerTagItem>, StorageError> {
        if puuids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        for puuid in puuids {
            let tags = self.get_player_tags(puuid, self_puuid)?;
            results.extend(tags);
        }
        Ok(results)
    }

    // =========================================================================
    // Preset Tag Phrases & App Settings API
    // =========================================================================

    /// Retrieves configured preset phrases ordered by sort_order.
    pub fn get_player_tag_phrases(&self) -> Result<Vec<String>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT phrase FROM player_tag_phrases ORDER BY sort_order ASC, id ASC;")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut phrases = Vec::new();
        for r in rows {
            phrases.push(r?);
        }
        Ok(phrases)
    }

    /// Replaces configured preset phrases with sanitized unique phrases (max 20, max 100 chars).
    pub fn update_player_tag_phrases(&self, phrases: &[String]) -> Result<Vec<String>, StorageError> {
        let normalized = normalize_saved_player_tag_phrases(phrases);
        let mut conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM player_tag_phrases;", [])?;
        {
            let mut stmt = tx.prepare("INSERT INTO player_tag_phrases (phrase, sort_order) VALUES (?1, ?2);")?;
            for (i, p) in normalized.iter().enumerate() {
                stmt.execute(params![p, i as i64])?;
            }
        }
        tx.commit()?;
        Ok(normalized)
    }

    /// Reads a persistent key-value setting from SQLite.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1 LIMIT 1;")?;
        let mut rows = stmt.query_map(params![key], |r| r.get(0))?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    /// Stores a persistent key-value setting in SQLite.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let conn = self.conn.lock().map_err(|e| StorageError::LockError(e.to_string()))?;
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value;",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn is_phrase_panel_expanded(&self) -> Result<bool, StorageError> {
        let val = self.get_setting("player_tag_phrase_panel_expanded")?;
        Ok(val.map_or(true, |v| v != "false" && v != "0"))
    }

    pub fn set_phrase_panel_expanded(&self, expanded: bool) -> Result<bool, StorageError> {
        self.set_setting("player_tag_phrase_panel_expanded", if expanded { "true" } else { "false" })?;
        Ok(expanded)
    }

    pub fn is_champ_select_alert_enabled(&self) -> Result<bool, StorageError> {
        let val = self.get_setting("champ_select_tagged_player_alert")?;
        Ok(val.map_or(true, |v| v != "false" && v != "0"))
    }

    pub fn set_champ_select_alert_enabled(&self, enabled: bool) -> Result<bool, StorageError> {
        self.set_setting("champ_select_tagged_player_alert", if enabled { "true" } else { "false" })?;
        Ok(enabled)
    }
}

// =========================================================================
// Automated Unit Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_storage() -> StorageManager {
        StorageManager::open_in_memory().expect("Failed to open test in-memory SQLite storage")
    }

    #[test]
    fn test_in_memory_initialization_and_migration() {
        let storage = create_test_storage();
        let conn = storage.conn.lock().unwrap();
        let version: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0)).unwrap();
        assert_eq!(version, 1);

        // Verify tables exist
        let enc_exists: bool = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='encountered_games';",
            [],
            |r| r.get::<_, i32>(0).map(|c| c > 0)
        ).unwrap();
        assert!(enc_exists);

        let tag_exists: bool = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='saved_player_tags';",
            [],
            |r| r.get::<_, i32>(0).map(|c| c > 0)
        ).unwrap();
        assert!(tag_exists);

        let phrase_exists: bool = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='player_tag_phrases';",
            [],
            |r| r.get::<_, i32>(0).map(|c| c > 0)
        ).unwrap();
        assert!(phrase_exists);
    }

    #[test]
    fn test_encountered_9_peers_batch_and_deduplication() {
        let storage = create_test_storage();
        let self_puuid = "local-puuid-self";
        let peers: Vec<String> = (1..=9).map(|i| format!("peer-puuid-{}", i)).collect();

        // 1. Record 9 peers
        storage.record_encountered_game(
            1234567,
            self_puuid,
            &peers,
            "TENCENT",
            "HN1",
            "RANKED_SOLO_5x5",
        ).unwrap();

        // Query peer 1
        let res = storage.query_encountered_games(&EncounteredGamesQueryDto {
            puuid: "peer-puuid-1".to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: None,
            page: Some(1),
            page_size: Some(10),
            time_order: Some("desc".to_string()),
        }).unwrap();
        assert_eq!(res.total, 1);
        assert_eq!(res.data.len(), 1);
        assert_eq!(res.data[0].game_id, 1234567);
        assert_eq!(res.data[0].queue_type, "RANKED_SOLO_5x5");

        // 2. Duplicate insertion with same game_id & peer
        storage.record_encountered_game(
            1234567,
            self_puuid,
            &peers,
            "TENCENT",
            "HN1",
            "RANKED_SOLO_5x5",
        ).unwrap();

        // Total should still be 1 (deduplicated by UNIQUE index)
        let res_after = storage.query_encountered_games(&EncounteredGamesQueryDto {
            puuid: "peer-puuid-1".to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: None,
            page: Some(1),
            page_size: Some(10),
            time_order: Some("desc".to_string()),
        }).unwrap();
        assert_eq!(res_after.total, 1);

        // Delete encounter
        let enc_id = res_after.data[0].id;
        let deleted = storage.delete_encountered_game(enc_id).unwrap();
        assert!(deleted);

        let res_deleted = storage.query_encountered_games(&EncounteredGamesQueryDto {
            puuid: "peer-puuid-1".to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: None,
            page: Some(1),
            page_size: Some(10),
            time_order: Some("desc".to_string()),
        }).unwrap();
        assert_eq!(res_deleted.total, 0);
    }

    #[test]
    fn test_paginated_queries_and_queue_filter() {
        let storage = create_test_storage();
        let self_puuid = "local-puuid-self";
        let target_puuid = "peer-rival";

        for gid in 1001..=1025 {
            let q_type = if gid % 2 == 0 { "RANKED_SOLO_5x5" } else { "ARAM" };
            storage.record_encountered_game(
                gid,
                self_puuid,
                &[target_puuid.to_string()],
                "TENCENT",
                "HN1",
                q_type,
            ).unwrap();
        }

        // All 25 games (page 1, size 10)
        let p1 = storage.query_encountered_games(&EncounteredGamesQueryDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: None,
            page: Some(1),
            page_size: Some(10),
            time_order: Some("desc".to_string()),
        }).unwrap();
        assert_eq!(p1.total, 25);
        assert_eq!(p1.total_pages, 3);
        assert_eq!(p1.data.len(), 10);

        // Page 3 (size 10 -> 5 remaining)
        let p3 = storage.query_encountered_games(&EncounteredGamesQueryDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: None,
            page: Some(3),
            page_size: Some(10),
            time_order: Some("desc".to_string()),
        }).unwrap();
        assert_eq!(p3.data.len(), 5);

        // Filter by queue_type = "ARAM"
        let p_aram = storage.query_encountered_games(&EncounteredGamesQueryDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            queue_type: Some("ARAM".to_string()),
            page: Some(1),
            page_size: Some(50),
            time_order: Some("desc".to_string()),
        }).unwrap();
        assert_eq!(p_aram.total, 13);
        assert_eq!(p_aram.data.len(), 13);
    }

    #[test]
    fn test_saved_player_tag_crud_and_marked_by_self() {
        let storage = create_test_storage();
        let self_puuid = "puuid-alice";
        let other_puuid = "puuid-bob";
        let target_puuid = "puuid-charlie";

        // Alice tags Charlie
        let saved = storage.update_player_tag(&UpdatePlayerTagDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            region: Some("TENCENT".to_string()),
            rso_platform_id: Some("HN1".to_string()),
            tag: Some("绝活亚索".to_string()),
        }).unwrap();
        assert_eq!(saved.tag.as_deref(), Some("绝活亚索"));

        // Bob tags Charlie
        storage.update_player_tag(&UpdatePlayerTagDto {
            puuid: target_puuid.to_string(),
            self_puuid: other_puuid.to_string(),
            region: Some("TENCENT".to_string()),
            rso_platform_id: Some("HN1".to_string()),
            tag: Some("演员".to_string()),
        }).unwrap();

        // Alice queries Charlie's tags: should see both, with Alice's tag having marked_by_self = true
        let tags_seen_by_alice = storage.get_player_tags(target_puuid, self_puuid).unwrap();
        assert_eq!(tags_seen_by_alice.len(), 2);
        let alice_tag = tags_seen_by_alice.iter().find(|t| t.tag == "绝活亚索").unwrap();
        assert!(alice_tag.marked_by_self);
        let bob_tag = tags_seen_by_alice.iter().find(|t| t.tag == "演员").unwrap();
        assert!(!bob_tag.marked_by_self);

        // Check tagged players in lobby
        let lobby = vec![target_puuid.to_string(), "puuid-unmarked".to_string()];
        let alerted = storage.check_tagged_players(self_puuid, &lobby).unwrap();
        assert_eq!(alerted.len(), 2);

        // Delete tag
        let deleted = storage.delete_saved_player(target_puuid, self_puuid).unwrap();
        assert!(deleted);
        let after_del = storage.get_saved_player(target_puuid, self_puuid).unwrap();
        assert!(after_del.is_none());
    }

    #[test]
    fn test_phrase_normalization_and_constraints() {
        let raw = vec![
            "   绝活中单   ".to_string(),
            "".to_string(),
            "   ".to_string(),
            "绝活中单".to_string(), // duplicate
            "a".repeat(150),       // over 100 chars
            "演员".to_string(),
            "车队".to_string(),
        ];

        let normalized = normalize_saved_player_tag_phrases(&raw);
        assert_eq!(normalized.len(), 4);
        assert_eq!(normalized[0], "绝活中单");
        assert_eq!(normalized[1].len(), 100);
        assert_eq!(normalized[2], "演员");
        assert_eq!(normalized[3], "车队");

        // Test persistence in storage
        let storage = create_test_storage();
        let saved_phrases = storage.update_player_tag_phrases(&raw).unwrap();
        assert_eq!(saved_phrases.len(), 4);
        let loaded = storage.get_player_tag_phrases().unwrap();
        assert_eq!(loaded, saved_phrases);
    }

    #[test]
    fn test_corruption_recovery_on_disk() {
        let temp_dir = std::env::temp_dir().join(format!("hexbrain_test_{}", now_ms()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("corrupt_test.db");

        // Write corrupt bytes
        std::fs::write(&db_path, b"NOT A VALID SQLITE FILE CORRUPT DATA 1234567890").unwrap();

        // Open with recovery
        let storage = StorageManager::open(&db_path).expect("Corruption recovery should reinitialize DB");
        let conn = storage.conn.lock().unwrap();
        let check: String = conn.query_row("PRAGMA quick_check;", [], |r| r.get(0)).unwrap();
        assert_eq!(check, "ok");

        // Verify corrupted file was backed up
        let mut found_corrupt = false;
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains(".db.corrupted.") {
                    found_corrupt = true;
                    break;
                }
            }
        }
        assert!(found_corrupt, "Expected timestamped .corrupted backup file to exist");

        // Cleanup temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_query_saved_player_with_games_and_counts() {
        let storage = create_test_storage();
        let self_puuid = "alice-self";
        let target_puuid = "bob-target";

        // Record 3 encounters
        for gid in [2001, 2002, 2003] {
            storage.record_encountered_game(
                gid,
                self_puuid,
                &[target_puuid.to_string()],
                "TENCENT",
                "HN1",
                "RANKED_SOLO_5x5",
            ).unwrap();
        }

        // Tag Bob
        storage.update_player_tag(&UpdatePlayerTagDto {
            puuid: target_puuid.to_string(),
            self_puuid: self_puuid.to_string(),
            region: Some("TENCENT".to_string()),
            rso_platform_id: Some("HN1".to_string()),
            tag: Some("常打野带节奏".to_string()),
        }).unwrap();

        // Count encounters
        let count = storage.count_encountered_games(self_puuid, target_puuid).unwrap();
        assert_eq!(count, 3);

        // Combined query
        let combined = storage.query_saved_player_with_games(target_puuid, self_puuid, None).unwrap();
        assert!(combined.player.is_some());
        assert_eq!(combined.player.unwrap().tag.as_deref(), Some("常打野带节奏"));
        assert_eq!(combined.tags.len(), 1);
        assert!(combined.tags[0].marked_by_self);
        assert_eq!(combined.encountered_games.total, 3);
        assert_eq!(combined.encountered_games.data.len(), 3);
    }

    #[test]
    fn test_query_all_player_tags_pagination_and_ordering() {
        let storage = create_test_storage();
        let self_puuid = "marker-self";

        // Insert tags for 5 players
        for i in 1..=5 {
            storage.update_player_tag(&UpdatePlayerTagDto {
                puuid: format!("player-puuid-{}", i),
                self_puuid: self_puuid.to_string(),
                region: Some("TENCENT".to_string()),
                rso_platform_id: Some("HN1".to_string()),
                tag: Some(format!("Tag {}", i)),
            }).unwrap();
        }

        // Query all with page_size = 2
        let p1 = storage.query_all_player_tags(&QueryAllPlayerTagsDto {
            page: Some(1),
            page_size: Some(2),
            time_order: Some("desc".to_string()),
            ..Default::default()
        }).unwrap();
        assert_eq!(p1.total, 5);
        assert_eq!(p1.data.len(), 2);

        // Query all with time_order = "asc"
        let p_asc = storage.query_all_player_tags(&QueryAllPlayerTagsDto {
            page: Some(1),
            page_size: Some(10),
            time_order: Some("asc".to_string()),
            ..Default::default()
        }).unwrap();
        assert_eq!(p_asc.total, 5);
        assert_eq!(p_asc.data.len(), 5);
    }

    #[test]
    fn test_app_settings_and_alert_toggles() {
        let storage = create_test_storage();

        // Default phrase panel expanded is true
        assert!(storage.is_phrase_panel_expanded().unwrap());
        storage.set_phrase_panel_expanded(false).unwrap();
        assert!(!storage.is_phrase_panel_expanded().unwrap());

        // Default champ select alert enabled is true
        assert!(storage.is_champ_select_alert_enabled().unwrap());
        storage.set_champ_select_alert_enabled(false).unwrap();
        assert!(!storage.is_champ_select_alert_enabled().unwrap());
        storage.set_champ_select_alert_enabled(true).unwrap();
        assert!(storage.is_champ_select_alert_enabled().unwrap());
    }
}

