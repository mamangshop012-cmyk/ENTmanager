use reqwest::Client;
use serde_json::{json, Value};
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, ParseMode},
    Bot, RequestError,
};

// ============================================================
// CONFIG
// ============================================================

const CHANNEL_URL: &str = "https://t.me/entcorporate";
const CHANNEL_NAME: &str = "@entcorporate";

// Effect hanya untuk private chat.
// Kosongkan "" untuk menonaktifkan.
const START_MESSAGE_EFFECT_ID: &str = "5104841245755180586";

// ============================================================
// KEYBOARDS
// ============================================================

pub fn make_start_keyboard(bot_username: &str, channel_url: &str) -> InlineKeyboardMarkup {
    let add_to_group_url = format!("https://t.me/{}?startgroup=true", bot_username);

    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::url(
            "➕ Tambahkan ke Grup",
            add_to_group_url.parse().unwrap(),
        )],
        vec![InlineKeyboardButton::callback("ℹ️ About Me", "show_about")],
        vec![InlineKeyboardButton::url(
            "📢 Channel Pembuat",
            channel_url.parse().unwrap(),
        )],
    ])
}

pub fn make_group_start_keyboard(bot_username: &str) -> InlineKeyboardMarkup {
    let pm_url = format!("https://t.me/{}", bot_username);

    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::url(
        "💬 Kelola di PM",
        pm_url.parse().unwrap(),
    )]])
}

pub fn make_about_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🔙 Kembali ke Menu Utama",
        "show_start",
    )]])
}

// ============================================================
// RICH HTML
// ============================================================

pub fn get_start_rich_html(first_name: &str, bot_name: &str) -> String {
    format!(
        r#"
<h1>🦀 {bot_name}</h1>

<p>Halo, <b>{first_name}</b>! 👋</p>

<p>
<b>{bot_name}</b> adalah bot <b>Group Management &amp; Security</b>
untuk Telegram.
</p>

<p>
🛡️ Aman • ⚡ Cepat • 🧩 Modular
</p>

<details>
<summary>🛡️ <b>Lihat Fitur Utama</b></summary>

<p>
🛡️ <b>Moderation</b> — Ban, mute, kick, warn, dan tindakan otomatis.
</p>

<p>
🔒 <b>Security &amp; Locks</b> — Kontrol link, media, bot, sticker,
dan berbagai tipe konten.
</p>

<p>
🚫 <b>Blacklist &amp; Filter</b> — Deteksi kata terlarang,
auto-response, dan action otomatis.
</p>

<p>
👋 <b>Welcome System</b> — Sambutan anggota baru yang dapat dikustomisasi.
</p>

<p>
⚙️ <b>Admin Tools</b> — Warning, locks, permission,
dan konfigurasi grup.
</p>

</details>

<details>
<summary>⚡ <b>Lihat Performa</b></summary>

<p>
Dibangun menggunakan <b>Rust 🦀</b> dengan arsitektur async,
SQLite, dan struktur modular.
</p>

<p>
⚡ Respons cepat<br/>
🧠 Resource efisien<br/>
🔄 Async processing<br/>
🗄️ SQLite database<br/>
🧩 Modular architecture
</p>

</details>

<details>
<summary>🔧 <b>Lihat Teknologi</b></summary>

<p>
🦀 <b>Rust</b> — Core application &amp; moderation engine
</p>

<p>
🤖 <b>Teloxide</b> — Telegram bot framework
</p>

<p>
🗄️ <b>SQLx + SQLite</b> — Database layer
</p>

<p>
🌐 <b>Reqwest</b> — Direct Telegram Bot API access
</p>

<p>
🏗️ <b>Modular Architecture</b> — Moderation, filters,
welcome, database, admin, dan security.
</p>

</details>

<hr/>

<p>
🚀 <b>Mulai sekarang:</b><br/>
Tambahkan gue ke grup untuk mulai mengelolanya.
</p>

<footer>
⚡ Powered by <a href="https://t.me/entcorporate">ENTcore</a> • © 2026
</footer>
"#
    )
}

