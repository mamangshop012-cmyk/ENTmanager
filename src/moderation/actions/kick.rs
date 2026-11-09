use crate::moderation::{
    resolver::{is_admin, resolve_target},
    utils::is_user_immune,
};
use sqlx::SqlitePool;
use teloxide::{
    prelude::*,
    types::{ParseMode, ReplyParameters, UserId},
};

pub async fn handle_kick(
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
        Ok((target, reason)) => {
            let target_id = UserId(target.user_id);

            // 🛡️ TARGET IMMUNITY CHECK: Cegah kick terhadap Bot Owner atau Admin/Owner Grup
            if is_user_immune(bot, chat_id, target_id).await {
                bot.send_message(
                    chat_id,
                    "🛡️<code>[SYSTEM]</code> User memiliki hak kekebalan.",
                )
                .parse_mode(ParseMode::Html)
                .await?;
                return Ok(());
            }

            if bot.ban_chat_member(chat_id, target_id).await.is_ok() {
                let _ = bot.unban_chat_member(chat_id, target_id).await;

                let user_link = format!(
                    "<a href=\"tg://user?id={}\">{}</a>",
                    target.user_id, target.first_name
                );
                let mut response = format!("👢 {} telah dikeluarkan dari grup.", user_link);
                if !reason.is_empty() {
                    response.push_str(&format!("\n💬 <b>Alasan:</b> {}", reason));
                }

                bot.send_message(chat_id, response)
                    .parse_mode(ParseMode::Html)
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

pub async fn handle_kickme(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let sender = match msg.from.as_ref() {
        Some(u) => u,
        None => return Ok(()),
    };

    // 🛡️ IMMUNITY CHECK: Admin & Bot Owner tidak bisa self-kick
    if is_user_immune(bot, chat_id, sender.id).await {
        bot.send_message(
            chat_id,
            "⚠️ Admin/Owner tidak bisa menggunakan perintah <code>/kickme</code>.",
        )
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
        return Ok(());
    }

    let user_link = format!(
        "<a href=\"tg://user?id={}\">{}</a>",
        sender.id.0, sender.first_name
    );
    if bot.ban_chat_member(chat_id, sender.id).await.is_ok() {
        let _ = bot.unban_chat_member(chat_id, sender.id).await;
        bot.send_message(
            chat_id,
            format!("👋 {} telah keluar dari grup (Self-kick).", user_link),
        )
        .parse_mode(ParseMode::Html)
        .await?;
    }
    Ok(())
}
