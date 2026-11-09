use crate::moderation::{
    db as mod_db,
    listeners::{execute_direct_action, send_action_notification},
    utils::{is_user_immune, parse_blacklist_input},
};
use serde_json::json;
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{ChatId, Message, MessageId, ParseMode},
};

// 🎨 FUNGSI UI TERPUSAT UNTUK SEMUA ALERT WARN (MANUAL & AUTOMATIC)
pub fn build_warn_alert_text(
    alert_title: &str,
    target_id: i64,
    target_name: &str,
    issuer_name: &str,
    extra_field: Option<(&str, &str)>,
    current_warn: i32,
    max_warn: i32,
) -> String {
    let filled_count = current_warn.clamp(0, max_warn) as usize;
    let empty_count = (max_warn - current_warn).max(0) as usize;
    let filled = "■".repeat(filled_count);
    let empty = "□".repeat(empty_count);
    let progress_bar = format!("{filled}{empty} {current_warn}/{max_warn}");

    let extra_line = match extra_field {
        Some((label, val)) => format!("├ <b>{label}:</b> <code>{val}</code>\n"),
        None => String::new(),
    };

    format!(
        "<code>[𝐒𝐘𝐒𝐓𝐄𝐌 𝐀𝐋𝐄𝐑𝐓: {alert_title}]</code>\n\
         <pre>┌ <b>Target:</b> <a href=\"tg://user?id={target_id}\">{target_name}</a>\n\
         ├ <b>ID:</b> <code>{target_id}</code>\n\
         ├ <b>By Admin:</b> <b>{issuer_name}</b>\n\
         {extra_line}\
         └ <b>Poin:</b> <code>{progress_bar}</code></pre>"
    )
}

pub async fn cmd_resetallwarn(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let keyboard_json = serde_json::json!({
        "inline_keyboard": [[
            { "text": "⚠️ Ya, Reset Semua Warn", "callback_data": "confirm_reset_all_warn", "style": "danger" },
            { "text": "❌ Batal", "callback_data": "cancel_reset_all_warn" }
        ]]
    });
    let keyboard: teloxide::types::InlineKeyboardMarkup = serde_json::from_value(keyboard_json)?;

    bot.send_message(
        msg.chat.id,
        "⚠️ <b>KONFIRMASI:</b> Apakah Anda yakin ingin menghapus seluruh riwayat warn semua member di grup ini?",
    )
    .parse_mode(ParseMode::Html)
    .reply_markup(keyboard)
    .await?;

    Ok(())
}