pub fn get_group_start_rich_html(first_name: &str, bot_name: &str) -> String {
    format!(
        r#"
<h1>🦀 {bot_name}</h1>

<p>Halo <b>{first_name}</b>! 👋</p>

<p>
<b>{bot_name}</b> sudah aktif di grup ini dan siap membantu mengamankannya.
</p>

<hr/>

<h3>⚙️ PENGELOLAAN BOT</h3>

<p>
Konfigurasi moderation, locks, blacklist, welcome,
permission, dan fitur admin dilakukan melalui
<b>Private Message (PM)</b>.
</p>

<ul>
<li>🛡️ Kelola moderation</li>
<li>🔒 Atur locks &amp; security</li>
<li>🚫 Kelola blacklist &amp; filter</li>
<li>👋 Konfigurasi welcome</li>
<li>⚙️ Atur permission &amp; setting</li>
</ul>

<hr/>

<p>
💬 Klik tombol di bawah untuk membuka pengelolaan di PM.
</p>

<footer>
⚡ Powered by <a href="https://t.me/entcorporate">ENTcore</a> • © 2026
</footer>
"#
    )
}

pub fn get_about_rich_html(channel_name: &str) -> String {
    format!(
        r#"
<h1>ℹ️ ABOUT ME</h1>

<p>
<b>ManagerRUSTent</b> adalah Telegram
<b>Group Management &amp; Security Bot</b>.
</p>

<p>
Dibuat untuk membantu admin menangani
moderation, security, automation, dan konfigurasi grup.
</p>

<hr/>

<h3>🦀 TECH STACK</h3>

<ul>
<li><b>Rust</b> — Core application</li>
<li><b>Teloxide</b> — Telegram bot framework</li>
<li><b>Reqwest</b> — Direct Bot API access</li>
<li><b>SQLx</b> — Async database layer</li>
<li><b>SQLite</b> — Persistent database</li>
</ul>

<hr/>

<h3>🏗️ ARCHITECTURE</h3>

<ul>
<li>Async runtime</li>
<li>Domain-based modular structure</li>
<li>Centralized database layer</li>
<li>Separated moderation modules</li>
<li>Direct HTTP access for newer Bot API features</li>
</ul>

<hr/>

<p>
💡 Developer: <b>{channel_name}</b>
</p>

<footer>
⚡ Powered by <a href="https://t.me/entcorporate">ENTcore</a> • © 2026
</footer>
"#
    )
}

// ============================================================
// TELEGRAM HTTP
// ============================================================

async fn telegram_api(
    bot: &Bot,
    method: &str,
    payload: &Value,
) -> Result<(bool, Value), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let url = format!("https://api.telegram.org/bot{}/{}", bot.token(), method);

    log::debug!("➡️ TELEGRAM {} REQUEST = {}", method, payload);

    let response = client.post(&url).json(payload).send().await?;

    let status = response.status();
    let response_text = response.text().await?;

    log::debug!(
        "⬅️ TELEGRAM {} RESPONSE = {} | {}",
        method,
        status,
        response_text
    );

    let parsed = match serde_json::from_str::<Value>(&response_text) {
        Ok(value) => value,
        Err(error) => {
            log::error!(
                "❌ Telegram response bukan JSON: {} | body={}",
                error,
                response_text
            );

            return Ok((
                false,
                json!({
                    "ok": false,
                    "description": format!(
                        "Invalid Telegram JSON response: {}",
                        error
                    ),
                    "raw": response_text
                }),
            ));
        }
    };

    let ok = parsed.get("ok").and_then(Value::as_bool).unwrap_or(false);

    Ok((ok, parsed))
}

// ============================================================
// SEND RICH MESSAGE
// ============================================================

