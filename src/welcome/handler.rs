use crate::db::DbPool;
use crate::moderation::db as mod_db; // 👈 Import module database moderasi
use crate::terminal::{DashboardSender, DashboardSenderExt, Feature}; // 👈 Import TUI Dashboard
use crate::welcome::db as welcome_db;
use crate::welcome::parser::parse_welcome_message;
use std::time::Duration;
use teloxide::prelude::*;
use teloxide::types::InputFile;

/// 1. HANDLER SAAT MEMBER BARU JOIN
pub async fn handle_new_members(
    bot: Bot,
    msg: Message,
    pool: DbPool,
    dashboard_tx: DashboardSender,
) -> ResponseResult<()> {
    let Some(new_members) = msg.new_chat_members() else {
        return Ok(());
    };

    // 📊 TUI Dashboard: Kirim sinyal Started
    dashboard_tx.send_start(
        Feature::Welcome,
        format!("New join event in Chat ID: {}", msg.chat.id),
    );

    // 💡 FITUR AUTO-CACHE: Simpan semua member baru ke database SQLite detik ini juga!
    for member in new_members {
        if !member.is_bot {
            let _ = mod_db::upsert_user_cache(
                &pool,
                member.id.0,
                member.username.as_deref(),
                &member.first_name,
            )
            .await;
        }
    }

    let setting = welcome_db::get_welcome_setting(&pool, msg.chat.id.0).await;

    // Jika clean_welcome aktif, LANGSUNG HAPUS pesan sistem "X bergabung..." saat ini juga (0 detik)
    if setting.clean_welcome {
        let _ = bot.delete_message(msg.chat.id, msg.id).await;
    }

    // Jika fitur welcome dimatikan, berhenti di sini
    if !setting.is_enabled {
        dashboard_tx.send_success(Feature::Welcome, Some("Welcome feature disabled"));
        return Ok(());
    }

    let chat_title = msg.chat.title().unwrap_or("Grup Ini");

    for member in new_members {
        // Jangan sambut bot lain
        if member.is_bot {
            continue;
        }

        let (text, markup) = parse_welcome_message(&setting.welcome_text, member, chat_title);

        // Pengecekan apakah pesan welcome menggunakan media (Foto/GIF/Video) atau teks murni
        let sent_result =
            if let (Some(m_type), Some(f_id)) = (&setting.media_type, &setting.file_id) {
                let input_file = InputFile::file_id(f_id.clone());

                match m_type.as_str() {
                    "photo" => {
                        bot.send_photo(msg.chat.id, input_file)
                            .caption(text)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .reply_markup(markup)
                            .await
                    }
                    "animation" => {
                        bot.send_animation(msg.chat.id, input_file)
                            .caption(text)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .reply_markup(markup)
                            .await
                    }
                    "video" => {
                        bot.send_video(msg.chat.id, input_file)
                            .caption(text)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .reply_markup(markup)
                            .await
                    }
                    _ => {
                        bot.send_message(msg.chat.id, text)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .reply_markup(markup)
                            .await
                    }
                }
            } else {
                bot.send_message(msg.chat.id, text)
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .reply_markup(markup)
                    .await
            };

        match sent_result {
            Ok(sent_msg) => {
                // 📊 TUI Dashboard: Kirim sinyal Success
                dashboard_tx.send_success(
                    Feature::Welcome,
                    Some(format!("Welcomed {}", member.first_name)),
                );

                // FITUR AUTO-DELETE WELCOME BOT SETELAH 5 MENIT (300 DETIK)
                if setting.clean_welcome {
                    let bot_clone = bot.clone();
                    let chat_id = msg.chat.id;
                    let message_id = sent_msg.id;

                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(300)).await; // Nunggu 5 menit
                        let _ = bot_clone.delete_message(chat_id, message_id).await;
                    });
                }
            }
            Err(err) => {
                // 📊 TUI Dashboard: Kirim sinyal Error jika gagal menyambut
                dashboard_tx.send_error(Feature::Welcome, err.to_string());
            }
        }
    }

    Ok(())
}

/// 2. HANDLER SAAT MEMBER KELUAR (LEFT)
pub async fn handle_left_member(
    bot: Bot,
    msg: Message,
    pool: DbPool,
    dashboard_tx: DashboardSender,
) -> ResponseResult<()> {
    if msg.left_chat_member().is_none() {
        return Ok(());
    }

    // 📊 TUI Dashboard: Kirim sinyal Started
    dashboard_tx.send_start(
        Feature::Cleanwelcome,
        format!("Left member event in Chat ID: {}", msg.chat.id),
    );

    let setting = welcome_db::get_welcome_setting(&pool, msg.chat.id.0).await;

    // Jika clean_welcome aktif, LANGSUNG HAPUS pesan sistem "X meninggalkan grup" saat ini juga (0 detik)
    if setting.clean_welcome {
        match bot.delete_message(msg.chat.id, msg.id).await {
            Ok(_) => {
                dashboard_tx.send_success(
                    Feature::Cleanwelcome,
                    Some("Service message deleted"),
                );
            }
            Err(e) => {
                dashboard_tx.send_error(Feature::Cleanwelcome, e.to_string());
            }
        }
    } else {
        dashboard_tx.send_success(
            Feature::Cleanwelcome,
            Some("Cleanwelcome disabled"),
        );
    }

    Ok(())
}
