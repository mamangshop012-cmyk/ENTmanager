// src/db/mod.rs
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::fs::File;
use std::path::Path;

pub mod filter;
pub mod moderation;
pub mod user_cache;
pub mod welcome;

pub use filter::*;
pub use moderation::*;
pub use user_cache::*;
pub use welcome::*;

pub type DbPool = Pool<Sqlite>;

/// Inisialisasi koneksi SQLite & migrasi seluruh tabel
pub async fn init_db() -> Result<DbPool, sqlx::Error> {
    if !Path::new("database.db").exists() {
        File::create("database.db").expect("Gagal membuat file database.db");
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://database.db")
        .await?;

    // 1. Tabel Welcome Settings
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS welcome_settings (
            chat_id BIGINT PRIMARY KEY,
            is_enabled BOOLEAN NOT NULL DEFAULT 0,
            welcome_text TEXT NOT NULL DEFAULT 'Selamat datang {first} di {chatname}!',
            clean_welcome BOOLEAN NOT NULL DEFAULT 0,
            media_type TEXT DEFAULT NULL,
            file_id TEXT DEFAULT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    let _ = sqlx::query("ALTER TABLE welcome_settings ADD COLUMN media_type TEXT DEFAULT NULL;")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE welcome_settings ADD COLUMN file_id TEXT DEFAULT NULL;")
        .execute(&pool)
        .await;

    // 2. Tabel Warn Settings
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS warn_settings (
            chat_id BIGINT PRIMARY KEY,
            warn_limit INTEGER NOT NULL DEFAULT 3,
            warn_action TEXT NOT NULL DEFAULT 'ban',
            log_channel_id BIGINT DEFAULT NULL,
            clean_interval TEXT DEFAULT NULL,
            last_cleaned_at BIGINT DEFAULT (unixepoch()),
            lock_warns INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
    .execute(&pool)
    .await?;

    let _ =
        sqlx::query("ALTER TABLE warn_settings ADD COLUMN lock_warns INTEGER NOT NULL DEFAULT 0;")
            .execute(&pool)
            .await;

    // 3. Tabel User Warns
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_warns (
            chat_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            warn_count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (chat_id, user_id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 4. Tabel Blacklist System
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blacklist (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id BIGINT NOT NULL,
            word TEXT NOT NULL,
            action TEXT NOT NULL DEFAULT 'warn',
            slot_id INTEGER NOT NULL DEFAULT 0,
            UNIQUE(chat_id, word)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    let _ = sqlx::query("ALTER TABLE blacklist ADD COLUMN slot_id INTEGER NOT NULL DEFAULT 0;")
        .execute(&pool)
        .await;

    // 5. Tabel Slot Settings
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS slot_settings (
            chat_id BIGINT NOT NULL,
            slot_id INTEGER NOT NULL,
            action TEXT NOT NULL DEFAULT 'tmute1d',
            PRIMARY KEY (chat_id, slot_id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 6. Tabel Warn Appeals
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS warn_appeals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id BIGINT NOT NULL,
            user_id BIGINT NOT NULL,
            message_id BIGINT NOT NULL,
            log_message_id BIGINT DEFAULT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at BIGINT DEFAULT (unixepoch())
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 7. Tabel Locks & Allowlist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat_locks (
            chat_id BIGINT NOT NULL,
            lock_type TEXT NOT NULL,
            PRIMARY KEY (chat_id, lock_type)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chat_allowlist (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id BIGINT NOT NULL,
            value TEXT NOT NULL,
            UNIQUE(chat_id, value)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 8. Tabel User Cache
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_cache (
            user_id BIGINT PRIMARY KEY,
            username TEXT,
            first_name TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 9. Tabel Auto-Responder Filters
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS filters (
            chat_id BIGINT NOT NULL,
            trigger_word TEXT NOT NULL,
            reply_text TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (chat_id, trigger_word)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    println!("✅ Database SQLite terhubung & terpusat di src/db/");

    Ok(pool)
}
