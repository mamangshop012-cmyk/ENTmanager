use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::sync::OnceLock;

use teloxide::{
    prelude::*,
    types::{CallbackQuery, Message},
};

use crate::moderation::db as mod_db;
use crate::terminal::{DashboardEvent, DashboardSender, LogKind};

// ==========================================================
// GLOBAL HTTP CLIENT
// ==========================================================
//
// Jangan bikin reqwest::Client baru setiap klik.
// Satu client dipakai berulang supaya connection pooling
// dan keep-alive bisa bekerja.
//

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static DASHBOARD_TX: OnceLock<DashboardSender> = OnceLock::new();

/// Register the live terminal dashboard sender once during bot startup.
/// Existing modstatus function signatures remain unchanged.
pub fn set_dashboard_sender(sender: DashboardSender) {
    let _ = DASHBOARD_TX.set(sender);
}

fn dashboard_log(kind: LogKind, message: impl Into<String>) {
    if let Some(sender) = DASHBOARD_TX.get() {
        let _ = sender.try_send(DashboardEvent::Log {
            kind,
            message: message.into(),
        });
    }
}


fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(8)
            .build()
            .expect("Gagal membuat HTTP client")
    })
}

// ==========================================================
// PAGE
// ==========================================================

#[derive(Clone, Copy, Debug)]
pub enum DashboardPage {
    Moderation = 1,
    BulkSlots = 2,
    Welcome = 3,
}

impl DashboardPage {
    pub fn from_number(page: i32) -> Self {
        match page {
            2 => DashboardPage::BulkSlots,
            3 => DashboardPage::Welcome,
            _ => DashboardPage::Moderation,
        }
    }

    pub fn number(&self) -> i32 {
        *self as i32
    }
}

// ==========================================================
// /modstatus
// ==========================================================
//
// ADMIN CHECK DI SINI TETAP ADA.
//
// Karena command /modstatus masuk melalui PintuGuard,
// command ini hanya akan sampai ke sini kalau user sudah
// lolos permission.
//
// ==========================================================

pub async fn cmd_modstatus(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dashboard_log(LogKind::Action, format!("[MODSTATUS] chat={}", msg.chat.id.0));
    send_modstatus_page(&bot, &pool, msg.chat.id.0, DashboardPage::Moderation).await?;
    Ok(())
}

// ==========================================================
// CALLBACK DIRECT
// ==========================================================
//
// Tetap dipertahankan agar kode lama tidak rusak.
//
// Jangan daftarkan ini sebagai callback tambahan kalau
// dispatcher utama sudah memakai process_modstatus_callback().
//

pub async fn cmd_modstatus_callback(
    bot: Bot,
    q: CallbackQuery,
    pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    process_modstatus_callback(&bot, &q, &pool).await
}

// ==========================================================
// SHARED CALLBACK PROCESSOR
// ==========================================================

pub async fn process_modstatus_callback(
    bot: &Bot,
    q: &CallbackQuery,
    pool: &SqlitePool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let data = match q.data.as_deref() {
        Some(data) if data.starts_with("modstatus:") => data,
        _ => return Ok(()),
    };

    // ======================================================
    // NOOP
    // ======================================================

    if data == "modstatus:noop" {
        let _ = bot.answer_callback_query(q.id.clone()).await;
        return Ok(());
    }

    // ======================================================
    // PARSE PAGE
    // ======================================================

    let page_number = data
        .strip_prefix("modstatus:")
        .unwrap_or("1")
        .parse::<i32>()
        .unwrap_or(1)
        .clamp(1, 3);

    let page = DashboardPage::from_number(page_number);

    // ======================================================
    // MESSAGE
    // ======================================================

    let message = match q.message.as_ref() {
        Some(message) => message,
        None => {
            let _ = bot.answer_callback_query(q.id.clone()).await;
            return Ok(());
        }
    };

    let chat_id = message.chat().id.0;
    let message_id = message.id().0;

    // ======================================================
    // ⚡ FAST PATH
    // ======================================================
    //
    // TIDAK ADA:
    // bot.get_chat_member(...)
    // karena modstatus adalah dashboard READ-ONLY.
    //
    // Admin sudah dicek ketika /modstatus dijalankan
    // melalui PintuGuard.
    //
    // ======================================================

    // Jalankan answer callback dan edit message secara
    // bersamaan supaya request tidak saling menunggu.

    let answer_future = bot.answer_callback_query(q.id.clone());
    let edit_future = edit_modstatus_page(bot, pool, chat_id, message_id, page);

    let (_answer_result, edit_result) = tokio::join!(answer_future, edit_future);
    edit_result?;

    Ok(())
}

// ==========================================================
// SEND PAGE
// ==========================================================

