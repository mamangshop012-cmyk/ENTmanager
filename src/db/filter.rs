// src/db/filter.rs
use sqlx::Pool;
use sqlx::Sqlite;

pub type DbPool = Pool<Sqlite>;

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct FilterEntry {
    pub trigger_word: String,
    pub reply_text: String,
}

/// Menyimpan atau memperbarui (UPSERT) filter balasan otomatis per grup
pub async fn add_filter(
    pool: &DbPool,
    chat_id: i64,
    trigger: &str,
    reply: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO filters (chat_id, trigger_word, reply_text)
        VALUES ($1, $2, $3)
        ON CONFLICT(chat_id, trigger_word) DO UPDATE SET reply_text = excluded.reply_text
        "#,
    )
    .bind(chat_id)
    .bind(trigger.to_lowercase())
    .bind(reply)
    .execute(pool)
    .await?;

    Ok(())
}

/// Mengambil seluruh filter aktif untuk grup tertentu
pub async fn get_filters(pool: &DbPool, chat_id: i64) -> Result<Vec<FilterEntry>, sqlx::Error> {
    let entries = sqlx::query_as::<_, FilterEntry>(
        r#"
        SELECT trigger_word, reply_text FROM filters WHERE chat_id = $1 ORDER BY trigger_word ASC
        "#,
    )
    .bind(chat_id)
    .fetch_all(pool)
    .await?;

    Ok(entries)
}

/// Menghapus satu filter kata spesifik pada grup tertentu
pub async fn delete_filter(
    pool: &DbPool,
    chat_id: i64,
    trigger: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM filters WHERE chat_id = $1 AND trigger_word = $2
        "#,
    )
    .bind(chat_id)
    .bind(trigger.to_lowercase())
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Menghapus SELURUH filter kata pada grup tertentu
pub async fn delete_all_filters(pool: &DbPool, chat_id: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM filters WHERE chat_id = $1
        "#,
    )
    .bind(chat_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
