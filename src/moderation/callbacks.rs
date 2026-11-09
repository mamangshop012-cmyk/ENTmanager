use sqlx::SqlitePool;

use crate::moderation::actions::modstatus::{self, DashboardPage};
use crate::moderation::callbackguard::guard_callback_query;
use crate::moderation::db as mod_db;
use crate::terminal::{DashboardSenderExt, Feature}; // 👈 Import helper TUI Dashboard

use teloxide::{
    prelude::*,
    types::{
        CallbackQuery, ChatId, ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup,
        ParseMode, UserId,
    },
};

/// ==========================================================
/// MAIN MODERATION CALLBACK HANDLER
/// ==========================================================
///
/// Semua callback moderation diproses di SATU tempat.
///
/// Read-only:
/// - modstatus:*
/// - modstatus:noop
///
/// Protected / mutating:
/// - appeal_*
/// - dismiss_warn_*
/// - approve_appeal_*
/// - reject_appeal_*
/// - confirm_reset_all_warn
/// - cancel_reset_all_warn
///
pub async fn handle_moderation_callback(
    bot: Bot,
    q: CallbackQuery,
    pool: SqlitePool,
    dashboard_tx: crate::terminal::DashboardSender,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // ======================================================
    // DASHBOARD LOG HELPER
    // ======================================================
    let dashboard_log = |kind: crate::terminal::LogKind, message: String| {
        let tx = dashboard_tx.clone();
        async move {
            let _ = tx
                .send(crate::terminal::DashboardEvent::Log { kind, message })
                .await;
        }
    };

    // ======================================================
    // CALLBACK MASUK
    // ======================================================

    dashboard_log(
        crate::terminal::LogKind::Callback,
        format!(
            "[CALLBACK MASUK] id={} data={:?} from={} ({})",
            q.id, q.data, q.from.first_name, q.from.id
        ),
    )
    .await;

    // ======================================================
    // CALLBACK DATA
    // ======================================================

    let data = match q.data.as_deref() {
        Some(data) => data,

        None => {
            dashboard_log(
                crate::terminal::LogKind::Callback,
                "[CALLBACK] Tidak ada callback_data".to_string(),
            )
            .await;

            let _ = bot.answer_callback_query(q.id).await;

            return Ok(());
        }
    };

    // 📊 TUI Dashboard: Nyalakan kartu CALLBACK
    dashboard_tx.send_start(Feature::Callback, format!("Data: {}", data));

    dashboard_log(
        crate::terminal::LogKind::Callback,
        format!("[CALLBACK DATA] {}", data),
    )
    .await;

    // ======================================================
    // MESSAGE
    // ======================================================

    let msg = match q.message.as_ref() {
        Some(message) => message,

        None => {
            dashboard_log(
                crate::terminal::LogKind::Callback,
                "[CALLBACK] Callback tidak mempunyai message".to_string(),
            )
            .await;

            let _ = bot.answer_callback_query(q.id).await;

            return Ok(());
        }
    };

    let chat_id = msg.chat().id.0;
    let message_id = msg.id().0;
    let clicker_id = q.from.id.0 as i64;

    dashboard_log(
        crate::terminal::LogKind::Callback,
        format!(
            "[CALLBACK CONTEXT] chat={} message={} user={}",
            chat_id, message_id, clicker_id
        ),
    )
    .await;

    // ==========================================================
    // 1. MODSTATUS
    // ==========================================================

    if data.starts_with("modstatus:") {
        // 📊 TUI Dashboard: Nyalakan kartu MODSTATUS
        dashboard_tx.send_start(Feature::Modstatus, format!("Callback: {}", data));

        if data == "modstatus:noop" {
            dashboard_log(
                crate::terminal::LogKind::Callback,
                "[MODSTATUS] NOOP callback".to_string(),
            )
            .await;

            let _ = bot.answer_callback_query(q.id).await;
            dashboard_tx.send_success(Feature::Modstatus, Some("NOOP"));

            return Ok(());
        }

        let page_number = data
            .strip_prefix("modstatus:")
            .unwrap_or("1")
            .parse::<i32>()
            .unwrap_or(1)
            .clamp(1, 3);

        let target_page = DashboardPage::from_number(page_number);

        dashboard_log(
            crate::terminal::LogKind::Callback,
            format!(
                "[MODSTATUS CALLBACK] requested={} target={}",
                page_number,
                target_page.number()
            ),
        )
        .await;

        if let Err(err) = bot.answer_callback_query(q.id.clone()).await {
            dashboard_log(
                crate::terminal::LogKind::Error,
                format!("[MODSTATUS ACK ERROR] {:?}", err),
            )
            .await;
        }

        match modstatus::edit_modstatus_page(&bot, &pool, chat_id, message_id, target_page).await {
            Ok(_) => {
                dashboard_log(
                    crate::terminal::LogKind::Callback,
                    format!(
                        "[MODSTATUS] Berhasil pindah ke page {}",
                        target_page.number()
                    ),
                )
                .await;

                // 📊 TUI Dashboard: Kirim status sukses berpindah page
                dashboard_tx.send_success(
                    Feature::Modstatus,
                    Some(format!("Page {}", target_page.number())),
                );
            }

            Err(err) => {
                dashboard_log(
                    crate::terminal::LogKind::Error,
                    format!("[MODSTATUS EDIT ERROR] {:?}", err),
                )
                .await;

                // 📊 TUI Dashboard: Kirim error jika gagal edit
                dashboard_tx.send_error(Feature::Modstatus, err.to_string());
            }
        }

        dashboard_tx.send_success(Feature::Callback, Some("Modstatus handled"));
        return Ok(());
    }

    // ==========================================================
    // 2. CALLBACK GUARD
    // ==========================================================

    if !guard_callback_query(&bot, &q).await? {
        dashboard_log(
            crate::terminal::LogKind::Error,
            "[CALLBACK DITOLAK] guard_callback_query".to_string(),
        )
        .await;

        dashboard_tx.send_error(Feature::Callback, "Guard rejected");
        return Ok(());
    }

    // ==========================================================
    // 3. APPEAL
    // ==========================================================

    if data.starts_with("appeal_") {
        dashboard_log(
            crate::terminal::LogKind::Callback,
            format!("[APPEAL] User {} mengajukan appeal", clicker_id),
        )
        .await;

        if mod_db::has_pending_appeal(&pool, chat_id, clicker_id).await? {
            let _ = bot
                .answer_callback_query(q.id)
                .text("⚠️ Banding Anda sedang ditinjau oleh Admin. Mohon tunggu.")
                .show_alert(true)
                .await;

            dashboard_tx.send_success(Feature::Callback, Some("Appeal pending alert"));
            return Ok(());
        }

        let appeal_id =
            mod_db::create_appeal(&pool, chat_id, clicker_id, message_id as i64).await?;

        let settings = mod_db::get_or_create_warn_settings(&pool, chat_id).await?;

        if let Some(log_channel_id) = settings.log_channel_id {
            let clicker_name = &q.from.first_name;

            let appeal_card = format!(
                "⚖️ <b>PENGAJUAN BANDING \
                 (APPEAL #{appeal_id})</b>\n\n\
                 • <b>Grup ID:</b> <code>{chat_id}</code>\n\
                 • <b>User:</b> \
                 <a href=\"tg://user?id={clicker_id}\">\
                 {clicker_name}</a> \
                 (<code>{clicker_id}</code>)\n\
                 • <b>Status:</b> \
                 ⏳ Menunggu Keputusan Admin"
            );

            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::callback(
                    "✅ Setujui",
                    format!("approve_appeal_{}_{}", appeal_id, clicker_id),
                ),
                InlineKeyboardButton::callback("❌ Tolak", format!("reject_appeal_{}", appeal_id)),
            ]]);

            match bot
                .send_message(ChatId(log_channel_id), appeal_card)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .await
            {
                Ok(sent_msg) => {
                    let _ =
                        mod_db::update_appeal_log_msg(&pool, appeal_id, sent_msg.id.0 as i64).await;
                }

                Err(err) => {
                    dashboard_log(
                        crate::terminal::LogKind::Error,
                        format!("[APPEAL LOG ERROR] {:?}", err),
                    )
                    .await;
                }
            }
        }

        let _ = bot
            .answer_callback_query(q.id)
            .text(
                "✅ Pengajuan banding Anda \
                 telah dikirimkan ke Admin!",
            )
            .show_alert(true)
            .await;

        dashboard_tx.send_success(Feature::Callback, Some("Appeal created"));
        return Ok(());
    }

    // ==========================================================
    // 4. DISMISS WARN
    // ==========================================================

    if data.starts_with("dismiss_warn_") {
        let target_user_id: i64 = data
            .strip_prefix("dismiss_warn_")
            .unwrap_or("0")
            .split(':')
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        // 📊 TUI Dashboard: Nyalakan indikator WARN
        dashboard_tx.send_start(Feature::Warn, format!("Dismiss warn: {}", target_user_id));

        let new_warn = mod_db::decrement_user_warn(&pool, chat_id, target_user_id).await?;

        let admin_name = &q.from.first_name;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            format!(
                "✅ <b>Peringatan Dibatalkan!</b>\n\
                 Admin <b>{admin_name}</b> telah \
                 menghapus peringatan untuk user \
                 <code>{target_user_id}</code>.\n\
                 Sisa Warn: <code>{new_warn}</code>"
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;

        let _ = bot
            .answer_callback_query(q.id)
            .text("✅ Peringatan berhasil dihapus!")
            .await;

        dashboard_tx.send_success(Feature::Warn, Some("Warn dismissed"));
        dashboard_tx.send_success(Feature::Callback, Some("Dismiss handled"));

        return Ok(());
    }

    // ==========================================================
    // 5. APPROVE APPEAL
    // ==========================================================

    if data.starts_with("approve_appeal_") {
        let clean_data = data.strip_prefix("approve_appeal_").unwrap_or("");

        let parts: Vec<&str> = clean_data.split('_').collect();

        if parts.len() >= 2 {
            let appeal_id: i64 = parts[0].parse().unwrap_or(0);

            let appeal_info: Option<(i64, i64)> = sqlx::query_as(
                "SELECT chat_id, user_id \
                     FROM warn_appeals \
                     WHERE id = ?",
            )
            .bind(appeal_id)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

            if let Some((group_chat_id, user_id)) = appeal_info {
                let target_id = UserId(user_id as u64);

                let admin_name = &q.from.first_name;

                let member = bot.get_chat_member(ChatId(group_chat_id), target_id).await;

                let is_restricted = match &member {
                    Ok(member) => member.is_restricted(),

                    Err(_) => false,
                };

                let action_text: String;

                if is_restricted {
                    let _ = bot
                        .restrict_chat_member(
                            ChatId(group_chat_id),
                            target_id,
                            ChatPermissions::all(),
                        )
                        .await;

                    action_text = "Pembungkaman (Mute) \
                         telah dicabut."
                        .to_string();
                } else {
                    let new_warn = mod_db::decrement_user_warn(&pool, group_chat_id, user_id)
                        .await
                        .unwrap_or(0);

                    action_text = format!(
                        "1 Poin Warn dihapus \
                             (Sisa Warn: \
                             <code>{new_warn}</code>)."
                    );
                }

                mod_db::update_appeal_status(&pool, appeal_id, "approved").await?;

                bot.edit_message_text(
                    msg.chat().id,
                    msg.id(),
                    format!(
                        "✅ <b>Appeal #{appeal_id} \
                         DISETUJUI oleh \
                         {admin_name}.</b>\n\
                         • Status: {action_text}"
                    ),
                )
                .parse_mode(ParseMode::Html)
                .await?;

                let user_link = format!(
                    "<a href=\"tg://user?id={}\">\
                         User</a>",
                    user_id
                );

                let _ = bot
                    .send_message(
                        ChatId(group_chat_id),
                        format!(
                            "⚖️ <b>Banding Disetujui!</b>\n\
                             Banding untuk {} telah \
                             disetujui Admin \
                             {admin_name}.\n\
                             • Status: {action_text}",
                            user_link
                        ),
                    )
                    .parse_mode(ParseMode::Html)
                    .await;
            }
        }

        let _ = bot.answer_callback_query(q.id).await;

        dashboard_tx.send_success(Feature::Callback, Some("Appeal approved"));
        return Ok(());
    }

    // ==========================================================
    // 6. REJECT APPEAL
    // ==========================================================

    if data.starts_with("reject_appeal_") {
        let appeal_id: i64 = data
            .strip_prefix("reject_appeal_")
            .unwrap_or("0")
            .split('_')
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        mod_db::update_appeal_status(&pool, appeal_id, "rejected").await?;

        bot.edit_message_text(
            msg.chat().id,
            msg.id(),
            format!(
                "❌ <b>Appeal #{appeal_id} \
                 DITOLAK.</b>"
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;

        let _ = bot.answer_callback_query(q.id).await;

        dashboard_tx.send_success(Feature::Callback, Some("Appeal rejected"));
        return Ok(());
    }

    // ==========================================================
    // 7. RESET ALL WARNS
    // ==========================================================

    if data.starts_with("confirm_reset_all_warn") {
        // 📊 TUI Dashboard: Nyalakan indikator RESETALLWARN
        dashboard_tx.send_start(Feature::Resetallwarn, format!("Confirming chat {}", chat_id));

        let _ = bot.answer_callback_query(q.id.clone()).await;

        match mod_db::reset_all_warns(&pool, chat_id).await {
            Ok(_) => {
                let _ = bot
                    .edit_message_text(
                        msg.chat().id,
                        msg.id(),
                        "✅ <b>Berhasil!</b> \
                         Seluruh warn member \
                         di grup ini telah di-reset.",
                    )
                    .parse_mode(ParseMode::Html)
                    .await;

                dashboard_tx.send_success(Feature::Resetallwarn, Some("Reset all warns completed"));
            }

            Err(err) => {
                dashboard_log(
                    crate::terminal::LogKind::Error,
                    format!("[ERROR RESET ALL WARNS] {:?}", err),
                )
                .await;

                dashboard_tx.send_error(Feature::Resetallwarn, err.to_string());
            }
        }

        dashboard_tx.send_success(Feature::Callback, Some("Reset all handled"));
        return Ok(());
    }

    // ==========================================================
    // 8. CANCEL RESET ALL WARNS
    // ==========================================================

    if data.starts_with("cancel_reset_all_warn") {
        let _ = bot.answer_callback_query(q.id.clone()).await;

        let _ = bot
            .edit_message_text(msg.chat().id, msg.id(), "❌ Proses reset warn dibatalkan.")
            .parse_mode(ParseMode::Html)
            .await;

        dashboard_tx.send_success(Feature::Resetallwarn, Some("Cancelled"));
        dashboard_tx.send_success(Feature::Callback, Some("Cancel reset handled"));
        return Ok(());
    }

    // ==========================================================
    // UNKNOWN CALLBACK
    // ==========================================================

    dashboard_log(
        crate::terminal::LogKind::Debug,
        format!("[CALLBACK UNKNOWN] {}", data),
    )
    .await;

    let _ = bot.answer_callback_query(q.id).await;
    dashboard_tx.send_success(Feature::Callback, Some("Unknown callback handled"));

    Ok(())
}
