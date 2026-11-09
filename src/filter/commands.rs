use crate::db;
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{ParseMode, ReplyParameters},
};

// 🛠️ Helper Parsing Argumen (Mendukung tanda kutip untuk frasa)
fn parse_filter_args(args: &str) -> Option<(String, String)> {
    let args = args.trim();
    if args.is_empty() {
        return None;
    }

    if args.starts_with('"') {
        if let Some(end_quote) = args[1..].find('"') {
            let trigger = args[1..=end_quote].trim().to_lowercase();
            let reply = args[end_quote + 2..].trim().to_string();
            if !trigger.is_empty() && !reply.is_empty() {
                return Some((trigger, reply));
            }
        }
    } else if let Some((trigger, reply)) = args.split_once(char::is_whitespace) {
        let trigger = trigger.trim().to_lowercase();
        let reply = reply.trim().to_string();
        if !trigger.is_empty() && !reply.is_empty() {
            return Some((trigger, reply));
        }
    }
    None
}

// 🔹 Command: /filter <trigger> <reply>
pub async fn cmd_filter(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or_default();
    let args = text.splitn(2, ' ').nth(1).unwrap_or("");

    if let Some((trigger, reply)) = parse_filter_args(args) {
        db::add_filter(&pool, msg.chat.id.0, &trigger, &reply).await?;
        bot.send_message(
            msg.chat.id,
            format!("✅ Berhasil menyimpan filter untuk <code>{trigger}</code>!"),
        )
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Format Salah!</b>\n\n\
             • Contoh 1 kata: <code>/filter halo Hai juga!</code>\n\
             • Contoh frasa: <code>/filter \"selamat pagi\" Pagi juga, kawan!</code>",
        )
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    }
    Ok(())
}

// 🔹 Command: /filters
pub async fn cmd_filters(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let filters = db::get_filters(&pool, msg.chat.id.0).await?;

    if filters.is_empty() {
        bot.send_message(msg.chat.id, "ℹ️ Belum ada filter aktif di grup ini.")
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(());
    }

    let mut response = format!("<b>Daftar Filter Aktif ({}) :</b>\n\n", filters.len());
    for f in filters {
        response.push_str(&format!("• <code>{}</code>\n", f.trigger_word));
    }

    bot.send_message(msg.chat.id, response)
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}

// 🔹 Command: /stop <trigger>
pub async fn cmd_stop(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let text = msg.text().unwrap_or_default();
    let trigger = text
        .splitn(2, ' ')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_matches('"');

    if trigger.is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ Harap masukkan kata kunci! Contoh: <code>/stop halo</code> atau <code>/stop \"selamat pagi\"</code>",
        )
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
        return Ok(());
    }

    if db::delete_filter(&pool, msg.chat.id.0, trigger).await? {
        bot.send_message(
            msg.chat.id,
            format!("🗑️ Filter untuk <code>{trigger}</code> telah dihapus."),
        )
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    } else {
        bot.send_message(
            msg.chat.id,
            format!("❌ Filter <code>{trigger}</code> tidak ditemukan."),
        )
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    }

    Ok(())
}

// 🔹 Command: /stopall
pub async fn cmd_stopall(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let count = db::delete_all_filters(&pool, msg.chat.id.0).await?;
    bot.send_message(
        msg.chat.id,
        format!("🧹 Berhasil menghapus <b>{count}</b> filter dari grup ini."),
    )
    .parse_mode(ParseMode::Html)
    .reply_parameters(ReplyParameters::new(msg.id))
    .await?;

    Ok(())
}
