use crate::moderation::{
    resolver::{is_admin, parse_duration, resolve_target},
    utils::{format_duration_human, is_user_immune},
};
use chrono::Utc;
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode, UserId},
};

pub async fn handle_mute(
    bot: &Bot,
    msg: &Message,
    args: &str,
    is_temp: bool,
    pool: &SqlitePool,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // 🛡️ ISSUER CHECK: Pastikan pengirim perintah adalah Admin Grup atau Bot Owner
    if let Some(issuer) = msg.from.as_ref() {
        if !is_user_immune(bot, chat_id, issuer.id).await {
            bot.send_message(
                chat_id,
                "❌ Perintah ini hanya bisa digunakan oleh Admin grup atau Bot Owner!",
            )
            .await?;
            return Ok(());
        }
    }

    let mut duration_secs = None;
    let mut parse_args = args;

    if is_temp {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if !parts.is_empty() {
            if let Some(dur) = parse_duration(parts[0]) {
                duration_secs = Some(dur as i64);
                parse_args = args.trim_start()[parts[0].len()..].trim();
            }
        }
    }

    match resolve_target(bot, msg, parse_args, pool).await {
        Ok((target, reason)) => {
            let target_id = UserId(target.user_id);

            // 🛡️ TARGET IMMUNITY CHECK: Cegah mute terhadap Bot Owner atau Admin/Owner Grup
            if is_user_immune(bot, chat_id, target_id).await {
                bot.send_message(
                    chat_id,
                    "🛡️<code>[SYSTEM]</code> User memiliki hak kekebalan.",
                )
                .parse_mode(ParseMode::Html)
                .await?;
                return Ok(());
            }

            let mut req = bot.restrict_chat_member(chat_id, target_id, ChatPermissions::empty());
            if let Some(secs) = duration_secs {
                let until = Utc::now() + chrono::Duration::seconds(secs);
                req = req.until_date(until);
            }

            if req.await.is_ok() {
                let user_link = format!(
                    "<a href=\"tg://user?id={}\">{}</a>",
                    target.user_id, target.first_name
                );
                let mut response = if let Some(secs) = duration_secs {
                    format!(
                        "🤐 {} di-mute sementara selama <b>{}</b>.",
                        user_link,
                        format_duration_human(secs)
                    )
                } else {
                    format!("🤐 {} telah di-mute permanen.", user_link)
                };

                if !reason.is_empty() {
                    response.push_str(&format!("\n💬 <b>Alasan:</b> {}", reason));
                }

                let keyboard =
                    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                        "⚖️ Appeal Warning",
                        format!("appeal_{}", target.user_id),
                    )]]);

                bot.send_message(chat_id, response)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;

                if let Some(reply_msg) = msg.reply_to_message() {
                    let _ = bot.delete_message(chat_id, reply_msg.id).await;
                }
                let _ = bot.delete_message(chat_id, msg.id).await;
            }
        }
        Err(err) => {
            bot.send_message(chat_id, format!("⚠️ {}", err)).await?;
        }
    }

    Ok(())
}

pub async fn handle_unmute(
    bot: &Bot,
    msg: &Message,
    args: &str,
    pool: &SqlitePool,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;

    // 🛡️ ISSUER CHECK: Pastikan pengirim perintah adalah Admin Grup atau Bot Owner
    if let Some(issuer) = msg.from.as_ref() {
        if !is_user_immune(bot, chat_id, issuer.id).await {
            bot.send_message(
                chat_id,
                "❌ Perintah ini hanya bisa digunakan oleh Admin grup atau Bot Owner!",
            )
            .await?;
            return Ok(());
        }
    }

    match resolve_target(bot, msg, args, pool).await {
        Ok((target, _)) => {
            let target_id = UserId(target.user_id);
            if bot
                .restrict_chat_member(chat_id, target_id, ChatPermissions::all())
                .await
                .is_ok()
            {
                let user_link = format!(
                    "<a href=\"tg://user?id={}\">{}</a>",
                    target.user_id, target.first_name
                );
                bot.send_message(
                    chat_id,
                    format!("🔊 Pembungkaman untuk {} telah dicabut.", user_link),
                )
                .parse_mode(ParseMode::Html)
                .await?;
            }
        }
        Err(err) => {
            bot.send_message(chat_id, format!("⚠️ {}", err)).await?;
        }
    }
    Ok(())
}
