// src/db/user_cache.rs
use sqlx::{Pool, Sqlite};

pub type DbPool = Pool<Sqlite>;

/// Menyimpan atau memperbarui data user (user_id, username, first_name) ke SQLite
pub async fn upsert_user_cache(
    pool: &DbPool,
    user_id: u64,
    username: Option<&str>,
    first_name: &str,
) -> Result<(), sqlx::Error> {
    let user_id_i64 = user_id as i64;
    let clean_username = username.map(|u| u.trim_start_matches('@').to_lowercase());

    sqlx::query(
        r#"
        INSERT INTO user_cache (user_id, username, first_name, updated_at)
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP)
        ON CONFLICT(user_id) DO UPDATE SET
            username = excluded.username,
            first_name = excluded.first_name,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(user_id_i64)
    .bind(clean_username)
    .bind(first_name)
    .execute(pool)
    .await?;

    Ok(())
}

/// Mencari user_id berdasarkan @username dari cache SQLite
pub async fn get_user_id_by_username(
    pool: &DbPool,
    username: &str,
) -> Result<Option<u64>, sqlx::Error> {
    let clean_username = username.trim_start_matches('@').to_lowercase();

    let row: Option<(i64,)> =
        sqlx::query_as("SELECT user_id FROM user_cache WHERE LOWER(username) = $1 LIMIT 1")
            .bind(clean_username)
            .fetch_optional(pool)
            .await?;

    Ok(row.map(|r| r.0 as u64))
}

/// Mengambil first_name dan username berdasarkan user_id dari cache
pub async fn get_user_by_id(
    pool: &DbPool,
    user_id: u64,
) -> Result<Option<(String, Option<String>)>, sqlx::Error> {
    let user_id_i64 = user_id as i64;

    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT first_name, username FROM user_cache WHERE user_id = $1 LIMIT 1")
            .bind(user_id_i64)
            .fetch_optional(pool)
            .await?;

    Ok(row)
}
