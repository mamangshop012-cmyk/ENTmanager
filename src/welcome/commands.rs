use crate::db::DbPool;
use crate::pintuguard::{self, Permission};
use crate::welcome::db as welcome_db;
use crate::welcome::formatter::telegram_entities_to_html;
use crate::welcome::parser::parse_welcome_message;
use teloxide::prelude::*;
use teloxide::types::InputFile;

/// 🛡️ HELPER BENTENG 1 UNTUK HANDLER WELCOME
/// Meneruskan pengecekan ke Pintuguard & melakukan Silent Delete jika member biasa.
async fn guard_admin_or_owner(bot: &Bot, msg: &Message) -> bool {
    let Some(user) = msg.from.as_ref() else {
        return false;
    };

    // Panggil Benteng 1 Pintuguard
    if pintuguard::check_permission(bot, msg.chat.id, user.id, Permission::ManageChat)
        .await
        .is_err()
    {
        // 🛑 SILENT DELETE: Hapus pesan member biasa secara diam-diam
        let _ = bot.delete_message(msg.chat.id, msg.id).await;
        return false;
    }

    true
}

/// Handler untuk perintah `/setwelcome <teks>` (Mendukung Media Foto, GIF, & Video)
pub async fn handle_set_welcome(bot: Bot, msg: Message, pool: DbPool) -> ResponseResult<()> {
    if !guard_admin_or_owner(&bot, &msg).await {
        return Ok(());
    }

    let mut media_type: Option<String> = None;
    let mut file_id: Option<String> = None;

    // Extract text & entities berdasarkan kondisi pesan (Reply, Caption, atau Teks Biasa)
    let (raw_text, entities) = if let Some(reply_msg) = msg.reply_to_message() {
        // 1. CARA A: Admin membalas (reply) ke pesan Foto / GIF / Video
        if let Some(photos) = reply_msg.photo() {
            if let Some(photo) = photos.last() {
                media_type = Some("photo".to_string());
                file_id = Some(photo.file.id.to_string());
            }
        } else if let Some(anim) = reply_msg.animation() {
            media_type = Some("animation".to_string());
            file_id = Some(anim.file.id.to_string());
        } else if let Some(video) = reply_msg.video() {
            media_type = Some("video".to_string());
            file_id = Some(video.file.id.to_string());
        }

        (
            msg.text().unwrap_or_default(),
            msg.entities().unwrap_or(&[]),
        )
    } else if msg.caption().is_some() {
        // 2. CARA B: Admin mengunggah media langsung dengan caption /setwelcome ...
        if let Some(photos) = msg.photo() {
            if let Some(photo) = photos.last() {
                media_type = Some("photo".to_string());
                file_id = Some(photo.file.id.to_string());
            }
        } else if let Some(anim) = msg.animation() {
            media_type = Some("animation".to_string());
            file_id = Some(anim.file.id.to_string());
        } else if let Some(video) = msg.video() {
            media_type = Some("video".to_string());
            file_id = Some(video.file.id.to_string());
        }

        (
            msg.caption().unwrap_or_default(),
            msg.caption_entities().unwrap_or(&[]),
        )
    } else {
        // 3. CARA C: Pesan teks biasa
        (
            msg.text().unwrap_or_default(),
            msg.entities().unwrap_or(&[]),
        )
    };

    // Convert formatting UI Telegram (Bold, Italic, Spoiler, dll) menjadi HTML string
    let full_html = telegram_entities_to_html(raw_text, entities);

    // Bersihkan prefix command `/setwelcome` atau `/setwelcome@botname` dari teks
    let custom_text = if let Some(first_word) = full_html.split_whitespace().next() {
        if first_word.starts_with("/setwelcome") {
            full_html.trim_start()[first_word.len()..]
                .trim()
                .to_string()
        } else {
            full_html.trim().to_string()
        }
    } else {
        full_html.trim().to_string()
    };

    if custom_text.trim().is_empty() {
        bot.send_message(
            msg.chat.id,
            "⚠️ <b>Format salah!</b> Berikan teks welcome.\n\n<b>Contoh:</b>\n<code>/setwelcome Selamat datang {first} di {chatname}!\n[Google](buttonurl://google.com)</code>",
        )
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
        return Ok(());
    }

    // Simpan ke SQLite (dengan media jika ada, atau teks murni)
    let save_result = if media_type.is_some() && file_id.is_some() {
        welcome_db::set_welcome_media(
            &pool,
            msg.chat.id.0,
            &custom_text,
            media_type.as_deref(),
            file_id.as_deref(),
        )
        .await
    } else {
        welcome_db::set_welcome_text(&pool, msg.chat.id.0, &custom_text).await
    };

    if let Err(e) = save_result {
        log::error!("Gagal menyimpan welcome text/media: {:?}", e);
        bot.send_message(msg.chat.id, "❌ Gagal menyimpan pesan welcome ke database.")
            .await?;
        return Ok(());
    }

    let media_info = match media_type.as_deref() {
        Some("photo") => " (dengan Foto)",
        Some("animation") => " (dengan GIF)",
        Some("video") => " (dengan Video)",
        _ => "",
    };

    bot.send_message(
        msg.chat.id,
        format!(
            "✅ Pesan welcome kustom berhasil disimpan & diaktifkan{}!",
            media_info
        ),
    )
    .await?;

    Ok(())
}

