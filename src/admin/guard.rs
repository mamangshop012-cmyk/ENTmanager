//// ========================================================================
//// 📦 IMPORTS
//// ========================================================================
use teloxide::prelude::*;
use teloxide::types::{ChatId, ChatMemberKind, UserId};

pub struct AdminGuard;

impl AdminGuard {
    //// ========================================================================
    //// 🛡️ LAYER 2 & 3: VERIFIKASI HAK AKSES EXECUTER (WITH MULTI-OWNER BYPASS)
    //// ========================================================================
    pub async fn verify_promote_rights(
        bot: &Bot,
        chat_id: ChatId,
        user_id: UserId,
        owner_ids: &[UserId], // 💡 Ubah ke &[UserId] agar bisa menampung banyak Owner
    ) -> bool {
        // ⚡ Layer 2: Owner Absolute Bypass (Cek apakah ID user ada di daftar owner_ids)
        if owner_ids.contains(&user_id) {
            return true;
        }

        // Layer 3: Cek apakah user adalah Admin/Creator di grup Telegram
        match bot.get_chat_member(chat_id, user_id).await {
            Ok(member) => member.is_administrator() || member.is_owner(),
            Err(_) => false,
        }
    }

    //// ========================================================================
    //// 🛡️ LAYER 4C: VERIFIKASI WEWENANG BOT SENDIRI DI GRUP
    //// ========================================================================
    pub async fn verify_bot_can_promote(bot: &Bot, chat_id: ChatId) -> bool {
        if let Ok(me) = bot.get_me().await {
            if let Ok(member) = bot.get_chat_member(chat_id, me.id).await {
                if let ChatMemberKind::Administrator(admin) = &member.kind {
                    return admin.can_promote_members;
                }
            }
        }
        false
    }
}
