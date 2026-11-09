// src/db/moderation.rs
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

pub type DbPool = Pool<Sqlite>;

pub use crate::db::user_cache::{get_user_by_id, get_user_id_by_username, upsert_user_cache};
pub use crate::db::welcome::get_welcome_info;
pub use get_blacklist_entries as get_blacklist;

// ========================================================================
// 📊 MODELS / STRUCTS
// ========================================================================

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct WarnSetting {
    pub chat_id: i64,
    pub warn_limit: i32,
    pub warn_action: String,
    pub log_channel_id: Option<i64>,
    pub clean_interval: Option<String>,
    pub last_cleaned_at: Option<i64>,
    pub lock_warns: i32,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct UserWarn {
    pub chat_id: i64,
    pub user_id: i64,
    pub warn_count: i32,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct BlacklistEntry {
    pub id: i64,
    pub chat_id: i64,
    pub word: String,
    pub action: String,
    pub slot_id: i32,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct AllowlistItem {
    pub id: i64,
    pub chat_id: i64,
    pub value: String,
}

// ========================================================================
// 1. WARN SETTINGS & LOG CHANNEL
// ========================================================================

pub async fn get_or_create_warn_settings(
    pool: &DbPool,
    chat_id: i64,
) -> Result<WarnSetting, sqlx::Error> {
    let setting =
        sqlx::query_as::<_, WarnSetting>("SELECT * FROM warn_settings WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_optional(pool)
            .await?;

    if let Some(s) = setting {
        Ok(s)
    } else {
        sqlx::query(
            "INSERT INTO warn_settings (chat_id, warn_limit, warn_action) VALUES ($1, 3, 'ban')",
        )
        .bind(chat_id)
        .execute(pool)
        .await?;

        Ok(WarnSetting {
            chat_id,
            warn_limit: 3,
            warn_action: "ban".to_string(),
            log_channel_id: None,
            clean_interval: None,
            last_cleaned_at: None,
            lock_warns: 0,
        })
    }
}

pub async fn set_warn_limit(pool: &DbPool, chat_id: i64, limit: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO warn_settings (chat_id, warn_limit) VALUES ($1, $2)
        ON CONFLICT(chat_id) DO UPDATE SET warn_limit = excluded.warn_limit
        "#,
    )
    .bind(chat_id)
    .bind(limit)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_warn_action(pool: &DbPool, chat_id: i64, action: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO warn_settings (chat_id, warn_action) VALUES ($1, $2)
        ON CONFLICT(chat_id) DO UPDATE SET warn_action = excluded.warn_action
        "#,
    )
    .bind(chat_id)
    .bind(action)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_log_channel(pool: &DbPool, chat_id: i64) -> Result<Option<i64>, sqlx::Error> {
    let row: Option<(Option<i64>,)> =
        sqlx::query_as("SELECT log_channel_id FROM warn_settings WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_optional(pool)
            .await?;

    Ok(row.and_then(|(log_id,)| log_id))
}

pub async fn set_log_channel(
    pool: &DbPool,
    chat_id: i64,
    log_channel_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO warn_settings (chat_id, log_channel_id) VALUES ($1, $2)
        ON CONFLICT(chat_id) DO UPDATE SET log_channel_id = excluded.log_channel_id
        "#,
    )
    .bind(chat_id)
    .bind(log_channel_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_clean_interval(
    pool: &DbPool,
    chat_id: i64,
    interval: Option<&str>,
) -> Result<(), sqlx::Error> {
    get_or_create_warn_settings(pool, chat_id).await?;

    sqlx::query("UPDATE warn_settings SET clean_interval = $1, last_cleaned_at = (unixepoch()) WHERE chat_id = $2")
        .bind(interval)
        .bind(chat_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn set_lock_warns(pool: &DbPool, chat_id: i64, enabled: bool) -> Result<(), sqlx::Error> {
    get_or_create_warn_settings(pool, chat_id).await?;
    let flag = if enabled { 1 } else { 0 };

    sqlx::query("UPDATE warn_settings SET lock_warns = $1 WHERE chat_id = $2")
        .bind(flag)
        .bind(chat_id)
        .execute(pool)
        .await?;

    Ok(())
}

// ========================================================================
// 2. USER WARNS
// ========================================================================

pub async fn get_user_warns(pool: &DbPool, chat_id: i64, user_id: i64) -> Result<i32, sqlx::Error> {
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT warn_count FROM user_warns WHERE chat_id = $1 AND user_id = $2")
            .bind(chat_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    Ok(row.map(|r| r.0).unwrap_or(0))
}

pub async fn increment_user_warn(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
) -> Result<i32, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_warns (chat_id, user_id, warn_count) VALUES ($1, $2, 1)
        ON CONFLICT(chat_id, user_id) DO UPDATE SET warn_count = warn_count + 1
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    get_user_warns(pool, chat_id, user_id).await
}

pub async fn decrement_user_warn(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
) -> Result<i32, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let warn = sqlx::query_as::<_, UserWarn>(
        "SELECT * FROM user_warns WHERE chat_id = $1 AND user_id = $2",
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;

    let new_count = match warn {
        Some(w) => (w.warn_count - 1).max(0),
        None => 0,
    };

    sqlx::query(
        r#"
        INSERT INTO user_warns (chat_id, user_id, warn_count) VALUES ($1, $2, $3)
        ON CONFLICT(chat_id, user_id) DO UPDATE SET warn_count = $4
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(new_count)
    .bind(new_count)
    .execute(&mut *conn)
    .await?;

    Ok(new_count)
}

pub async fn reset_user_warns(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_warns WHERE chat_id = $1 AND user_id = $2")
        .bind(chat_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn reset_all_warns(pool: &DbPool, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_warns WHERE chat_id = $1")
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ========================================================================
// 3. HYBRID BLACKLIST & BULK SLOTS
// ========================================================================

pub async fn add_blacklist_word(
    pool: &DbPool,
    chat_id: i64,
    word: &str,
    action: &str,
    slot_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO blacklist (chat_id, word, action, slot_id) VALUES ($1, $2, $3, $4)
        ON CONFLICT(chat_id, word) DO UPDATE SET action = excluded.action, slot_id = excluded.slot_id
        "#,
    )
    .bind(chat_id)
    .bind(word.to_lowercase())
    .bind(action)
    .bind(slot_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_blacklist_word(
    pool: &DbPool,
    chat_id: i64,
    word: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM blacklist WHERE chat_id = $1 AND word = $2")
        .bind(chat_id)
        .bind(word.to_lowercase())
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

pub async fn get_blacklist_entries(
    pool: &DbPool,
    chat_id: i64,
) -> Result<Vec<BlacklistEntry>, sqlx::Error> {
    sqlx::query_as::<_, BlacklistEntry>(
        "SELECT id, chat_id, word, action, slot_id FROM blacklist WHERE chat_id = $1 ORDER BY id ASC",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await
}

pub async fn set_slot_action(
    pool: &DbPool,
    chat_id: i64,
    slot_id: i32,
    action: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO slot_settings (chat_id, slot_id, action) VALUES ($1, $2, $3)
        ON CONFLICT(chat_id, slot_id) DO UPDATE SET action = excluded.action
        "#,
    )
    .bind(chat_id)
    .bind(slot_id)
    .bind(action)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_slot_action(
    pool: &DbPool,
    chat_id: i64,
    slot_id: i32,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT action FROM slot_settings WHERE chat_id = $1 AND slot_id = $2")
            .bind(chat_id)
            .bind(slot_id)
            .fetch_optional(pool)
            .await?;

    Ok(row.map(|r| r.0))
}

pub async fn get_bulk_slots(
    pool: &DbPool,
    chat_id: i64,
) -> Result<HashMap<u8, String>, sqlx::Error> {
    let rows: Vec<(i32, String)> =
        sqlx::query_as("SELECT slot_id, action FROM slot_settings WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_all(pool)
            .await?;

    let mut map = HashMap::new();
    for (slot_id, action) in rows {
        map.insert(slot_id as u8, action);
    }
    Ok(map)
}

// ========================================================================
// 4. WARN APPEALS
// ========================================================================

pub async fn create_appeal(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
    message_id: i64,
) -> Result<i64, sqlx::Error> {
    let id = sqlx::query(
        "INSERT INTO warn_appeals (chat_id, user_id, message_id, status) VALUES ($1, $2, $3, 'pending')",
    )
    .bind(chat_id)
    .bind(user_id)
    .bind(message_id)
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

pub async fn update_appeal_log_msg(
    pool: &DbPool,
    appeal_id: i64,
    log_msg_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE warn_appeals SET log_message_id = $1 WHERE id = $2")
        .bind(log_msg_id)
        .bind(appeal_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_appeal_status(
    pool: &DbPool,
    appeal_id: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE warn_appeals SET status = $1 WHERE id = $2")
        .bind(status)
        .bind(appeal_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn has_pending_appeal(
    pool: &DbPool,
    chat_id: i64,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let mut conn = pool.acquire().await?;

    let exists: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM warn_appeals
            WHERE chat_id = $1
              AND user_id = $2
              AND status = 'pending'
              AND created_at > (strftime('%s', 'now') - 600)
        )
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_one(&mut *conn)
    .await?;

    Ok(exists.0)
}

// ========================================================================
// 5. LOCKS & ALLOWLIST
// ========================================================================

pub async fn set_lock(
    pool: &DbPool,
    chat_id: i64,
    lock_type: &str,
    enable: bool,
) -> Result<(), sqlx::Error> {
    if enable {
        sqlx::query(
            r#"
            INSERT INTO chat_locks (chat_id, lock_type) VALUES ($1, $2)
            ON CONFLICT(chat_id, lock_type) DO NOTHING
            "#,
        )
        .bind(chat_id)
        .bind(lock_type)
        .execute(pool)
        .await?;
    } else {
        sqlx::query("DELETE FROM chat_locks WHERE chat_id = $1 AND lock_type = $2")
            .bind(chat_id)
            .bind(lock_type)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn get_active_locks(pool: &DbPool, chat_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT lock_type FROM chat_locks WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_all(pool)
            .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn is_locked(pool: &DbPool, chat_id: i64, lock_type: &str) -> Result<bool, sqlx::Error> {
    let result: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM chat_locks WHERE chat_id = $1 AND lock_type = $2)",
    )
    .bind(chat_id)
    .bind(lock_type)
    .fetch_one(pool)
    .await?;

    Ok(result.0)
}

pub async fn add_allowlist_item(
    pool: &DbPool,
    chat_id: i64,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO chat_allowlist (chat_id, value) VALUES ($1, $2)
        ON CONFLICT(chat_id, value) DO NOTHING
        "#,
    )
    .bind(chat_id)
    .bind(value.to_lowercase())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_allowlist_item(
    pool: &DbPool,
    chat_id: i64,
    value: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM chat_allowlist WHERE chat_id = $1 AND value = $2")
        .bind(chat_id)
        .bind(value.to_lowercase())
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_allowlist(pool: &DbPool, chat_id: i64) -> Result<Vec<AllowlistItem>, sqlx::Error> {
    sqlx::query_as::<_, AllowlistItem>(
        "SELECT * FROM chat_allowlist WHERE chat_id = $1 ORDER BY id ASC",
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await
}

pub async fn clear_allowlist(pool: &DbPool, chat_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM chat_allowlist WHERE chat_id = $1")
        .bind(chat_id)
        .execute(pool)
        .await?;
    Ok(())
}