pub async fn send_modstatus_page(
    bot: &Bot,
    pool: &SqlitePool,
    chat_id: i64,
    page: DashboardPage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rich_html = build_page_html(pool, chat_id, page).await?;
    let keyboard = build_navigation_keyboard(page);
    let url = format!(
        "https://api.telegram.org/bot{}/sendRichMessage",
        bot.token()
    );

    let body = json!({
        "chat_id": chat_id,
        "rich_message": { "html": rich_html },
        "reply_markup": keyboard
    });

    let response = http_client().post(&url).json(&body).send().await?;
    let status = response.status();

    if !status.is_success() {
        let text = response.text().await?;
        dashboard_log(LogKind::Error, format!("[MODSTATUS SEND] HTTP {} {}", status, text));
        return Ok(());
    }

    let parsed: Value = response.json().await?;
    let ok = parsed.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

    if !ok {
        dashboard_log(LogKind::Error, format!("[MODSTATUS SEND] Telegram error: {}", parsed));
    }

    Ok(())
}

// ==========================================================
// EDIT PAGE
// ==========================================================

pub async fn edit_modstatus_page(
    bot: &Bot,
    pool: &SqlitePool,
    chat_id: i64,
    message_id: i32,
    page: DashboardPage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // ======================================================
    // BUILD
    // ======================================================

    let rich_html = build_page_html(pool, chat_id, page).await?;
    let keyboard = build_navigation_keyboard(page);

    // ======================================================
    // TELEGRAM URL
    // ======================================================

    let url = format!(
        "https://api.telegram.org/bot{}/editMessageText",
        bot.token()
    );

    let body = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "rich_message": { "html": rich_html },
        "reply_markup": keyboard
    });

    // ======================================================
    // REQUEST
    // ======================================================

    let response = http_client().post(&url).json(&body).send().await?;
    let status = response.status();

    // Jangan print seluruh JSON response setiap klik.
    // Ini jauh lebih ringan dan terminal tidak spam.

    if !status.is_success() {
        let text = response.text().await?;
        dashboard_log(LogKind::Error, format!("[MODSTATUS EDIT] HTTP {} {}", status, text));
        return Ok(());
    }

    // Parse hanya untuk memastikan Telegram menerima request.

    let parsed: Value = response.json().await?;
    let ok = parsed.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

    if !ok {
        dashboard_log(LogKind::Error, format!("[MODSTATUS EDIT] Telegram error: {}", parsed));
    }

    Ok(())
}

// ==========================================================
// NAVIGATION KEYBOARD
// ==========================================================

fn build_navigation_keyboard(page: DashboardPage) -> Value {
    let current = page.number();
    let previous = if current <= 1 { 3 } else { current - 1 };
    let next = if current >= 3 { 1 } else { current + 1 };

    json!({
        "inline_keyboard": [
            [
                { "text": "‹ Back", "callback_data": format!("modstatus:{}", previous) },
                { "text": format!("{} / 3", current), "callback_data": "modstatus:noop" },
                { "text": "Next ›", "callback_data": format!("modstatus:{}", next) }
            ]
        ]
    })
}

// ==========================================================
// BUILD PAGE
// ==========================================================

async fn build_page_html(
    pool: &SqlitePool,
    chat_id: i64,
    page: DashboardPage,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match page {
        DashboardPage::Moderation => build_moderation_page(pool, chat_id).await,
        DashboardPage::BulkSlots => build_bulk_slots_page(pool, chat_id).await,
        DashboardPage::Welcome => build_welcome_page(pool, chat_id).await,
    }
}

// ==========================================================
// PAGE 1
// MODERATION
// ==========================================================

async fn build_moderation_page(
    pool: &SqlitePool,
    chat_id: i64,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // ======================================================
    // ⚡ 3 QUERY DATABASE DIJALANKAN BERSAMAAN
    // ======================================================

    let (settings_result, blacklist_result, allowlist_result) = tokio::join!(
        mod_db::get_or_create_warn_settings(pool, chat_id),
        mod_db::get_blacklist(pool, chat_id),
        mod_db::get_allowlist(pool, chat_id)
    );

    let settings = settings_result?;
    let blacklist = blacklist_result?;
    let allowlist = allowlist_result.unwrap_or_default();

    // ======================================================
    // SETTINGS
    // ======================================================

    let lockwarns = if settings.lock_warns != 0 {
        "Enabled"
    } else {
        "Disabled"
    };
    let clean = settings
        .clean_interval
        .clone()
        .unwrap_or_else(|| "Disabled".to_string());
    let log = settings
        .log_channel_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "Disabled".to_string());

    Ok(format!(
        r#"<h1>🛡️ MODERATION</h1>

<p>🟢 <b>Status:</b> <code>ACTIVE</code></p>

<table bordered striped>

<tr>
<td><b>Setting</b></td>
<td align="right"><b>Value</b></td>
</tr>

<tr>
<td>⚠️ Warn Limit</td>
<td align="right"><code>{} / warns</code></td>
</tr>

<tr>
<td>🔨 Punishment</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>🧹 Auto-Clean</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>⚠️ Lockwarns</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>🪵 Log Channel</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>🌐 Allowlist</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>🚫 Blacklist</td>
<td align="right"><code>{}</code></td>
</tr>

</table>

<footer>MODERATION CONFIG •</footer> @entcorporate"#,
        settings.warn_limit,
        settings.warn_action.to_uppercase(),
        clean,
        lockwarns,
        log,
        allowlist.len(),
        blacklist.len(),
    ))
}