/// Handler untuk perintah `/cleanwelcome [on|off]`
pub async fn handle_clean_welcome(bot: Bot, msg: Message, pool: DbPool) -> ResponseResult<()> {
    if !guard_admin_or_owner(&bot, &msg).await {
        return Ok(());
    }

    let text = msg.text().unwrap_or_default();
    let arg = text
        .trim_start_matches("/cleanwelcome")
        .trim()
        .to_lowercase();

    let enable = match arg.as_str() {
        "on" | "enable" | "1" => true,
        "off" | "disable" | "0" => false,
        _ => {
            bot.send_message(
                msg.chat.id,
                "⚠️ Format salah! Gunakan:\n• <code>/cleanwelcome on</code> (Hapus pesan welcome otomatis setelah 5 menit)\n• <code>/cleanwelcome off</code> (Matikan auto-delete)",
            )
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
            return Ok(());
        }
    };

    if let Err(e) = welcome_db::toggle_clean_welcome(&pool, msg.chat.id.0, enable).await {
        log::error!("Gagal mengupdate clean welcome: {:?}", e);
        bot.send_message(msg.chat.id, "❌ Gagal mengupdate pengaturan clean welcome.")
            .await?;
        return Ok(());
    }

    let status_msg = if enable {
        "✅ <b>Clean Welcome AKTIF!</b> Pesan welcome akan otomatis dihapus setelah 5 menit."
    } else {
        "🔴 <b>Clean Welcome NONAKTIF!</b> Pesan welcome tidak akan dihapus otomatis."
    };

    bot.send_message(msg.chat.id, status_msg)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}

/// Handler untuk perintah `/welcome` (Cek status / preview / toggle ON/OFF)
pub async fn handle_welcome_toggle(bot: Bot, msg: Message, pool: DbPool) -> ResponseResult<()> {
    let text = msg.text().unwrap_or_default();
    let arg = text.trim_start_matches("/welcome").trim().to_lowercase();

    let setting = welcome_db::get_welcome_setting(&pool, msg.chat.id.0).await;

    // Jika cuma mengetik `/welcome` tanpa argumen, tampilkan status & preview
    if arg.is_empty() {
        let status_str = if setting.is_enabled {
            "AKTIF 🟢"
        } else {
            "NONAKTIF 🔴"
        };
        let clean_str = if setting.clean_welcome {
            "AKTIF (5 Menit) ⏳"
        } else {
            "NONAKTIF ❌"
        };

        let user = match msg.from.as_ref() {
            Some(u) => u.clone(),
            None => return Ok(()),
        };
        let chat_title = msg.chat.title().unwrap_or("Grup Ini");

        let (preview_text, markup) =
            parse_welcome_message(&setting.welcome_text, &user, chat_title);

        let info = format!(
            "<b>Status Welcome:</b> {}\n<b>Clean Welcome:</b> {}\n\n<b>Preview Pesan:</b>\n{}",
            status_str, clean_str, preview_text
        );

        // Jika ada media tersimpan, tampilkan preview beserta medianya
        if let (Some(m_type), Some(f_id)) = (&setting.media_type, &setting.file_id) {
            let input_file = InputFile::file_id(f_id.clone());

            match m_type.as_str() {
                "photo" => {
                    bot.send_photo(msg.chat.id, input_file)
                        .caption(info)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(markup)
                        .await?;
                }
                "animation" => {
                    bot.send_animation(msg.chat.id, input_file)
                        .caption(info)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(markup)
                        .await?;
                }
                "video" => {
                    bot.send_video(msg.chat.id, input_file)
                        .caption(info)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(markup)
                        .await?;
                }
                _ => {
                    bot.send_message(msg.chat.id, info)
                        .parse_mode(teloxide::types::ParseMode::Html)
                        .reply_markup(markup)
                        .await?;
                }
            }
        } else {
            bot.send_message(msg.chat.id, info)
                .parse_mode(teloxide::types::ParseMode::Html)
                .reply_markup(markup)
                .await?;
        }

        return Ok(());
    }

    // Jika mengetik `/welcome on` atau `/welcome off`, butuh izin Admin
    if !guard_admin_or_owner(&bot, &msg).await {
        return Ok(());
    }

    let enable = match arg.as_str() {
        "on" | "enable" | "1" => true,
        "off" | "disable" | "0" => false,
        _ => return Ok(()),
    };

    if let Err(e) = welcome_db::toggle_welcome(&pool, msg.chat.id.0, enable).await {
        log::error!("Gagal toggle welcome: {:?}", e);
        bot.send_message(msg.chat.id, "❌ Gagal mengupdate status welcome.")
            .await?;
        return Ok(());
    }

    let status_msg = if enable {
        "✅ <b>Pesan Welcome diaktifkan!</b>"
    } else {
        "🔴 <b>Pesan Welcome dimatikan!</b>"
    };

    bot.send_message(msg.chat.id, status_msg)
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;

    Ok(())
}
