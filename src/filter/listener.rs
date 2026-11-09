use super::parser::parse_filter_message;
use crate::db;
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{ParseMode, ReplyParameters},
};

pub async fn handle_filter_check(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if (!msg.chat.is_group() && !msg.chat.is_supergroup())
        || msg.from.as_ref().map_or(true, |u| u.is_bot)
    {
        return Ok(false);
    }

    let text = match msg.text().or_else(|| msg.caption()) {
        Some(t) => t,
        None => return Ok(false),
    };

    let filters = db::get_filters(&pool, msg.chat.id.0).await?;
    if filters.is_empty() {
        return Ok(false);
    }

    let lower_text = text.to_lowercase();
    let chat_title = msg.chat.title().unwrap_or("grup");

    for f in filters {
        if lower_text.contains(&f.trigger_word) {
            // 🎯 Parse variabel placeholder ({first}, {username}, {chatname}, dll) & tombol inline
            let (final_text, reply_markup) =
                parse_filter_message(&f.reply_text, msg.from.as_ref(), chat_title);

            let mut req = bot
                .send_message(msg.chat.id, &final_text)
                .reply_parameters(ReplyParameters::new(msg.id))
                .parse_mode(ParseMode::Html);

            if !reply_markup.inline_keyboard.is_empty() {
                req = req.reply_markup(reply_markup);
            }

            let _ = req.await;

            return Ok(true); // Filter terpicu
        }
    }

    Ok(false)
}
