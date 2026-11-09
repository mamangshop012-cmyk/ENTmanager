// src/db/welcome.rs
use sqlx::{Pool, Sqlite};

pub type DbPool = Pool<Sqlite>;
pub use crate::db::user_cache::upsert_user_cache;

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct WelcomeSetting {
    pub chat_id: i64,
    pub is_enabled: bool,
    pub welcome_text: String,
    pub clean_welcome: bool,
    pub media_type: Option<String>,
    pub file_id: Option<String>,
}

/// Mengambil settingan welcome grup. Jika belum ada, kembalikan nilai default.
pub async fn get_welcome_setting(pool: &DbPool, chat_id: i64) -> WelcomeSetting {
    let result = sqlx::query_as::<_, WelcomeSetting>(
        "SELECT chat_id, is_enabled, welcome_text, clean_welcome, media_type, file_id FROM welcome_settings WHERE chat_id = $1"
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(setting)) => setting,
        _ => WelcomeSetting {
            chat_id,
            is_enabled: false,
            welcome_text: "Selamat datang {first} di {chatname}!".to_string(),
            clean_welcome: false,
            media_type: None,
            file_id: None,
        },
    }
}

/// Mengambil status Welcome System & Cleanwelcome dalam format UI
pub async fn get_welcome_info(
    pool: &DbPool,
    chat_id: i64,
) -> Result<(String, String), sqlx::Error> {
    let row: Option<(bool, bool)> =
        sqlx::query_as("SELECT is_enabled, clean_welcome FROM welcome_settings WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_optional(pool)
            .await?;

    if let Some((is_enabled, clean_welcome)) = row {
        let status = if is_enabled { "Active" } else { "Disabled" };
        let clean = if clean_welcome { "5 Min" } else { "Off" };
        Ok((status.to_string(), clean.to_string()))
    } else {
        Ok(("Disabled".to_string(), "Off".to_string()))
    }
}

/// Menyimpan atau mengupdate teks welcome kustom (murni teks)
pub async fn set_welcome_text(pool: &DbPool, chat_id: i64, text: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO welcome_settings (chat_id, is_enabled, welcome_text, clean_welcome, media_type, file_id)
        VALUES ($1, 1, $2, 0, NULL, NULL)
        ON CONFLICT(chat_id) DO UPDATE SET
            welcome_text = excluded.welcome_text,
            media_type = NULL,
            file_id = NULL,
            is_enabled = 1;
        "#,
    )
    .bind(chat_id)
    .bind(text)
    .execute(pool)
    .await?;

    Ok(())
}

/// Menyimpan atau mengupdate teks welcome kustom beserta media (Foto/GIF/Video)
pub async fn set_welcome_media(
    pool: &DbPool,
    chat_id: i64,
    text: &str,
    media_type: Option<&str>,
    file_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO welcome_settings (chat_id, is_enabled, welcome_text, clean_welcome, media_type, file_id)
        VALUES ($1, 1, $2, 0, $3, $4)
        ON CONFLICT(chat_id) DO UPDATE SET
            welcome_text = excluded.welcome_text,
            media_type = excluded.media_type,
            file_id = excluded.file_id,
            is_enabled = 1;
        "#,
    )
    .bind(chat_id)
    .bind(text)
    .bind(media_type)
    .bind(file_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Menyalakan/Mematikan pesan welcome (ON/OFF)
pub async fn toggle_welcome(pool: &DbPool, chat_id: i64, enable: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO welcome_settings (chat_id, is_enabled, welcome_text, clean_welcome)
        VALUES ($1, $2, 'Selamat datang {first} di {chatname}!', 0)
        ON CONFLICT(chat_id) DO UPDATE SET is_enabled = excluded.is_enabled;
        "#,
    )
    .bind(chat_id)
    .bind(enable)
    .execute(pool)
    .await?;

    Ok(())
}

/// Menyalakan/Mematikan fitur Auto-Delete 5 Menit (ON/OFF)
pub async fn toggle_clean_welcome(
    pool: &DbPool,
    chat_id: i64,
    enable: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO welcome_settings (chat_id, is_enabled, welcome_text, clean_welcome)
        VALUES ($1, 0, 'Selamat datang {first} di {chatname}!', $2)
        ON CONFLICT(chat_id) DO UPDATE SET clean_welcome = excluded.clean_welcome;
        "#,
    )
    .bind(chat_id)
    .bind(enable)
    .execute(pool)
    .await?;

    Ok(())
}
