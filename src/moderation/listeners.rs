use crate::moderation::actions::warn::build_warn_alert_text;
use crate::moderation::{
    db as mod_db,
    utils::{contains_blacklisted_word, is_user_immune, parse_action_and_duration},
};
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{
        ChatId, ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, Message,
        MessageEntityKind, MessageOrigin, ParseMode, User, UserId,
    },
};

pub async fn handle_blacklist_check(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;

    if !msg.chat.is_group() && !msg.chat.is_supergroup() {
        return Ok(false);
    }

    let user = match msg.from.as_ref() {
        Some(u) if !u.is_bot => u,
        _ => return Ok(false),
    };

    // 🛡️ IMMUNITY CHECK: Skip filter blacklist jika user adalah Owner Bot atau Admin/Owner Grup
    if is_user_immune(&bot, msg.chat.id, user.id).await {
        return Ok(false);
    }

    if msg.is_automatic_forward() {
        return Ok(false);
    }

    let text = match msg.text().or_else(|| msg.caption()) {
        Some(t) => t,
        None => return Ok(false),
    };

    let blacklist = mod_db::get_blacklist(&pool, chat_id).await?;
    if blacklist.is_empty() {
        return Ok(false);
    }

    for entry in blacklist {
        if contains_blacklisted_word(text, &entry.word) {
            let target_msg_id = msg.id.0;
            let _ = bot.delete_message(msg.chat.id, msg.id).await;

            // ========================================================
            // 🎯 PENENTUAN AKSI HUKUMAN HYBRID SLOT SYSTEM (SLOT 0 VS 1-5)
            // ========================================================
            let effective_action = if entry.slot_id == 0 {
                // Slot 0 = Custom Granular Action
                entry.action.clone()
            } else {
                // Slot 1..=5 = Bulk Action dari tabel slot_settings
                match mod_db::get_slot_action(&pool, chat_id, entry.slot_id).await? {
                    Some(act) if !act.trim().is_empty() => act,
                    _ => "tmute1d".to_string(), // Fallback default jika slot belum diatur
                }
            };

            execute_blacklist_action_ext(
                bot.clone(),
                pool.clone(),
                msg.chat.id.0,
                user,
                &entry.word,
                &effective_action,
                Some(target_msg_id),
            )
            .await?;

            return Ok(true);
        }
    }

    Ok(false)
}

