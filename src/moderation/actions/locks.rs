use crate::moderation::{db as mod_db, models::LockType};
use sqlx::SqlitePool;
use std::str::FromStr;
use teloxide::{
    prelude::*,
    types::{Message, ParseMode},
};

/// Handler Perintah /lock <tipe>
pub async fn cmd_lock(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if msg.chat.is_private() {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya dapat digunakan di dalam grup!",
        )
        .await?;
        return Ok(());
    }

    let user_id = match msg.from.as_ref() {
        Some(user) => user.id,
        None => return Ok(()),
    };

    let is_admin = match bot.get_chat_administrators(msg.chat.id).await {
        Ok(admins) => admins.iter().any(|a| a.user.id == user_id),
        Err(e) => {
            log::error!("Gagal mengambil daftar admin: {:?}", e);
            false
        }
    };

    if !is_admin {
        bot.send_message(
            msg.chat.id,
            "⚠️ Hanya <b>Admin</b> yang dapat mengubah pengaturan Lock!",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let mut parts = text.split_whitespace();
    parts.next(); // Skip "/lock"

    let lock_type_str = match parts.next() {
        Some(arg) => arg,
        None => {
            bot.send_message(
                msg.chat.id,
                "⚠️ Format salah! Gunakan: <code>/lock &lt;tipe&gt;</code>\nContoh: <code>/lock photo</code>\nKetik <code>/locktypes</code> untuk daftar tipe.",
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    let lock_type = match LockType::from_str(lock_type_str) {
        Ok(lt) => lt,
        Err(_) => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "❌ Tipe lock <code>{}</code> tidak valid!\nKetik <code>/locktypes</code> untuk melihat daftar lengkap.",
                    lock_type_str
                ),
            )
            .parse_mode(ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    sqlx::query(
        "INSERT INTO chat_locks (chat_id, lock_type) VALUES (?, ?) ON CONFLICT(chat_id, lock_type) DO NOTHING",
    )
    .bind(msg.chat.id.0)
    .bind(lock_type.as_str())
    .execute(&pool)
    .await?;

    bot.send_message(
        msg.chat.id,
        format!(
            "🔒 Berhasil mengunci konten <code>{}</code> di grup ini.",
            lock_type.as_str()
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

/// Handler Perintah /unlock <tipe>
pub async fn cmd_unlock(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let clicker_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    let member = bot.get_chat_member(msg.chat.id, clicker_id).await;
    if !member.map_or(false, |m| m.is_administrator() || m.is_owner()) {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya bisa digunakan oleh Admin grup!",
        )
        .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/unlock").unwrap_or("").trim();

    if args.is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Penggunaan:</b> <code>/unlock &lt;tipe&gt;</code>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let types: Vec<&str> = args.split_whitespace().collect();
    let mut unlocked_list = Vec::new();

    for t in types {
        if let Ok(lock_type) = LockType::from_str(t) {
            mod_db::set_lock(&pool, chat_id, lock_type.as_str(), false).await?;
            unlocked_list.push(format!("<code>{}</code>", lock_type.as_str()));
        }
    }

    if unlocked_list.is_empty() {
        bot.send_message(msg.chat.id, "❌ Tipe lock tidak dikenali!")
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            format!("🔓 Berhasil membuka kunci: {}", unlocked_list.join(", ")),
        )
        .parse_mode(ParseMode::Html)
        .await?;
    }

    Ok(())
}

/// Handler Perintah /locks
pub async fn cmd_locks(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let active_locks = mod_db::get_active_locks(&pool, chat_id).await?;

    if active_locks.is_empty() {
        bot.send_message(
            msg.chat.id,
            "🔓 <b>Tidak ada item yang terkunci di grup ini.</b>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
    } else {
        let formatted: Vec<String> = active_locks
            .iter()
            .map(|l| format!("• <code>{l}</code>"))
            .collect();
        bot.send_message(
            msg.chat.id,
            format!(
                "🔒 <b>Daftar Item Terkunci (Active Locks):</b>\n\n{}",
                formatted.join("\n")
            ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
    }

    Ok(())
}

/// Handler Perintah /locktypes
pub async fn handle_locktypes(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = "<b>🔒 Daftar Tipe Lock yang Didukung:</b>\n\n\
    <code>all</code>, <code>album</code>, <code>anonchannel</code>, <code>audio</code>, <code>bot</code>, <code>botlink</code>, <code>button</code>, \
    <code>cashtag</code>, <code>checklist</code>, <code>cjk</code>, <code>collage</code>, <code>command</code>, <code>comment</code>, \
    <code>contact</code>, <code>cyrillic</code>, <code>document</code>, <code>email</code>, <code>emoji</code>, <code>emojicustom</code>, \
    <code>emojigame</code>, <code>emojionly</code>, <code>externalreply</code>, <code>forward</code>, <code>forwardbot</code>, \
    <code>forwardchannel</code>, <code>forwardstory</code>, <code>forwarduser</code>, <code>game</code>, <code>gif</code>, \
    <code>guestbot</code>, <code>inline</code>, <code>invitelink</code>, <code>location</code>, <code>outsidereaction</code>, \
    <code>phone</code>, <code>photo</code>, <code>poll</code>, <code>reaction</code>, <code>richmessage</code>, <code>rtl</code>, \
    <code>slideshow</code>, <code>spoiler</code>, <code>sticker</code>, <code>stickeranimated</code>, <code>stickerpremium</code>, \
    <code>text</code>, <code>url</code>, <code>video</code>, <code>videonote</code>, <code>voice</code>, <code>zalgo</code>\n\n\
    <i>Gunakan <code>/lock &lt;tipe&gt;</code> untuk mengunci atau <code>/unlock &lt;tipe&gt;</code> untuk membuka.</i>";

    bot.send_message(msg.chat.id, text)
        .parse_mode(ParseMode::Html)
        .await?;

    Ok(())
}

/// Handler Perintah /lockwarns <on/off/yes/no>
pub async fn cmd_lockwarns(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let clicker_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    let member = bot.get_chat_member(msg.chat.id, clicker_id).await;
    if !member.map_or(false, |m| m.is_administrator() || m.is_owner()) {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya bisa digunakan oleh Admin grup!",
        )
        .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args = text
        .strip_prefix("/lockwarns")
        .unwrap_or("")
        .trim()
        .to_lowercase();

    match args.as_str() {
        "on" | "yes" | "1" => {
            mod_db::set_lock_warns(&pool, chat_id, true).await?;
            bot.send_message(
                msg.chat.id,
                "✅ <b>Lock Warns AKTIF!</b> Member yang melanggar lock akan diberikan warning.",
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        "off" | "no" | "0" => {
            mod_db::set_lock_warns(&pool, chat_id, false).await?;
            bot.send_message(msg.chat.id, "✅ <b>Lock Warns NONAKTIF!</b> Pesan terlarang hanya akan dihapus tanpa memberikan warning.")
                .parse_mode(ParseMode::Html)
                .await?;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "⚠️ <b>Penggunaan:</b> <code>/lockwarns &lt;on/off&gt;</code>",
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
    }

    Ok(())
}

/// Handler Perintah /allowlist [items]
pub async fn cmd_allowlist(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/allowlist").unwrap_or("").trim();

    if args.is_empty() {
        let items = mod_db::get_allowlist(&pool, chat_id).await?;
        if items.is_empty() {
            bot.send_message(msg.chat.id, "📋 <b>Allowlist grup ini masih kosong.</b>")
                .parse_mode(ParseMode::Html)
                .await?;
        } else {
            let list: Vec<String> = items
                .iter()
                .map(|i| format!("• <code>{}</code>", i.value))
                .collect();
            bot.send_message(
                msg.chat.id,
                format!("📋 <b>Daftar Item Allowlist:</b>\n\n{}", list.join("\n")),
            )
            .parse_mode(ParseMode::Html)
            .await?;
        }
        return Ok(());
    }

    let clicker_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    let member = bot.get_chat_member(msg.chat.id, clicker_id).await;
    if !member.map_or(false, |m| m.is_administrator() || m.is_owner()) {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya bisa digunakan oleh Admin grup!",
        )
        .await?;
        return Ok(());
    }

    let items: Vec<&str> = args.split_whitespace().collect();
    let mut added_count = 0;

    for item in items {
        mod_db::add_allowlist_item(&pool, chat_id, item).await?;
        added_count += 1;
    }

    bot.send_message(
        msg.chat.id,
        format!("✅ Berhasil menambahkan <b>{added_count}</b> item ke allowlist."),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

/// Handler Perintah /rmallowlist <items>
pub async fn cmd_rmallowlist(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let clicker_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    let member = bot.get_chat_member(msg.chat.id, clicker_id).await;
    if !member.map_or(false, |m| m.is_administrator() || m.is_owner()) {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya bisa digunakan oleh Admin grup!",
        )
        .await?;
        return Ok(());
    }

    let text = msg.text().unwrap_or("");
    let args = text.strip_prefix("/rmallowlist").unwrap_or("").trim();

    if args.is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Penggunaan:</b> <code>/rmallowlist &lt;domain/url&gt;</code>",
        )
        .parse_mode(ParseMode::Html)
        .await?;
        return Ok(());
    }

    let items: Vec<&str> = args.split_whitespace().collect();
    let mut removed_count = 0;

    for item in items {
        if mod_db::remove_allowlist_item(&pool, chat_id, item).await? {
            removed_count += 1;
        }
    }

    bot.send_message(
        msg.chat.id,
        format!("🗑️ Berhasil menghapus <b>{removed_count}</b> item dari allowlist."),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

/// Handler Perintah /rmallowlistall
pub async fn cmd_rmallowlistall(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id.0;
    let clicker_id = match msg.from.as_ref() {
        Some(u) => u.id,
        None => return Ok(()),
    };

    let member = bot.get_chat_member(msg.chat.id, clicker_id).await;
    if !member.map_or(false, |m| m.is_administrator() || m.is_owner()) {
        bot.send_message(
            msg.chat.id,
            "❌ Perintah ini hanya bisa digunakan oleh Admin grup!",
        )
        .await?;
        return Ok(());
    }

    mod_db::clear_allowlist(&pool, chat_id).await?;

    bot.send_message(
        msg.chat.id,
        "🗑️ <b>Seluruh entri allowlist grup telah berhasil dihapus.</b>",
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}
