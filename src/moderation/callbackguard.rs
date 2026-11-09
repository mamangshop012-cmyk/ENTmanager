use teloxide::{prelude::*, types::CallbackQuery};

/// 🛡️ SATPAM TERPUSAT (MULTI-OWNER & DEFAULT ADMIN-ONLY)
pub async fn guard_callback_query(
    bot: &Bot,
    query: &CallbackQuery,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let data = match &query.data {
        Some(d) => d.as_str(),
        None => return Ok(true),
    };

    let clicker_id = query.from.id;

    // ===================================================
    // 👑 0. HAK MUTLAK BOT OWNER (BISA BANYAK ID DI .env)
    // ===================================================
    let is_bot_owner = std::env::var("OWNER_ID")
        .unwrap_or_default()
        .split(',')
        .filter_map(|id| id.trim().parse::<u64>().ok())
        .any(|id| id == clicker_id.0);

    if is_bot_owner {
        return Ok(true);
    }

    // ===================================================
    // 🟢 1. TOMBOL KHUSUS MEMBER BIASA / TARGET USER
    // ===================================================
    if data.starts_with("appeal_") {
        let target_user_id: u64 = data
            .strip_prefix("appeal_")
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        if clicker_id.0 != target_user_id {
            bot.answer_callback_query(&query.id)
                .text("❌ Ini bukan tombol appeal milik kamu!")
                .show_alert(true)
                .await?;
            return Ok(false);
        }
        return Ok(true);
    }

    if data == "noop" {
        return Ok(true);
    }

    // ===================================================
    // ⚖️ 2. TOMBOL ACTION APPEAL DI LOG CHANNEL
    // ===================================================
    // Approval/rejection di Log Channel tidak perlu cek member grup di sini
    // karena akan di-handle langsung oleh handler
    if data.starts_with("approve_appeal_") || data.starts_with("reject_appeal_") {
        return Ok(true);
    }

    // Ambil info status member di grup
    let message = match &query.message {
        Some(m) => m,
        None => return Ok(true),
    };

    let chat_id = message.chat().id;
    let member = bot.get_chat_member(chat_id, clicker_id).await?;

    // ===================================================
    // ⚠️ 3. TOMBOL KHUSUS OWNER GRUP (SANGAT SENSITIF)
    // ===================================================
    let is_owner_only_button = data.contains("reset_all_warn");

    if is_owner_only_button {
        if !member.is_owner() {
            bot.answer_callback_query(&query.id)
                .text("🚫 Akses Ditolak: Tombol ini khusus untuk OWNER GRUP!")
                .show_alert(true)
                .await?;
            return Ok(false);
        }
        return Ok(true);
    }

    // ===================================================
    // 🛡️ 4. SISANYA KHUSUS UNTUK ADMIN GRUP
    // ===================================================
    let is_admin = member.is_administrator() || member.is_owner();

    if !is_admin {
        bot.answer_callback_query(&query.id)
            .text("❌ Akses Ditolak: Tombol ini khusus untuk Admin grup!")
            .show_alert(true)
            .await?;
        return Ok(false);
    }

    Ok(true)
}