pub async fn cmd_setwarn(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/setwarn").unwrap_or("").trim();

    let limit: i32 = match args.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            bot.send_message(
                msg.chat.id,
                "⚠️ <b>Format Salah!</b>\nGunakan: <code>/setwarn &lt;jumlah&gt;</code>\nContoh: <code>/setwarn 3</code>",
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    mod_db::set_warn_limit(&pool, chat_id, limit).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Batas maksimum warning berhasil diubah menjadi <b>{}</b>.",
            limit
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

pub async fn cmd_warnmode(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text
        .strip_prefix("/warnmode")
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let valid_actions = ["ban", "mute", "kick", "tmute"];

    if !valid_actions.contains(&args.as_str()) {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Format Salah!</b>\nPilihan mode yang valid: <code>ban</code>, <code>mute</code>, <code>kick</code>, <code>tmute</code>\nContoh: <code>/warnmode mute</code>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    mod_db::set_warn_action(&pool, chat_id, &args).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Mode hukuman warning berhasil diubah menjadi <b>{}</b>.",
            args.to_uppercase()
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

pub async fn cmd_warn(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
    dashboard_tx: crate::terminal::DashboardSender,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let clicker_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    // 🛡️ ISSUER CHECK: Pastikan pengirim perintah adalah Admin Grup atau Bot Owner
    if !is_user_immune(&bot, msg.chat.id, clicker_id).await {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya bisa digunakan oleh Admin grup atau Bot Owner!",
        )
        .await?;
        return Ok(());
    }

    let target_user = match msg.reply_to_message().and_then(|m| m.from.as_ref()) {
        Some(u) => u,
        None => {
            bot.send_message(
                msg.chat.id,
                "⚠️ <b>Penggunaan:</b> Balas (reply) pesan user yang ingin diberi warn, lalu ketik <code>/warn</code>",
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    if target_user.is_bot {
        bot.send_message(msg.chat.id, "⚠️ Bot tidak bisa diberi warning.")
            .await?;
        return Ok(());
    }

    // 🛡️ TARGET IMMUNITY CHECK
    if is_user_immune(&bot, msg.chat.id, target_user.id).await {
        bot.send_message(
            msg.chat.id,
            "🛡️<code>[SYSTEM⨻]</code> User memiliki hak kekebalan.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    // ================================================================
    // 📊 TERMINAL DASHBOARD — WARN START
    // ================================================================
    let dashboard_detail = format!(
        "target: @{}",
        target_user
            .username
            .as_deref()
            .unwrap_or(&target_user.first_name)
    );

    // Dashboard bersifat observer. Jika channel penuh/ditutup,
    // logic warn tetap berjalan.
    let _ = dashboard_tx
        .send(crate::terminal::DashboardEvent::Started {
            feature: crate::terminal::Feature::Warn,
            detail: dashboard_detail,
        })
        .await;

    // ================================================================
    // ⚙️ WARN LOGIC ASLI
    // ================================================================
    let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = async {
        let target_user_id = target_user.id.0 as i64;

        let warn_count = mod_db::increment_user_warn(&pool, chat_id, target_user_id).await?;

        let settings = mod_db::get_or_create_warn_settings(&pool, chat_id).await?;

        if warn_count >= settings.warn_limit {
            mod_db::reset_user_warns(&pool, chat_id, target_user_id).await?;

            execute_direct_action(
                bot,
                pool,
                chat_id,
                target_user,
                "Manual Warn Limit",
                &settings.warn_action,
            )
            .await?;
        } else {
            let admin_name = msg
                .from
                .as_ref()
                .map(|u| u.first_name.as_str())
                .unwrap_or("Admin");

            let text = build_warn_alert_text(
                "MANUAL WARN ISSUED",
                target_user_id,
                &target_user.first_name,
                admin_name,
                None,
                warn_count,
                settings.warn_limit,
            );

            send_action_notification(
                &bot,
                &pool,
                chat_id,
                target_user,
                &format!("Manual Warning [{warn_count}/{}]", settings.warn_limit),
                "Manual Warn",
                &text,
                true,
                true,
            )
            .await?;
        }

        Ok(())
    }
    .await;

    // ================================================================
    // 📊 TERMINAL DASHBOARD — WARN RESULT
    // ================================================================
    match result {
        Ok(()) => {
            let _ = dashboard_tx
                .send(crate::terminal::DashboardEvent::Success {
                    feature: crate::terminal::Feature::Warn,
                    detail: Some("warning processed successfully".to_string()),
                })
                .await;

            Ok(())
        }

        Err(error) => {
            let error_message = error.to_string();

            let _ = dashboard_tx
                .send(crate::terminal::DashboardEvent::Error {
                    feature: crate::terminal::Feature::Warn,
                    message: error_message,
                })
                .await;

            Err(error)
        }
    }
}

pub async fn cmd_cleanwarn(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/cleanwarn").unwrap_or("").trim();

    if args.is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Penggunaan:</b> <code>/cleanwarn &lt;durasi&gt;</code>\n\
             Contoh: <code>/cleanwarn 1d</code> (1 hari), <code>/cleanwarn 2w</code> (2 minggu), <code>/cleanwarn 1m</code> (1 bulan).\n\
             Atau <code>/cleanwarn off</code> untuk mematikan auto-reset.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    if args.eq_ignore_ascii_case("off") {
        mod_db::set_clean_interval(&pool, chat_id, None).await?;
        bot.send_message(msg.chat.id, "✅ Fitur auto-clean warn telah dinonaktifkan.")
            .await?;
        return Ok(());
    }

    mod_db::set_clean_interval(&pool, chat_id, Some(args)).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Auto-clean warn berhasil diatur dengan interval <code>{}</code>.",
            args
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

pub async fn cmd_setlog(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/setlog").unwrap_or("").trim();

    if args.is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Penggunaan:</b> <code>/setlog -1001234567890</code>\nAtau <code>/setlog off</code> untuk mematikan log channel.",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    if args.eq_ignore_ascii_case("off") || args.eq_ignore_ascii_case("clear") {
        mod_db::set_log_channel(&pool, chat_id, None).await?;
        bot.send_message(msg.chat.id, "✅ Log channel telah dinonaktifkan.")
            .await?;
        return Ok(());
    }

    match args.parse::<i64>() {
        Ok(channel_id) => {
            mod_db::set_log_channel(&pool, chat_id, Some(channel_id)).await?;
            bot.send_message(
                msg.chat.id,
                format!(
                    "✅ Log channel berhasil diatur ke <code>{}</code>",
                    channel_id
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                "❌ ID Channel tidak valid! ID Channel diawali `-100` (contoh: <code>-1001234567890</code>).",
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }

    Ok(())
}

// ==========================================================
// 🎯 UPDATE FITUR HYBRID BLACKLIST (SLOT 0 VS SLOTS 1-5)
// ==========================================================

/// Handler `/setslot <1-5> <action>`
pub async fn cmd_setslot(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/setslot").unwrap_or("").trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Penggunaan:</b> <code>/setslot &lt;1-5&gt; &lt;action&gt;</code>\nContoh: <code>/setslot 1 tmute1d</code> atau <code>/setslot 2 ban</code>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let slot_id: i32 = match parts[0].parse() {
        Ok(n) if (1..=5).contains(&n) => n,
        _ => {
            bot.send_message(
                msg.chat.id,
                "❌ Nomor slot harus di antara <b>1 sampai 5</b>!",
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    let action = parts[1].to_lowercase();
    crate::db::set_slot_action(&pool, chat_id, slot_id, &action).await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Aksi hukuman massal untuk <b>Slot {}</b> berhasil diatur ke <code>{}</code>.",
            slot_id, action
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

/// Handler `/addbl1` sampai `/addbl5` (Bulk Slot 1..=5)
pub async fn cmd_addbl_slot(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
    slot_id: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let prefix = format!("/addbl{}", slot_id);
    let args = text.strip_prefix(&prefix).unwrap_or("").trim();

    if args.is_empty() {
        bot.send_message(
            msg.chat.id,
            format!(
                "⚠️ <b>Penggunaan:</b> <code>/addbl{} kata1, kata2</code>",
                slot_id
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let words: Vec<&str> = args
        .split(',')
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();
    let mut added_count = 0;

    for word in words {
        crate::db::add_blacklist_word(&pool, chat_id, word, "slot", slot_id).await?;
        added_count += 1;
    }

    let slot_action = crate::db::get_slot_action(&pool, chat_id, slot_id)
        .await?
        .unwrap_or_else(|| "tmute1d (default)".to_string());

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Berhasil menambahkan <b>{added_count}</b> kata ke <b>Slot {slot_id}</b> (Aksi: <code>{slot_action}</code>).",
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

/// Handler `/addbl kata1, kata2 {action}` (Slot 0 Custom Granular)
pub async fn cmd_addbl(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/addbl").unwrap_or("").trim();

    if args.is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Penggunaan:</b> <code>/addbl kata1, kata2 {tmute1d}</code>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let parsed = parse_blacklist_input(args);
    let mut added_count = 0;

    for word in &parsed.words {
        // Slot 0 = Custom Granular Action
        crate::db::add_blacklist_word(&pool, chat_id, word, &parsed.action, 0).await?;
        added_count += 1;
    }

    let response = format!(
        "✅ Berhasil menambahkan <b>{added_count}</b> kata ke blacklist Slot 0 (Custom) dengan aksi <code>{{{}}}</code>.",
        parsed.action
    );

    bot.send_message(msg.chat.id, response)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

/// Handler `/delbl kata1, kata2`
pub async fn cmd_delbl(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/delbl").unwrap_or("").trim();

    if args.is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Penggunaan:</b> <code>/delbl kata1, kata2</code>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let words: Vec<&str> = args
        .split(',')
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();
    let mut removed_count = 0;

    for word in words {
        let rows = crate::db::remove_blacklist_word(&pool, chat_id, word).await?;
        if rows > 0 {
            removed_count += 1;
        }
    }

    bot.send_message(
        msg.chat.id,
        format!("🗑️ Berhasil menghapus <b>{removed_count}</b> kata dari blacklist."),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

pub async fn cmd_blacklist(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = msg.from.as_ref().map(|u| u.id.0 as i64).unwrap_or(0);
    send_blacklist_page(&bot, &pool, msg.chat.id.0, msg.chat.id, 1, None, user_id).await
}

pub async fn send_blacklist_page(
    bot: &Bot,
    pool: &SqlitePool,
    chat_id: i64,
    target_chat: ChatId,
    page: usize,
    edit_msg_id: Option<MessageId>,
    receiver_user_id: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let items = crate::db::get_blacklist_entries(pool, chat_id).await?;

    if items.is_empty() {
        let text = "📋 Daftar blacklist grup ini masih kosong.";
        if let Some(msg_id) = edit_msg_id {
            let client = reqwest::Client::new();
            let url = format!(
                "https://api.telegram.org/bot{}/editMessageText",
                bot.token()
            );
            let body = json!({
                "chat_id": target_chat.0,
                "message_id": msg_id.0,
                "text": text,
                "receiver_user_id": receiver_user_id
            });
            let _ = client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .send()
                .await;
        } else {
            bot.send_message(target_chat, text).await?;
        }
        return Ok(());
    }

    const PAGE_SIZE: usize = 10;
    let total_pages = (items.len() + PAGE_SIZE - 1) / PAGE_SIZE;
    let current_page = page.clamp(1, total_pages);

    let start_idx = (current_page - 1) * PAGE_SIZE;
    let end_idx = (start_idx + PAGE_SIZE).min(items.len());
    let page_items = &items[start_idx..end_idx];

    let mut text =
        format!("📋 <b>Daftar Blacklist Hybrid (Halaman {current_page}/{total_pages}):</b>\n\n");
    for (i, item) in page_items.iter().enumerate() {
        let slot_label = if item.slot_id == 0 {
            format!("<i>{{{}}}</i> [Granular]", item.action)
        } else {
            format!("<b>[Slot {}]</b>", item.slot_id)
        };

        text.push_str(&format!(
            "{}. <code>{}</code> — {}\n",
            start_idx + i + 1,
            item.word,
            slot_label
        ));
    }

    let mut row1 = Vec::new();
    if current_page > 1 {
        row1.push(json!({
            "text": "⬅️ Prev",
            "callback_data": format!("blpage_{}", current_page - 1)
        }));
    }
    row1.push(json!({
        "text": format!("{current_page}/{total_pages}"),
        "callback_data": "noop"
    }));
    if current_page < total_pages {
        row1.push(json!({
            "text": "Next ➡️",
            "callback_data": format!("blpage_{}", current_page + 1)
        }));
    }

    let keyboard_json = json!({
        "inline_keyboard": [
            row1,
            [
                { "text": "🔙 Back to Dashboard", "callback_data": "back_to_modstatus" }
            ]
        ]
    });

    if let Some(msg_id) = edit_msg_id {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.telegram.org/bot{}/editMessageText",
            bot.token()
        );
        let body = json!({
            "chat_id": target_chat.0,
            "message_id": msg_id.0,
            "text": text,
            "parse_mode": "HTML",
            "reply_markup": keyboard_json,
            "receiver_user_id": receiver_user_id
        });

        let res = client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await;

        if let Err(e) = res {
            println!("❌ [HTTP EDIT BL ERROR]: {:?}", e);
        }
    } else {
        bot.send_message(target_chat, text)
            .parse_mode(ParseMode::Html)
            .await?;
    }

    Ok(())
}

pub async fn cmd_warns(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;

    let target_user = match msg.reply_to_message().and_then(|m| m.from.as_ref()) {
        Some(u) => u.clone(),
        None => match msg.from.as_ref() {
            Some(u) => u.clone(),
            None => return Ok(()),
        },
    };

    let count = mod_db::get_user_warns(&pool, chat_id, target_user.id.0 as i64).await?;
    let settings = mod_db::get_or_create_warn_settings(&pool, chat_id).await?;

    let response = format!(
        "📊 Status Warn <b>{}</b>: <code>{}/{}</code>",
        target_user.first_name, count, settings.warn_limit
    );

    bot.send_message(msg.chat.id, response)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

pub async fn cmd_resetwarn(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let clicker_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    // 🛡️ ISSUER CHECK: Pastikan pengirim perintah adalah Admin Grup atau Bot Owner
    if !is_user_immune(&bot, msg.chat.id, clicker_id).await {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya bisa digunakan oleh Admin grup atau Bot Owner!",
        )
        .await?;
        return Ok(());
    }

    let target_user = match msg.reply_to_message().and_then(|m| m.from.as_ref()) {
        Some(u) => u,
        None => {
            bot.send_message(
                msg.chat.id,
                "⚠️ <b>Penggunaan:</b> Balas (reply) pesan user yang ingin di-reset warn-nya, lalu ketik <code>/resetwarn</code>",
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    let target_user_id = target_user.id.0 as i64;
    let target_name = &target_user.first_name;

    mod_db::reset_user_warns(&pool, chat_id, target_user_id).await?;

    bot.send_message(
        msg.chat.id,
        format!("✅ Poin warn untuk <b>{target_name}</b> (<code>{target_user_id}</code>) telah berhasil di-reset menjadi <b>0</b>."),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