pub async fn handle_locks_check(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if (!msg.chat.is_group() && !msg.chat.is_supergroup())
        || msg.from.as_ref().map_or(true, |u| u.is_bot)
    {
        return Ok(false);
    }

    let chat_id = msg.chat.id.0;
    let user = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(false),
    };

    // 🛡️ IMMUNITY CHECK: Melindungi Admin/Owner Grup & Owner Bot dari Aturan Lock
    if is_user_immune(&bot, msg.chat.id, user.id).await {
        return Ok(false);
    }

    let active_locks = mod_db::get_active_locks(&pool, chat_id).await?;
    if active_locks.is_empty() {
        return Ok(false);
    }

    let text_content = msg.text().or_else(|| msg.caption()).unwrap_or("");
    let entities = msg
        .entities()
        .or_else(|| msg.caption_entities())
        .unwrap_or(&[]);
    let mut violated_lock: Option<&str> = None;

    if active_locks.contains(&"all".to_string()) {
        violated_lock = Some("all");
    }

    if violated_lock.is_none()
        && active_locks.contains(&"album".to_string())
        && msg.media_group_id().is_some()
    {
        violated_lock = Some("album");
    }

    if violated_lock.is_none() {
        if active_locks.contains(&"anonchannel".to_string())
            && msg.sender_chat.as_ref().map_or(false, |c| c.is_channel())
        {
            violated_lock = Some("anonchannel");
        } else if active_locks.contains(&"comment".to_string()) && msg.is_automatic_forward() {
            violated_lock = Some("comment");
        }
    }

    if violated_lock.is_none() {
        if active_locks.contains(&"photo".to_string()) && msg.photo().is_some() {
            violated_lock = Some("photo");
        } else if active_locks.contains(&"video".to_string()) && msg.video().is_some() {
            violated_lock = Some("video");
        } else if active_locks.contains(&"videonote".to_string()) && msg.video_note().is_some() {
            violated_lock = Some("videonote");
        } else if active_locks.contains(&"audio".to_string()) && msg.audio().is_some() {
            violated_lock = Some("audio");
        } else if active_locks.contains(&"voice".to_string()) && msg.voice().is_some() {
            violated_lock = Some("voice");
        } else if active_locks.contains(&"document".to_string()) && msg.document().is_some() {
            violated_lock = Some("document");
        } else if active_locks.contains(&"gif".to_string()) && msg.animation().is_some() {
            violated_lock = Some("gif");
        } else if active_locks.contains(&"location".to_string()) && msg.location().is_some() {
            violated_lock = Some("location");
        } else if active_locks.contains(&"contact".to_string()) && msg.contact().is_some() {
            violated_lock = Some("contact");
        } else if active_locks.contains(&"poll".to_string()) && msg.poll().is_some() {
            violated_lock = Some("poll");
        } else if active_locks.contains(&"game".to_string()) && msg.game().is_some() {
            violated_lock = Some("game");
        } else if active_locks.contains(&"emojigame".to_string()) && msg.dice().is_some() {
            violated_lock = Some("emojigame");
        }
    }

    if violated_lock.is_none() && msg.sticker().is_some() {
        let st = msg.sticker().unwrap();
        if active_locks.contains(&"sticker".to_string()) {
            violated_lock = Some("sticker");
        } else if active_locks.contains(&"stickeranimated".to_string())
            && (st.is_animated() || st.is_video())
        {
            violated_lock = Some("stickeranimated");
        } else if active_locks.contains(&"stickerpremium".to_string())
            && st.premium_animation().is_some()
        {
            violated_lock = Some("stickerpremium");
        }
    }

    if violated_lock.is_none() && msg.forward_origin().is_some() {
        if active_locks.contains(&"forward".to_string()) {
            violated_lock = Some("forward");
        } else if let Some(origin) = msg.forward_origin() {
            match origin {
                MessageOrigin::User { .. } => {
                    if active_locks.contains(&"forwarduser".to_string()) {
                        violated_lock = Some("forwarduser");
                    }
                }
                MessageOrigin::Chat { .. } | MessageOrigin::Channel { .. } => {
                    if active_locks.contains(&"forwardchannel".to_string()) {
                        violated_lock = Some("forwardchannel");
                    }
                }
                MessageOrigin::HiddenUser { .. } => {
                    if active_locks.contains(&"forwarduser".to_string()) {
                        violated_lock = Some("forwarduser");
                    }
                }
            }
        }
    }

    if violated_lock.is_none() {
        if active_locks.contains(&"inline".to_string()) && msg.via_bot.is_some() {
            violated_lock = Some("inline");
        } else if active_locks.contains(&"button".to_string()) && msg.reply_markup().is_some() {
            violated_lock = Some("button");
        } else if active_locks.contains(&"externalreply".to_string())
            && msg.reply_to_message().is_some()
        {
            violated_lock = Some("externalreply");
        }
    }

    if violated_lock.is_none() {
        let lower_text = text_content.to_lowercase();
        let allowlist = mod_db::get_allowlist(&pool, chat_id).await?;

        for entity in entities {
            match &entity.kind {
                MessageEntityKind::Url | MessageEntityKind::TextLink { .. } => {
                    let is_allowlisted = allowlist
                        .iter()
                        .any(|item| lower_text.contains(&item.value));
                    if !is_allowlisted {
                        if lower_text.contains("t.me/") || lower_text.contains("telegram.me/") {
                            if active_locks.contains(&"invitelink".to_string()) {
                                violated_lock = Some("invitelink");
                                break;
                            }
                        } else if active_locks.contains(&"url".to_string()) {
                            violated_lock = Some("url");
                            break;
                        }
                    }
                }
                MessageEntityKind::BotCommand => {
                    if active_locks.contains(&"command".to_string()) {
                        violated_lock = Some("command");
                        break;
                    }
                }
                MessageEntityKind::Cashtag => {
                    if active_locks.contains(&"cashtag".to_string()) {
                        violated_lock = Some("cashtag");
                        break;
                    }
                }
                MessageEntityKind::Email => {
                    if active_locks.contains(&"email".to_string()) {
                        violated_lock = Some("email");
                        break;
                    }
                }
                MessageEntityKind::PhoneNumber => {
                    if active_locks.contains(&"phone".to_string()) {
                        violated_lock = Some("phone");
                        break;
                    }
                }
                MessageEntityKind::Spoiler => {
                    if active_locks.contains(&"spoiler".to_string()) {
                        violated_lock = Some("spoiler");
                        break;
                    }
                }
                MessageEntityKind::CustomEmoji { .. } => {
                    if active_locks.contains(&"emojicustom".to_string()) {
                        violated_lock = Some("emojicustom");
                        break;
                    }
                }
                MessageEntityKind::Mention => {
                    if active_locks.contains(&"botlink".to_string()) && lower_text.contains("bot") {
                        violated_lock = Some("botlink");
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    if violated_lock.is_none() && !text_content.is_empty() {
        if active_locks.contains(&"text".to_string()) {
            violated_lock = Some("text");
        } else {
            if active_locks.contains(&"cjk".to_string()) && text_content.chars().any(|c| matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3040}'..='\u{30FF}' | '\u{AC00}'..='\u{D7AF}')) {
                violated_lock = Some("cjk");
            }
            else if active_locks.contains(&"cyrillic".to_string()) && text_content.chars().any(|c| matches!(c, '\u{0400}'..='\u{04FF}')) {
                violated_lock = Some("cyrillic");
            }
            else if active_locks.contains(&"rtl".to_string()) && text_content.chars().any(|c| matches!(c, '\u{0600}'..='\u{06FF}' | '\u{0590}'..='\u{05FF}')) {
                violated_lock = Some("rtl");
            }
            else if active_locks.contains(&"zalgo".to_string()) {
                let combining_count = text_content.chars().filter(|c| matches!(c, '\u{0300}'..='\u{036F}')).count();
                if combining_count > 3 {
                    violated_lock = Some("zalgo");
                }
            }
        }
    }

    if let Some(lock_name) = violated_lock {
        let target_msg_id = msg.id.0;
        let _ = bot.delete_message(msg.chat.id, msg.id).await;

        let settings = mod_db::get_or_create_warn_settings(&pool, chat_id).await?;
        let lock_warns_enabled = settings.lock_warns == 1;

        if lock_warns_enabled {
            let user_id = user.id.0 as i64;
            let user_name = &user.first_name;
            let warn_count = mod_db::increment_user_warn(&pool, chat_id, user_id).await?;

            let lock_detail = format!("Locked {lock_name}");
            let text = build_warn_alert_text(
                "LOCK VIOLATION",
                user_id,
                user_name,
                "SYSTEM",
                Some(("Violation", &lock_detail)),
                warn_count,
                settings.warn_limit,
            );

            if warn_count >= settings.warn_limit {
                mod_db::reset_user_warns(&pool, chat_id, user_id).await?;
                execute_direct_action_ext(
                    bot,
                    pool,
                    chat_id,
                    user,
                    &format!("Lock Violation ({lock_name})"),
                    &settings.warn_action,
                    Some(target_msg_id),
                )
                .await?;
            } else {
                send_action_notification_ext(
                    &bot,
                    &pool,
                    chat_id,
                    user,
                    &format!("Lock Warning [{warn_count}/{}]", settings.warn_limit),
                    lock_name,
                    &text,
                    true,
                    true,
                    Some(target_msg_id),
                )
                .await?;
            }
        }

        return Ok(true);
    }

    Ok(false)
}

// 🟢 Wrapper Kompatibilitas untuk execute_direct_action (6 argumen)
pub async fn execute_direct_action(
    bot: Bot,
    pool: SqlitePool,
    chat_id: i64,
    user: &User,
    word_detected: &str,
    action_tag: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    execute_direct_action_ext(bot, pool, chat_id, user, word_detected, action_tag, None).await
}

pub async fn execute_direct_action_ext(
    bot: Bot,
    pool: SqlitePool,
    chat_id: i64,
    user: &User,
    word_detected: &str,
    action_tag: &str,
    msg_id: Option<i32>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base_action, duration) = parse_action_and_duration(action_tag);
    let user_name = &user.first_name;

    match base_action.as_str() {
        "del" => {
            send_action_notification_ext(
                &bot,
                &pool,
                chat_id,
                user,
                "Silent Delete",
                word_detected,
                "",
                false,
                false,
                msg_id,
            )
            .await?;
        }
        "mute" => {
            let permissions = ChatPermissions::empty();
            if let Err(e) = bot
                .restrict_chat_member(ChatId(chat_id), UserId(user.id.0), permissions)
                .await
            {
                log::warn!("Gagal Mute user {} di chat {}: {:?}", user.id.0, chat_id, e);
                let _ = bot.send_message(
                    ChatId(chat_id),
                    format!("⚠️ <b>Gagal memproses Mute!</b> Pastikan bot diangkat menjadi Admin dengan hak <i>Restrict Members</i>."),
                )
                .parse_mode(ParseMode::Html)
                .await;
            } else {
                send_action_notification_ext(
                    &bot, &pool, chat_id, user, "Permanently Muted", word_detected,
                    &format!("🔇 <b>{user_name}</b> telah di-Mute permanen karena menggunakan kata terlarang."),
                    true, true, msg_id,
                ).await?;
            }
        }
        "tmute" => {
            let secs = duration.unwrap_or(3600) as i64;
            let permissions = ChatPermissions::empty();
            let until_date = chrono::Utc::now() + chrono::Duration::seconds(secs);

            if let Err(e) = bot
                .restrict_chat_member(ChatId(chat_id), UserId(user.id.0), permissions)
                .until_date(until_date)
                .await
            {
                log::warn!(
                    "Gagal Temporary Mute user {} di chat {}: {:?}",
                    user.id.0,
                    chat_id,
                    e
                );
                let _ = bot.send_message(
                    ChatId(chat_id),
                    format!("⚠️ <b>Gagal memproses Temporary Mute!</b> Pastikan bot diangkat menjadi Admin dengan hak <i>Restrict Members</i>."),
                )
                .parse_mode(ParseMode::Html)
                .await;
            } else {
                send_action_notification_ext(
                    &bot,
                    &pool,
                    chat_id,
                    user,
                    "Temporary Muted",
                    word_detected,
                    &format!("⏳ <b>{user_name}</b> di-Mute sementara karena kata terlarang."),
                    true,
                    true,
                    msg_id,
                )
                .await?;
            }
        }
        "kick" => {
            if let Err(e) = bot
                .ban_chat_member(ChatId(chat_id), UserId(user.id.0))
                .await
            {
                log::warn!("Gagal Kick user {} di chat {}: {:?}", user.id.0, chat_id, e);
                let _ = bot.send_message(
                    ChatId(chat_id),
                    format!("⚠️ <b>Gagal mengeluarkan user!</b> Pastikan bot diangkat menjadi Admin dengan hak <i>Ban Users</i>."),
                )
                .parse_mode(ParseMode::Html)
                .await;
            } else {
                let _ = bot
                    .unban_chat_member(ChatId(chat_id), UserId(user.id.0))
                    .await;
                send_action_notification_ext(
                    &bot,
                    &pool,
                    chat_id,
                    user,
                    "Kicked from group",
                    word_detected,
                    &format!("🥾 <b>{user_name}</b> telah dikeluarkan (kick) dari grup."),
                    false,
                    true,
                    msg_id,
                )
                .await?;
            }
        }
        "ban" | "tban" | _ => {
            if let Err(e) = bot
                .ban_chat_member(ChatId(chat_id), UserId(user.id.0))
                .await
            {
                log::warn!("Gagal Ban user {} di chat {}: {:?}", user.id.0, chat_id, e);
                let _ = bot.send_message(
                    ChatId(chat_id),
                    format!("⚠️ <b>Gagal memproses Ban!</b> Pastikan bot diangkat menjadi Admin dengan hak <i>Ban Users</i>."),
                )
                .parse_mode(ParseMode::Html)
                .await;
            } else {
                send_action_notification_ext(
                    &bot,
                    &pool,
                    chat_id,
                    user,
                    "Banned from group",
                    word_detected,
                    &format!("🚫 <b>{user_name}</b> telah di-Ban dari grup."),
                    false,
                    true,
                    msg_id,
                )
                .await?;
            }
        }
    }

    Ok(())
}

// 🟢 Wrapper Kompatibilitas untuk execute_blacklist_action (6 argumen)
pub async fn execute_blacklist_action(
    bot: Bot,
    pool: SqlitePool,
    chat_id: i64,
    user: &User,
    word_detected: &str,
    action_tag: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    execute_blacklist_action_ext(bot, pool, chat_id, user, word_detected, action_tag, None).await
}

pub async fn execute_blacklist_action_ext(
    bot: Bot,
    pool: SqlitePool,
    chat_id: i64,
    user: &User,
    word_detected: &str,
    action_tag: &str,
    msg_id: Option<i32>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (base_action, _) = parse_action_and_duration(action_tag);

    if base_action == "warn"
        || (!["del", "mute", "tmute", "kick", "ban", "tban"].contains(&base_action.as_str()))
    {
        let user_id = user.id.0 as i64;
        let user_name = &user.first_name;
        let warn_count = mod_db::increment_user_warn(&pool, chat_id, user_id).await?;
        let settings = mod_db::get_or_create_warn_settings(&pool, chat_id).await?;

        let text = build_warn_alert_text(
            "BLACKLIST VIOLATION",
            user_id,
            user_name,
            "SYSTEM",
            Some(("Trigger Word", word_detected)),
            warn_count,
            settings.warn_limit,
        );

        if warn_count >= settings.warn_limit {
            mod_db::reset_user_warns(&pool, chat_id, user_id).await?;
            execute_direct_action_ext(
                bot,
                pool,
                chat_id,
                user,
                word_detected,
                &settings.warn_action,
                msg_id,
            )
            .await?;
        } else {
            send_action_notification_ext(
                &bot,
                &pool,
                chat_id,
                user,
                &format!("Warning [{warn_count}/{}]", settings.warn_limit),
                word_detected,
                &text,
                true,
                true,
                msg_id,
            )
            .await?;
        }
    } else {
        execute_direct_action_ext(bot, pool, chat_id, user, word_detected, action_tag, msg_id)
            .await?;
    }

    Ok(())
}

// 🟢 Wrapper Kompatibilitas untuk send_action_notification (9 argumen)
pub async fn send_action_notification(
    bot: &Bot,
    pool: &SqlitePool,
    chat_id: i64,
    user: &User,
    action_title: &str,
    word_detected: &str,
    group_message_text: &str,
    show_appeal_button: bool,
    send_to_group: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    send_action_notification_ext(
        bot,
        pool,
        chat_id,
        user,
        action_title,
        word_detected,
        group_message_text,
        show_appeal_button,
        send_to_group,
        None,
    )
    .await
}

pub async fn send_action_notification_ext(
    bot: &Bot,
    pool: &SqlitePool,
    chat_id: i64,
    user: &User,
    action_title: &str,
    word_detected: &str,
    group_message_text: &str,
    show_appeal_button: bool,
    send_to_group: bool,
    msg_id: Option<i32>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_id = user.id.0 as i64;

    if send_to_group && !group_message_text.is_empty() {
        let mut keyboard = Vec::new();

        if show_appeal_button {
            keyboard.push(vec![InlineKeyboardButton::callback(
                "⚖️ Appeal Warning",
                format!("appeal_{user_id}"),
            )]);

            keyboard.push(vec![InlineKeyboardButton::callback(
                "🗑️ Dismiss Warning (Admin Only)",
                format!("dismiss_warn_{user_id}"),
            )]);
        }

        let reply_markup = InlineKeyboardMarkup::new(keyboard);

        let _ = bot
            .send_message(ChatId(chat_id), group_message_text)
            .parse_mode(ParseMode::Html)
            .reply_markup(reply_markup)
            .await;
    }

    let settings = mod_db::get_or_create_warn_settings(pool, chat_id).await?;
    if let Some(log_channel_id) = settings.log_channel_id {
        let msg_link_html = if let Some(m_id) = msg_id {
            let clean_chat_id = chat_id.to_string().replace("-100", "");
            format!("<a href=\"https://t.me/c/{clean_chat_id}/{m_id}\">Buka Lokasi Chat</a>")
        } else {
            "<code>N/A</code>".to_string()
        };

        let log_text = format!(
            "🛡️ <b>MODERATION AUDIT LOG</b>\n\n\
             • <b>Grup ID:</b> <code>{chat_id}</code>\n\
             • <b>Target:</b> <a href=\"tg://user?id={user_id}\">{}</a> (<code>{user_id}</code>)\n\
             • <b>Pelanggaran:</b> Blacklist/Lock (<code>{word_detected}</code>)\n\
             • <b>Tindakan:</b> {}\n\
             • <b>Link Chat:</b> {}\n\
             • <b>Waktu:</b> <code>{} UTC</code>",
            user.first_name,
            action_title,
            msg_link_html,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        let _ = bot
            .send_message(ChatId(log_channel_id), log_text)
            .parse_mode(ParseMode::Html)
            .await;
    }

    Ok(())
}
