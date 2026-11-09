use std::error::Error;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, ReplyParameters, UserId};

/// Fungsi pembantu untuk mencabut seluruh hak akses administrator
pub async fn revoke_all_admin_rights(
    bot: &Bot,
    chat_id: ChatId,
    user_id: UserId,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.promote_chat_member(chat_id, user_id)
        .is_anonymous(false)
        .can_manage_chat(false)
        .can_post_messages(false)
        .can_edit_messages(false)
        .can_delete_messages(false)
        .can_manage_video_chats(false)
        .can_restrict_members(false)
        .can_promote_members(false)
        .can_change_info(false)
        .can_invite_users(false)
        .can_pin_messages(false)
        .can_post_stories(false)
        .can_edit_stories(false)
        .can_delete_stories(false)
        .await?;

    Ok(())
}

/// Handler untuk perintah `/demote`
pub async fn handle_demote(bot: Bot, msg: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = msg.chat.id;

    // 1. Ekstrak target user (dari reply message atau argumen User ID)
    let target = if let Some(reply) = msg.reply_to_message() {
        reply.from.as_ref().map(|u| (u.id, u.full_name()))
    } else {
        let text = msg.text().unwrap_or_default();
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() > 1 {
            parts[1]
                .parse::<u64>()
                .ok()
                .map(|id| (UserId(id), format!("User {}", id)))
        } else {
            None
        }
    };

    let (target_id, target_name) = match target {
        Some(val) => val,
        None => {
            bot.send_message(
                chat_id,
                "❌ <b>Target tidak ditemukan!</b>\nBalas pesan user yang ingin didemote atau gunakan: <code>/demote <user_id></code>",
            )
            .parse_mode(ParseMode::Html)
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
            return Ok(());
        }
    };

    // 2. Eksekusi pencabutan jabatan
    match revoke_all_admin_rights(&bot, chat_id, target_id).await {
        Ok(_) => {
            let response = format!(
                "📉 <b>ADMIN DEMOTED</b>\n\n\
                👤 <b>Target:</b> <a href=\"tg://user?id={}\">{}</a> [<code>{}</code>]\n\
                🛡️ Seluruh hak akses administrator telah berhasil dicabut.",
                target_id, target_name, target_id
            );
            bot.send_message(chat_id, response)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ <b>Gagal demote user:</b> {}", e))
                .parse_mode(ParseMode::Html)
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
        }
    }

    Ok(())
}

/// Handler untuk perintah `/demoteme`
pub async fn handle_demoteme(bot: Bot, msg: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = msg.chat.id;

    let sender = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // Eksekusi demote untuk pengirim pesan itu sendiri
    match revoke_all_admin_rights(&bot, chat_id, sender.id).await {
        Ok(_) => {
            let response = format!(
                "🚶 <b>SELF DEMOTE SUCCESSFUL</b>\n\n\
                👤 <a href=\"tg://user?id={}\">{}</a> telah secara sukarela mengundurkan diri dari jabatan Administrator.",
                sender.id,
                sender.full_name()
            );
            bot.send_message(chat_id, response)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("❌ <b>Gagal mengundurkan diri:</b> {}", e))
                .parse_mode(ParseMode::Html)
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
        }
    }

    Ok(())
}
