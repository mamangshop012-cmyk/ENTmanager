use super::frank_ui;
use teloxide::prelude::*;
use teloxide::types::{ParseMode, UserId}; // Import modul Frankenstein

// Constant Flags tetep sama seperti kode kamu
const PERM_DELETE: u16 = 1 << 0;
const PERM_BAN: u16 = 1 << 1;
const PERM_PIN: u16 = 1 << 2;
const PERM_INVITE: u16 = 1 << 3;
const PERM_VOICE: u16 = 1 << 4;
const PERM_MANAGE: u16 = 1 << 5;
const PERM_PROMOTE: u16 = 1 << 6;
const PERM_INFO: u16 = 1 << 7;
const PERM_STORIES: u16 = 1 << 8;

pub async fn handle_admin_approval_callback(
    bot: Bot,
    q: CallbackQuery,
    owner_ids: Vec<UserId>,
) -> ResponseResult<()> {
    if !owner_ids.contains(&q.from.id) {
        bot.answer_callback_query(&q.id)
            .text("⚠️ Akses Ditolak! Hanya Owner yang berhak mengelola permission.")
            .show_alert(true)
            .await?;
        return Ok(());
    }

    let data = match &q.data {
        Some(d) => d,
        None => return Ok(()),
    };

    let msg = match q.message.clone() {
        Some(m) => m,
        None => return Ok(()),
    };

    if data.starts_with("adm_app:") {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() < 3 {
            return Ok(());
        }

        let chat_id: i64 = parts[1].parse().unwrap_or(0);
        let target_id: u64 = parts[2].parse().unwrap_or(0);
        let initial_mask: u16 = PERM_DELETE | PERM_BAN;

        render_permission_panel(
            &bot,
            msg.chat().id,
            msg.id(),
            chat_id,
            target_id,
            initial_mask,
        )
        .await?;
        bot.answer_callback_query(&q.id).await?;
        return Ok(());
    }

    if data.starts_with("p_tgl:") {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() < 5 {
            return Ok(());
        }

        let chat_id: i64 = parts[1].parse().unwrap_or(0);
        let target_id: u64 = parts[2].parse().unwrap_or(0);
        let current_mask: u16 = parts[3].parse().unwrap_or(0);
        let flag: u16 = parts[4].parse().unwrap_or(0);

        let new_mask = current_mask ^ flag;

        render_permission_panel(&bot, msg.chat().id, msg.id(), chat_id, target_id, new_mask)
            .await?;
        bot.answer_callback_query(&q.id).await?;
        return Ok(());
    }

    if data.starts_with("p_app:") {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() < 4 {
            return Ok(());
        }

        let chat_id = ChatId(parts[1].parse().unwrap_or(0));
        let target_id = UserId(parts[2].parse().unwrap_or(0));
        let mask: u16 = parts[3].parse().unwrap_or(0);

        let res = bot
            .promote_chat_member(chat_id, target_id)
            .can_delete_messages((mask & PERM_DELETE) != 0)
            .can_restrict_members((mask & PERM_BAN) != 0)
            .can_pin_messages((mask & PERM_PIN) != 0)
            .can_invite_users((mask & PERM_INVITE) != 0)
            .can_manage_video_chats((mask & PERM_VOICE) != 0)
            .can_manage_chat((mask & PERM_MANAGE) != 0)
            .can_promote_members((mask & PERM_PROMOTE) != 0)
            .can_change_info((mask & PERM_INFO) != 0)
            .can_post_stories((mask & PERM_STORIES) != 0)
            .can_edit_stories((mask & PERM_STORIES) != 0)
            .can_delete_stories((mask & PERM_STORIES) != 0)
            .is_anonymous(false)
            .await;

        match res {
            Ok(_) => {
                let success_text = format!(
                    "<b>✅ PROMOTION GRANTED</b>\n\
                    <blockquote>\n\
                    ╔══ EXECUTION SUCCESS ═════\n\
                    ║🏠 Group  : <code>{}</code>\n\
                    ║👤 Target : <code>{}</code>\n\
                    ║🛡️ Exec   : <b>{}</b>\n\
                    ╚══════════════════════════\n\
                    </blockquote>",
                    chat_id.0, target_id.0, q.from.first_name
                );

                bot.edit_message_text(msg.chat().id, msg.id(), success_text)
                    .parse_mode(ParseMode::Html)
                    .await?;

                let group_msg = format!(
                    "<b>🎉 ADMIN PROMOTED</b>\n\
                    <blockquote>\n\
                    Target ID <code>{}</code> telah diangkat menjadi Admin dengan hak akses kustom!\n\
                    </blockquote>",
                    target_id.0
                );
                let _ = bot
                    .send_message(chat_id, group_msg)
                    .parse_mode(ParseMode::Html)
                    .await;
            }
            Err(e) => {
                bot.answer_callback_query(&q.id)
                    .text(format!("❌ Promotion Failed: {}", e))
                    .show_alert(true)
                    .await?;
            }
        }
        return Ok(());
    }

    if data.starts_with("adm_rej:") || data == "p_cls" {
        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            "<b>❌ PROPOSAL REJECTED / CLOSED</b>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        bot.answer_callback_query(&q.id).await?;
        return Ok(());
    }

    Ok(())
}

// Fungsi ini sekarang murni memanggil renderer Frankenstein
async fn render_permission_panel(
    bot: &Bot,
    log_chat_id: ChatId,
    msg_id: teloxide::types::MessageId,
    target_chat_id: i64,
    target_id: u64,
    mask: u16,
) -> ResponseResult<()> {
    if let Err(e) = frank_ui::render_colored_permission_panel(
        bot,
        log_chat_id.0,
        msg_id.0,
        target_chat_id,
        target_id,
        mask,
    )
    .await
    {
        eprintln!("❌ Failed rendering Frankenstein UI: {}", e);
    }

    Ok(())
}
