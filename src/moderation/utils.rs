use chrono::Duration;
use regex::Regex;
use teloxide::prelude::*;
use teloxide::types::{ChatId, UserId};
use teloxide::utils::html::escape;

/// Structure hasil parsing input /addbl
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedBlacklistInput {
    pub words: Vec<String>,
    pub action: String,
}

// ==========================================
// 1. ANTI-BYPASS & NORMALISASI TEKS
// ==========================================

/// Normalisasi teks untuk mencocokkan kata terlarang (Anti-Bypass)
/// Mengubah huruf besar, leetspeak, dan membuang simbol khusus
pub fn normalize_text(text: &str) -> String {
    let lower = text.to_lowercase();

    // 1. Replace Leetspeak umum
    let leet_replaced = lower
        .replace('0', "o")
        .replace('1', "i")
        .replace('!', "i")
        .replace('|', "i")
        .replace('3', "e")
        .replace('4', "a")
        .replace('@', "a")
        .replace('5', "s")
        .replace('$', "s")
        .replace('7', "t");

    // 2. Buang karakter non-alphanumeric (simbol, spasi tersembunyi, tanda baca)
    leet_replaced
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

/// Mengecek apakah teks mengandung kata terlarang (dengan normalisasi)
pub fn contains_blacklisted_word(message: &str, blacklisted_word: &str) -> bool {
    let norm_msg = normalize_text(message);
    let norm_word = normalize_text(blacklisted_word);

    if norm_word.is_empty() {
        return false;
    }

    // Cek exact word match atau substring match
    norm_msg.split_whitespace().any(|word| word == norm_word) || norm_msg.contains(&norm_word)
}

// ==========================================
// 2. PARSER TAG ACTION (/addbl kata1, kata2 {tag})
// ==========================================

/// Memapar input perintah /addbl menjadi daftar kata dan tag aksi
/// Contoh Input: "kontol, memek, judi {tmute1d}"
/// Hasil: words = ["kontol", "memek", "judi"], action = "tmute1d"
pub fn parse_blacklist_input(input: &str) -> ParsedBlacklistInput {
    let tag_regex = Regex::new(r"\{([a-zA-Z0-9_]+)\}").unwrap();

    let mut action = "warn".to_string(); // Action default
    let mut cleaned_input = input.to_string();

    // Cari apakah ada tag {action} di dalam input
    if let Some(captures) = tag_regex.captures(input) {
        if let Some(matched_tag) = captures.get(1) {
            action = matched_tag.as_str().to_lowercase();
        }
        // Hapus tag {action} dari teks input kata
        cleaned_input = tag_regex.replace_all(input, "").to_string();
    }

    // Split kata-kata berdasarkan koma (,)
    let words: Vec<String> = cleaned_input
        .split(',')
        .map(|w| w.trim().to_lowercase())
        .filter(|w| !w.is_empty())
        .collect();

    ParsedBlacklistInput { words, action }
}

// ==========================================
// 3. PARSER DURASI WAKTU (1m, 1h, 1d, 2w)
// ==========================================

/// Mengubah string durasi singkat menjadi detik (u64)
/// Menggunakan logika terpusat dari `resolver::parse_duration`
pub fn parse_duration_to_seconds(duration_str: &str) -> Option<u64> {
    super::resolver::parse_duration(duration_str)
}

/// Helper untuk mengekstrak jenis aksi dan durasi dari tag aksi
/// Contoh: "tmute1d" -> ("tmute", Some(86400))
/// Contoh: "ban"     -> ("ban", None)
pub fn parse_action_and_duration(action_tag: &str) -> (String, Option<u64>) {
    let tag = action_tag.trim().to_lowercase();
    if tag.starts_with("tmute") {
        let duration_part = &tag["tmute".len()..];
        let seconds = parse_duration_to_seconds(duration_part);
        ("tmute".to_string(), seconds)
    } else if tag.starts_with("tban") {
        let duration_part = &tag["tban".len()..];
        let seconds = parse_duration_to_seconds(duration_part);
        ("tban".to_string(), seconds)
    } else {
        (tag, None)
    }
}

// ==========================================
// 4. HELPER MODERASI MANUAL & UI FORMATTING
// ==========================================

/// Memparse durasi dengan validasi aturan Telegram API.
/// Mengembalikan `chrono::Duration` untuk perintah /tban dan /tmute.
pub fn parse_duration(input: &str) -> Result<Duration, String> {
    let seconds = parse_duration_to_seconds(input.trim()).ok_or_else(|| {
        "Format durasi tidak valid. Contoh yang benar: 30m, 2h, 3d, 1w".to_string()
    })?;

    if seconds < 30 {
        return Err("Durasi terlalu singkat. Minimal pembatasan adalah 30 detik.".to_string());
    }
    if seconds > 31_536_000 {
        return Err("Durasi terlalu lama. Maksimal pembatasan adalah 365 hari.".to_string());
    }

    Ok(Duration::seconds(seconds as i64))
}

/// Membuat hyperlink nama user ke ID Telegram-nya (`tg://user?id=...`).
/// Nama user di-escape otomatis agar aman dari HTML parser Teloxide.
pub fn format_user_link(user_id: u64, first_name: &str) -> String {
    let safe_name = escape(first_name);
    format!("<a href=\"tg://user?id={user_id}\">{safe_name}</a>")
}

/// Mengubah detik menjadi teks manusiawi untuk output notifikasi grup.
pub fn format_duration_human(seconds: i64) -> String {
    if seconds >= 86400 {
        let days = seconds / 86400;
        format!("{days} hari")
    } else if seconds >= 3600 {
        let hours = seconds / 3600;
        format!("{hours} jam")
    } else if seconds >= 60 {
        let mins = seconds / 60;
        format!("{mins} menit")
    } else {
        format!("{seconds} detik")
    }
}

// ==========================================
// 5. HELPER HIERARKI KEKEBALAN (IMMUNITY CHECK)
// ==========================================

/// Cek Hierarki Kekebalan:
/// - Layer 1: Bot Owner (OWNER_ID dari .env) -> Kebal Global (Godmode di semua grup)
/// - Layer 2: Admin & Creator/Owner Grup -> Kebal Lokal (Hanya di grup tempat menjabat)
pub async fn is_user_immune(bot: &Bot, chat_id: ChatId, user_id: UserId) -> bool {
    // 👑 1. LAYER 1: BOT OWNER CHECK (Support multiple owner dari .env)
    if let Ok(owners_str) = std::env::var("OWNER_ID") {
        let is_bot_owner = owners_str
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .any(|id| id == user_id.0);

        if is_bot_owner {
            return true;
        }
    }

    // 🛡️ 2. LAYER 2: LOCAL GROUP ADMIN & CREATOR/OWNER CHECK
    match bot.get_chat_member(chat_id, user_id).await {
        Ok(member) => member.is_administrator() || member.is_owner(),
        Err(_) => false,
    }
}

// ==========================================
// UNIT TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("K0NT0L"), "kontol");
        assert_eq!(normalize_text("J_u_d_i"), "judi");
        assert_eq!(normalize_text("S!l0t G@c0r"), "silot gacor");
    }

    #[test]
    fn test_parse_blacklist_input() {
        let result = parse_blacklist_input("slotgacor.com, judi online {tmute1d}");
        assert_eq!(
            result,
            ParsedBlacklistInput {
                words: vec!["slotgacor.com".to_string(), "judi online".to_string()],
                action: "tmute1d".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_duration_to_seconds() {
        assert_eq!(parse_duration_to_seconds("30m"), Some(1800));
        assert_eq!(parse_duration_to_seconds("2h"), Some(7200));
        assert_eq!(parse_duration_to_seconds("1d"), Some(86400));
    }

    #[test]
    fn test_parse_duration_with_validation() {
        assert!(parse_duration("30m").is_ok());
        assert!(parse_duration("10s").is_err()); // < 30 detik
        assert!(parse_duration("400d").is_err()); // > 365 hari
    }

    #[test]
    fn test_format_duration_human() {
        assert_eq!(format_duration_human(7200), "2 jam");
        assert_eq!(format_duration_human(86400), "1 hari");
    }
}
