use sqlx::FromRow;
use std::str::FromStr;

// 🟢 Re-export untuk backward compatibility (mencegah error di handler lama)
pub use super::commands::ModerationCommand;

// ==========================================
// 1. MODERATION DATABASE MODELS
// ==========================================

/// Model data untuk pengaturan warn per chat/grup
#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct WarnSetting {
    pub chat_id: i64,
    pub warn_limit: i32,
    pub warn_action: String,
    pub log_channel_id: Option<i64>,
    pub clean_interval: Option<String>,
    pub last_cleaned_at: Option<i64>,
    pub lock_warns: i32,
}

/// Model data untuk tracking akumulasi warn user
#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct UserWarn {
    pub chat_id: i64,
    pub user_id: i64,
    pub warn_count: i32,
}

/// Model data untuk entri kata terlarang (Blacklist)
#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct BlacklistEntry {
    pub id: i64,
    pub chat_id: i64,
    pub word: String,
    pub action: String,
    pub slot_id: i32,
}

/// Model data untuk status pengajuan banding (Warn Appeal)
#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct WarnAppeal {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub message_id: i64,
    pub log_message_id: Option<i64>,
    pub status: String,
    pub created_at: Option<i64>,
}

// ==========================================
// 2. LOCKS & ALLOWLIST MODELS
// ==========================================

/// Wrapper Struct Tipe Lock (Mendukung 50+ Tipe Lock Rose Bot)
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LockType(pub String);

impl LockType {
    /// Cek apakah tipe lock yang dimasukkan pengguna valid
    pub fn is_valid(s: &str) -> bool {
        matches!(
            s,
            "all"
                | "album"
                | "anonchannel"
                | "audio"
                | "bot"
                | "botlink"
                | "button"
                | "cashtag"
                | "checklist"
                | "cjk"
                | "collage"
                | "command"
                | "comment"
                | "contact"
                | "cyrillic"
                | "document"
                | "email"
                | "emoji"
                | "emojicustom"
                | "emojigame"
                | "emojionly"
                | "externalreply"
                | "forward"
                | "forwardbot"
                | "forwardchannel"
                | "forwardstory"
                | "forwarduser"
                | "game"
                | "gif"
                | "guestbot"
                | "inline"
                | "invitelink"
                | "location"
                | "outsidereaction"
                | "phone"
                | "photo"
                | "poll"
                | "reaction"
                | "richmessage"
                | "rtl"
                | "slideshow"
                | "spoiler"
                | "sticker"
                | "stickeranimated"
                | "stickerpremium"
                | "text"
                | "url"
                | "video"
                | "videonote"
                | "voice"
                | "zalgo"
        )
    }

    /// Mengembalikan representasi &str dari LockType
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for LockType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.to_lowercase();
        let trimmed = clean.trim();

        // Normalisasi kata jamak dan alias umum
        let normalized = match trimmed {
            "stickers" => "sticker",
            "urls" | "link" | "links" => "url",
            "invitelinks" => "invitelink",
            "forwards" => "forward",
            "photos" => "photo",
            "videos" => "video",
            "documents" => "document",
            "bots" => "bot",
            "commands" => "command",
            other => other,
        };

        if Self::is_valid(normalized) {
            Ok(LockType(normalized.to_string()))
        } else {
            Err(format!("Tipe lock '{}' tidak valid.", s))
        }
    }
}

impl std::fmt::Display for LockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Model data untuk chat_locks (status lock per grup)
#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct ChatLock {
    pub chat_id: i64,
    pub lock_type: String,
}

/// Model data untuk chat_allowlist (daftar domain/URL/username yang diizinkan)
#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct AllowlistItem {
    pub id: i64,
    pub chat_id: i64,
    pub value: String,
}

// ==========================================
// 3. TARGET RESOLVER & CALLBACK MODELS
// ==========================================

/// Struct penampung data Target User hasil resolusi
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TargetUser {
    pub user_id: u64,
    pub username: Option<String>,
    pub first_name: String,
}

/// Structural Payload untuk callback data tombol Appeal
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppealCallback {
    pub target_user_id: u64,
}

impl AppealCallback {
    pub fn to_callback_data(&self) -> String {
        format!("appeal:{}", self.target_user_id)
    }

    pub fn parse(data: &str) -> Option<Self> {
        if let Some(id_str) = data.strip_prefix("appeal:") {
            if let Ok(target_user_id) = id_str.parse::<u64>() {
                return Some(Self { target_user_id });
            }
        }
        None
    }
}