async fn send_rich_html(
    bot: &Bot,
    chat_id: i64,
    html: String,
    keyboard: InlineKeyboardMarkup,
    message_effect_id: Option<&str>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = json!({
        "chat_id": chat_id,
        "rich_message": {
            "html": html
        },
        "reply_markup": serde_json::to_value(keyboard)?
    });

    if let Some(effect_id) = message_effect_id {
        if !effect_id.is_empty() {
            payload["message_effect_id"] = json!(effect_id);
        }
    }

    let (ok, response) = telegram_api(bot, "sendRichMessage", &payload).await?;

    if !ok {
        log::warn!("❌ sendRichMessage ditolak Telegram: {}", response);
    }

    Ok(ok)
}

// ============================================================
// EDIT RICH MESSAGE
// ============================================================

async fn edit_rich_html(
    bot: &Bot,
    chat_id: i64,
    message_id: i32,
    html: String,
    keyboard: InlineKeyboardMarkup,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "rich_message": {
            "html": html
        },
        "reply_markup": serde_json::to_value(keyboard)?
    });

    let (ok, response) = telegram_api(bot, "editMessageText", &payload).await?;

    if !ok {
        log::warn!("❌ editMessageText Rich ditolak Telegram: {}", response);
    }

    Ok(ok)
}

// ============================================================
// FALLBACK TEXT
// ============================================================

fn get_start_fallback_text(first_name: &str, bot_name: &str) -> String {
    format!(
        "<b>🦀 {}</b>\n\n\
         Halo, <b>{}</b>! 👋\n\n\
         Gue adalah <b>{}</b>, bot Group Management & Security.\n\n\
         🛡️ Moderation\n\
         🔒 Security & Locks\n\
         🚫 Blacklist & Filter\n\
         👋 Welcome System\n\
         ⚙️ Admin Tools\n\n\
         🦀 Powered by Rust.",
        bot_name, first_name, bot_name
    )
}

fn get_group_start_fallback_text(first_name: &str, bot_name: &str) -> String {
    format!(
        "<b>🦀 {}</b>\n\n\
         Halo <b>{}</b>! 👋\n\n\
         Bot sudah aktif di grup ini.\n\n\
         ⚙️ Konfigurasi moderation, security, welcome,\n\
         blacklist, locks, dan admin tools dilakukan melalui PM.",
        bot_name, first_name
    )
}

fn get_about_fallback_text(channel_name: &str) -> String {
    format!(
        "<b>ℹ️ ABOUT ME</b>\n\n\
         <b>ManagerRUSTent</b>\n\
         Group Management & Security Bot\n\n\
         🦀 Rust — Core application\n\
         🤖 Teloxide — Telegram framework\n\
         🌐 Reqwest — Direct Bot API\n\
         🗄️ SQLx + SQLite — Database\n\n\
         💡 Developer: <b>{}</b>",
        channel_name
    )
}

// ============================================================
// START HANDLER
// ============================================================

pub async fn handle_start(bot: Bot, msg: Message) -> Result<(), RequestError> {
    let me = bot.get_me().await?;

    let bot_username = me.username();
    let bot_name = me.first_name.clone();

    let first_name = msg
        .from
        .as_ref()
        .map_or("User", |user| user.first_name.as_str());

    if msg.chat.is_private() {
        let keyboard = make_start_keyboard(bot_username, CHANNEL_URL);

        let html = get_start_rich_html(first_name, &bot_name);

        let result = send_rich_html(
            &bot,
            msg.chat.id.0,
            html,
            keyboard.clone(),
            Some(START_MESSAGE_EFFECT_ID),
        )
        .await;

        match result {
            Ok(true) => {
                log::info!("✅ Rich /start berhasil dikirim: {}", msg.chat.id);
            }

            Ok(false) | Err(_) => {
                log::warn!("⚠️ Rich /start gagal -> fallback regular message");

                bot.send_message(msg.chat.id, get_start_fallback_text(first_name, &bot_name))
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;
            }
        }

        return Ok(());
    }

    // --------------------------------------------------------
    // GROUP
    // --------------------------------------------------------

    let keyboard = make_group_start_keyboard(bot_username);

    let html = get_group_start_rich_html(first_name, &bot_name);

    let result = send_rich_html(&bot, msg.chat.id.0, html, keyboard.clone(), None).await;

    match result {
        Ok(true) => {
            log::info!("✅ Rich group /start berhasil dikirim: {}", msg.chat.id);
        }

        Ok(false) | Err(_) => {
            log::warn!("⚠️ Rich group /start gagal -> fallback regular message");

            bot.send_message(
                msg.chat.id,
                get_group_start_fallback_text(first_name, &bot_name),
            )
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
        }
    }

    Ok(())
}

