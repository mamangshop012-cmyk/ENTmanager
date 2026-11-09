use crate::moderation::{db as mod_db, models::TargetUser};
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{ChatId, Message, UserId},
};

/// Cek apakah user adalah Admin / Owner di grup
pub async fn is_admin(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    if let Ok(admins) = bot.get_chat_administrators(chat_id).await {
        admins.iter().any(|m| m.user.id == user_id)
    } else {
        false
    }
}

/// Mengubah format waktu seperti "30m", "2h", "1d" menjadi durasi detik
pub fn parse_duration(input: &str) -> Option<u64> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Menggunakan char_indices agar aman dari panic UTF-8 char boundary
    let (idx, unit) = input.char_indices().last()?;
    let num_str = &input[..idx];
    let num: u64 = num_str.parse().ok()?;

    match unit {
        's' | 'S' => Some(num),
        'm' | 'M' => Some(num * 60),
        'h' | 'H' => Some(num * 3600),
        'd' | 'D' => Some(num * 86400),
        'w' | 'W' => Some(num * 604800),
        _ => None,
    }
}

/// Helper Resolver Target: Mendeteksi via REPLY pesan atau via TEXT (@username / User ID)
pub async fn resolve_target(
    _bot: &Bot,
    msg: &Message,
    args: &str,
    pool: &SqlitePool,
) -> Result<(TargetUser, String), String> {
    // Skenario 1: Via Reply Pesan
    if let Some(reply) = msg.reply_to_message() {
        if let Some(user) = reply.from.as_ref() {
            let _ = mod_db::upsert_user_cache(
                pool,
                user.id.0,
                user.username.as_deref(),
                &user.first_name,
            )
            .await;

            let reason = args.trim().to_string();
            let target = TargetUser {
                user_id: user.id.0,
                username: user.username.clone(),
                first_name: user.first_name.clone(),
            };
            return Ok((target, reason));
        }
    }

    // Skenario 2: Via Text (@username / User ID)
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.is_empty() {
        return Err(
            "Target tidak ditemukan! Reply pesan user atau ketik `@username` / `User ID`."
                .to_string(),
        );
    }

    let raw_target = parts[0];
    let reason = parts[1..].join(" ");

    // Sub-Skenario A: Input berupa User ID Angka
    if let Ok(user_id) = raw_target.parse::<u64>() {
        let (first_name, username) = mod_db::get_user_by_id(pool, user_id)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| ("User".to_string(), None));

        return Ok((
            TargetUser {
                user_id,
                username,
                first_name,
            },
            reason,
        ));
    }

    // Sub-Skenario B: Input berupa @username
    if raw_target.starts_with('@') {
        if let Some(user_id) = mod_db::get_user_id_by_username(pool, raw_target)
            .await
            .ok()
            .flatten()
        {
            let (first_name, username) = mod_db::get_user_by_id(pool, user_id)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| (raw_target.to_string(), Some(raw_target.to_string())));

            return Ok((
                TargetUser {
                    user_id,
                    username,
                    first_name,
                },
                reason,
            ));
        }
        return Err(format!(
            "Username `{}` belum terdaftar di database bot. Harap gunakan **Reply Pesan** atau **User ID**.",
            raw_target
        ));
    }

    Err(
        "Format target tidak dikenal. Gunakan Reply pesan, `@username`, atau `User ID` angka."
            .to_string(),
    )
}

// ==========================================
// UNIT TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_valid() {
        assert_eq!(parse_duration("30s"), Some(30));
        assert_eq!(parse_duration("5m"), Some(300));
        assert_eq!(parse_duration("2h"), Some(7200));
        assert_eq!(parse_duration("1d"), Some(86400));
        assert_eq!(parse_duration("1w"), Some(604800));
    }

    #[test]
    fn test_parse_duration_case_insensitive() {
        assert_eq!(parse_duration("10M"), Some(600));
        assert_eq!(parse_duration("1H"), Some(3600));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert_eq!(parse_duration(""), None);
        assert_eq!(parse_duration("abc"), None);
        assert_eq!(parse_duration("10x"), None);
        assert_eq!(parse_duration("30分"), None); // Karakter non-ASCII aman tanpa panic
    }
}
