use rusqlite::Connection;
use crate::storage::error::StorageError;

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

pub fn run_migrations(conn: &mut Connection) -> Result<(), StorageError> {
    let current_version: i32 = conn.query_row("PRAGMA user_version;", [], |row| row.get(0))?;

    if current_version < 1 {
        apply_v1_migration(conn)?;
    }

    Ok(())
}

fn apply_v1_migration(conn: &mut Connection) -> Result<(), StorageError> {
    let tx = conn.transaction()?;

    tx.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS encountered_games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id INTEGER NOT NULL,
            puuid TEXT NOT NULL,
            self_puuid TEXT NOT NULL,
            region TEXT NOT NULL,
            rso_platform_id TEXT NOT NULL,
            update_at INTEGER NOT NULL,
            queue_type TEXT NOT NULL DEFAULT ''
        );

        CREATE INDEX IF NOT EXISTS idx_encountered_games_game_id 
            ON encountered_games(game_id);
        CREATE INDEX IF NOT EXISTS idx_encountered_games_puuid 
            ON encountered_games(puuid);
        CREATE INDEX IF NOT EXISTS idx_encountered_games_self_puuid 
            ON encountered_games(self_puuid);
        CREATE INDEX IF NOT EXISTS idx_encountered_games_self_puuid_puuid_update_at 
            ON encountered_games(self_puuid, puuid, update_at DESC);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_enc_games_unique 
            ON encountered_games(game_id, self_puuid, puuid);

        CREATE TABLE IF NOT EXISTS saved_player_tags (
            puuid TEXT NOT NULL,
            self_puuid TEXT NOT NULL,
            region TEXT NOT NULL,
            rso_platform_id TEXT NOT NULL,
            tag TEXT,
            update_at INTEGER NOT NULL,
            last_met_at INTEGER,
            PRIMARY KEY (puuid, self_puuid, region, rso_platform_id)
        );

        CREATE INDEX IF NOT EXISTS idx_saved_player_tags_update_at 
            ON saved_player_tags(update_at);
        CREATE INDEX IF NOT EXISTS idx_saved_player_tags_last_met_at 
            ON saved_player_tags(last_met_at);
        CREATE INDEX IF NOT EXISTS idx_saved_player_tags_self_puuid_puuid 
            ON saved_player_tags(self_puuid, puuid);

        CREATE TABLE IF NOT EXISTS player_tag_phrases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            phrase TEXT NOT NULL UNIQUE,
            sort_order INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#
    )?;

    // Legacy LeagueAkari table check and migration
    let has_legacy_encountered: bool = tx.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='EncounteredGames'",
        [],
        |row| row.get::<_, i32>(0).map(|c| c > 0)
    ).unwrap_or(false);

    if has_legacy_encountered {
        let _ = tx.execute(
            r#"
            INSERT OR IGNORE INTO encountered_games (game_id, puuid, self_puuid, region, rso_platform_id, update_at, queue_type)
            SELECT gameId, puuid, selfPuuid, region, rsoPlatformId, 
                   strftime('%s', updateAt) * 1000, 
                   COALESCE(queueType, '')
            FROM EncounteredGames;
            "#,
            []
        );
    }

    let has_legacy_saved_players: bool = tx.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='SavedPlayers'",
        [],
        |row| row.get::<_, i32>(0).map(|c| c > 0)
    ).unwrap_or(false);

    if has_legacy_saved_players {
        let _ = tx.execute(
            r#"
            INSERT OR IGNORE INTO saved_player_tags (puuid, self_puuid, region, rso_platform_id, tag, update_at, last_met_at)
            SELECT puuid, selfPuuid, region, rsoPlatformId, tag,
                   strftime('%s', updateAt) * 1000,
                   CASE WHEN lastMetAt IS NOT NULL THEN strftime('%s', lastMetAt) * 1000 ELSE NULL END
            FROM SavedPlayers;
            "#,
            []
        );
    }

    tx.commit()?;
    conn.pragma_update(None, "user_version", CURRENT_SCHEMA_VERSION)?;
    Ok(())
}