// ============================================================
// CALLBACK HANDLER
// ============================================================

pub async fn handle_callback(bot: Bot, q: CallbackQuery) -> Result<(), RequestError> {
    log::info!("🔘 START CALLBACK RECEIVED: data={:?}, id={}", q.data, q.id);

    // Answer secepat mungkin supaya tombol berhenti loading.
    bot.answer_callback_query(&q.id).await?;

    log::info!("✅ CALLBACK ANSWERED: data={:?}", q.data);

    let me = bot.get_me().await?;

    let bot_username = me.username();
    let bot_name = me.first_name.clone();

    let Some(data) = q.data.as_deref() else {
        log::warn!("⚠️ Callback tidak memiliki data");
        return Ok(());
    };

    let Some(msg) = q.message else {
        log::warn!("⚠️ Callback tidak memiliki message");
        return Ok(());
    };

    let chat_id = msg.chat().id;
    let message_id = msg.id();

    let first_name = q.from.first_name.as_str();

    log::info!(
        "🔘 CALLBACK MATCH DATA = {} | chat={} | message={}",
        data,
        chat_id,
        message_id
    );

    match data {
        // ====================================================
        // ABOUT ME
        // ====================================================
        "show_about" => {
            log::info!("ℹ️ ABOUT BUTTON CLICKED");

            let keyboard = make_about_keyboard();

            let html = get_about_rich_html(CHANNEL_NAME);

            log::info!("➡️ STARTING RICH ABOUT EDIT");

            let result = edit_rich_html(
                &bot,
                chat_id.0,
                message_id.0,
                html.clone(),
                keyboard.clone(),
            )
            .await;

            log::info!("⬅️ RICH ABOUT RESULT = {:?}", result);

            match result {
                Ok(true) => {
                    log::info!("✅ ABOUT berhasil diubah ke Rich Message");
                }

                Ok(false) | Err(_) => {
                    log::warn!("⚠️ Rich About gagal -> fallback editMessageText");

                    bot.edit_message_text(
                        chat_id,
                        message_id,
                        get_about_fallback_text(CHANNEL_NAME),
                    )
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;

                    log::info!("✅ ABOUT fallback berhasil");
                }
            }
        }

        // ====================================================
        // BACK TO START
        // ====================================================
        "show_start" => {
            log::info!("🏠 BACK TO START BUTTON CLICKED");

            let keyboard = make_start_keyboard(bot_username, CHANNEL_URL);

            let html = get_start_rich_html(first_name, &bot_name);

            let result = edit_rich_html(
                &bot,
                chat_id.0,
                message_id.0,
                html.clone(),
                keyboard.clone(),
            )
            .await;

            log::info!("⬅️ RICH START RESULT = {:?}", result);

            match result {
                Ok(true) => {
                    log::info!("✅ START berhasil dikembalikan ke Rich Message");
                }

                Ok(false) | Err(_) => {
                    log::warn!("⚠️ Rich Start gagal -> fallback");

                    bot.edit_message_text(
                        chat_id,
                        message_id,
                        get_start_fallback_text(first_name, &bot_name),
                    )
                    .parse_mode(ParseMode::Html)
                    .reply_markup(keyboard)
                    .await?;

                    log::info!("✅ START fallback berhasil");
                }
            }
        }

        _ => {
            log::debug!("ℹ️ Callback bukan milik start.rs: {}", data);
        }
    }

    Ok(())
}
