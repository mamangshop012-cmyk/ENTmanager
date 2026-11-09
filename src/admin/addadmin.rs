//// ========================================================================
//// 📦 IMPORTS
//// ========================================================================
use crate::admin::guard::AdminGuard;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode, UserId};

//// ========================================================================
//// 🛡️ HANDLER: ADD ADMIN (WITH GOD MODE SELF-PROMOTION & MULTI-OWNER)
//// ========================================================================
pub async fn handle_add_admin(
    bot: Bot,
    msg: Message,
    owner_ids: Vec<UserId>, // 💡 Support Multiple Owner IDs
    log_channel_id: Option<ChatId>,
) -> ResponseResult<()> {
    // 1. Ambil data pengirim
    let sender = match msg.from.as_ref() {
        Some(user) => user,
        None => return Ok(()),
    };
    let chat_id = msg.chat.id;

    // Cek apakah pengirim adalah salah satu Owner terdaftar
    let is_owner = owner_ids.contains(&sender.id);

    // 2. LAYER 2 & 3: Verifikasi Hak Akses (Owner Absolute Bypass)
    let is_authorized =
        AdminGuard::verify_promote_rights(&bot, chat_id, sender.id, &owner_ids).await;
    if !is_authorized {
        return Ok(());
    }

    // 3. LAYER 4C: Cek Wewenang Bot Sendiri di Grup
    if !AdminGuard::verify_bot_can_promote(&bot, chat_id).await {
        let err_msg = "<b>⚠️ PROMOTION ERROR</b>\n\
                       <blockquote>\n\
                       ╔══ STATUS : REJECTED ═════\n\
                       ║❌ Reason : Bot Lacks     \n\
                       ║   'can_promote_members' \n\
                       ║📌 Promote bot to Admin! \n\
                       ╚══════════════════════════\n\
                       </blockquote>";
        bot.send_message(chat_id, err_msg)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    // 4. Tentukan Target Promotion (Smart Target Resolver)
    let target_user = match msg.reply_to_message() {
        Some(reply) => match reply.from.as_ref() {
            Some(user) => user,
            None => return Ok(()),
        },
        None => {
            // ⚡ GOD MODE: Jika BUKAN reply & Pengirim adalah salah satu Owner -> Self Target
            if is_owner {
                sender
            } else {
                let err_msg = "<b>⚠️ PROMOTION ERROR</b>\n\
                               <blockquote>\n\
                               ╔══ STATUS : REJECTED ═════\n\
                               ║❌ Reason : Must reply    \n\
                               ║   to target message!     \n\
                               ╚══════════════════════════\n\
                               </blockquote>";
                bot.send_message(chat_id, err_msg)
                    .parse_mode(ParseMode::Html)
                    .await?;
                return Ok(());
            }
        }
    };

    // 5. Validasi Target
    if target_user.is_bot {
        let err_msg = "<b>⚠️ PROMOTION ERROR</b>\n\
                       <blockquote>\n\
                       ╔══ STATUS : REJECTED ═════\n\
                       ║❌ Reason : Target is Bot \n\
                       ╚══════════════════════════\n\
                       </blockquote>";
        bot.send_message(chat_id, err_msg)
            .parse_mode(ParseMode::Html)
            .await?;
        return Ok(());
    }

    // Blokir Admin biasa yang mencoba self-promote (Owner dikecualikan)
    if target_user.id == sender.id && !is_owner {
        return Ok(());
    }

    // ⚡ 6. SPECIAL CASE: OWNER SELF-PROMOTION (GOD MODE INSTANT EXECUTION)
    if is_owner && target_user.id == sender.id {
        // Hak Akses Murni Supergroup (8 Akses Utama Admin)
        let promote_res = bot
            .promote_chat_member(chat_id, sender.id)
            .can_change_info(true)
            .can_delete_messages(true)
            .can_invite_users(true)
            .can_restrict_members(true)
            .can_pin_messages(true)
            .can_promote_members(true)
            .can_manage_video_chats(true)
            .can_manage_chat(true)
            .is_anonymous(false)
            .await;

        match promote_res {
            Ok(_) => {
                let success_msg = format!(
                    "<b>⚡ OWNER GOD MODE ACTIVATED</b>\n\
                        <blockquote>\n\
                        ╔══ PROMOTION EFFECTIVE ═══\n\
                        ║👑 Status : Self-Promoted \n\
                        ║👤 Owner  : <b>{}</b>\n\
                        ║🛡️ Access : Full Access Granted\n\
                        ╚══════════════════════════\n\
                        </blockquote>",
                    sender.first_name
                );
                bot.send_message(chat_id, success_msg)
                    .parse_mode(ParseMode::Html)
                    .await?;

                if let Some(log_channel) = log_channel_id {
                    let audit_msg = format!(
                        "<b>⚡ AUDIT LOG: OWNER SELF-PROMOTION</b>\n\
                            <blockquote>\n\
                            ╔══ DETAILS ═══════════════\n\
                            ║🏠 Group : <code>{}</code>\n\
                            ║👑 Owner : <b>{}</b> (<code>{}</code>)\n\
                            ║📌 Action: Self-Promoted via God Mode\n\
                            ╚══════════════════════════\n\
                            </blockquote>",
                        chat_id.0, sender.first_name, sender.id
                    );
                    let _ = bot
                        .send_message(log_channel, audit_msg)
                        .parse_mode(ParseMode::Html)
                        .await;
                }
            }
            Err(e) => {
                let err_msg = format!(
                    "<b>⚠️ GOD MODE FAILED</b>\n\
                        <blockquote>\n\
                        ❌ Telegram Error: {}\n\
                        </blockquote>",
                    e
                );
                bot.send_message(chat_id, err_msg)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }
        return Ok(());
    }

    // 7. FLOW NORMAL: Jika Owner/Admin mempromosikan USER LAIN (Proposal Log Channel)
    let target_log_channel = match log_channel_id {
        Some(id) => id,
        None => {
            let err_msg = "<b>⚠️ PROMOTION ERROR</b>\n\
                           <blockquote>\n\
                           ╔══ STATUS : REJECTED ═════\n\
                           ║❌ Reason : Log Channel   \n\
                           ║   is not set yet!        \n\
                           ╠══ ACTION REQUIRED ═══════\n\
                           ║📌 Set via /setlog first  \n\
                           ╚══════════════════════════\n\
                           </blockquote>";
            bot.send_message(chat_id, err_msg)
                .parse_mode(ParseMode::Html)
                .await?;
            return Ok(());
        }
    };

    // Notifikasi Pending di Grup
    let group_pending_msg = format!(
        "<b>⏳ PROMOTION PENDING</b>\n\
        <blockquote>\n\
        ╔══ REQUEST SUBMITTED ═════\n\
        ║👤 Target : <b>{}</b>\n\
        ║🆔 ID     : <code>{}</code>\n\
        ║🛡️ Exec   : <b>{}</b>\n\
        ║📌 Status : Waiting Approval\n\
        ╚══════════════════════════\n\
        </blockquote>\n\
        <i>📩 Permintaan dikirim ke Log Channel.</i>",
        target_user.first_name, target_user.id, sender.first_name
    );

    bot.send_message(chat_id, group_pending_msg)
        .parse_mode(ParseMode::Html)
        .await?;

    // Keyboard Proposal
    let approve_data = format!("adm_app:{}:{}", chat_id.0, target_user.id.0);
    let reject_data = format!("adm_rej:{}:{}", chat_id.0, target_user.id.0);

    let keyboard = InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("✅ Setujui", approve_data),
        InlineKeyboardButton::callback("❌ Tolak", reject_data),
    ]]);

    // Kirim Log Proposal
    let log_caption = format!(
        "<b>📥 ADMIN PROPOSAL REQUEST</b>\n\
        <blockquote>\n\
        ╔══ REQUEST DETAILS ═══════\n\
        ║🏠 Group  : <code>{}</code>\n\
        ║👤 Target : <b>{}</b> (<code>{}</code>)\n\
        ║🛡️ Exec   : <b>{}</b> (<code>{}</code>)\n\
        ╠══ ACCESS TO GRANT ═══════\n\
        ║🔥 <b>FULL ACCESS (15/15)</b>\n\
        ╚══════════════════════════\n\
        </blockquote>",
        chat_id.0, target_user.first_name, target_user.id, sender.first_name, sender.id
    );

    bot.send_message(target_log_channel, log_caption)
        .parse_mode(ParseMode::Html)
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