// ==========================================================
// PAGE 2
// BULK SLOTS
// ==========================================================

async fn build_bulk_slots_page(
    pool: &SqlitePool,
    chat_id: i64,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let slots = mod_db::get_bulk_slots(pool, chat_id)
        .await
        .unwrap_or_default();

    let slot1 = slots.get(&1).cloned().unwrap_or_else(|| "Off".to_string());
    let slot2 = slots.get(&2).cloned().unwrap_or_else(|| "Off".to_string());
    let slot3 = slots.get(&3).cloned().unwrap_or_else(|| "Off".to_string());
    let slot4 = slots.get(&4).cloned().unwrap_or_else(|| "Off".to_string());
    let slot5 = slots.get(&5).cloned().unwrap_or_else(|| "Off".to_string());

    Ok(format!(
        r#"<h1>📦 BULK SLOTS</h1>

<p>⚙️ <b>Blacklist punishment profiles</b></p>

<table bordered striped>

<tr>
<td><b>Slot</b></td>
<td align="right"><b>Action</b></td>
</tr>

<tr>
<td>📦 S1</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>📦 S2</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>📦 S3</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>📦 S4</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>📦 S5</td>
<td align="right"><code>{}</code></td>
</tr>

</table>

<footer>BULK SLOT CONFIG • managerRUSTent</footer>"#,
        slot1, slot2, slot3, slot4, slot5,
    ))
}

// ==========================================================
// PAGE 3
// WELCOME
// ==========================================================

async fn build_welcome_page(
    pool: &SqlitePool,
    chat_id: i64,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // ======================================================
    // ⚡ 2 QUERY DATABASE BERSAMAAN
    // ======================================================

    let (welcome_result, locks_result) = tokio::join!(
        mod_db::get_welcome_info(pool, chat_id),
        mod_db::get_active_locks(pool, chat_id)
    );

    let (welcome_status, welcome_clean) =
        welcome_result.unwrap_or(("Disabled".to_string(), "Off".to_string()));

    let active_locks = locks_result?;

    // ======================================================
    // LOCK HTML
    // ======================================================

    let locks_html = if active_locks.is_empty() {
        "<i>No active locks</i>".to_string()
    } else {
        active_locks
            .iter()
            .map(|lock| format!("🔒 <code>{}</code>", lock))
            .collect::<Vec<_>>()
            .join("\n")
    };

    Ok(format!(
        r#"<h1>👋 WELCOME</h1>

<table bordered striped>

<tr>
<td><b>Setting</b></td>
<td align="right"><b>Status</b></td>
</tr>

<tr>
<td>👋 Welcome</td>
<td align="right"><code>{}</code></td>
</tr>

<tr>
<td>🧹 Auto-Clean</td>
<td align="right"><code>{}</code></td>
</tr>

</table>

<h2>🔒 ACTIVE LOCKS ({})</h2>

<blockquote>
{}
</blockquote>

<footer>WELCOME & LOCKS • managerRUSTent</footer>"#,
        welcome_status,
        welcome_clean,
        active_locks.len(),
        locks_html,
    ))
}

// ==========================================================
// EPHEMERAL WHISPER
// ==========================================================

pub async fn send_ephemeral_whisper(
    bot: &Bot,
    chat_id: i64,
    receiver_user_id: i64,
    callback_query_id: Option<&str>,
    text: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot.token());

    let mut body = json!({
        "chat_id": chat_id,
        "receiver_user_id": receiver_user_id,
        "text": text,
        "parse_mode": "HTML"
    });

    if let Some(callback_id) = callback_query_id {
        body["callback_query_id"] = json!(callback_id);
    }

    let response = http_client().post(&url).json(&body).send().await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        dashboard_log(LogKind::Error, format!("[EPHEMERAL] HTTP error: {}", text));
    }

    Ok(())
}
