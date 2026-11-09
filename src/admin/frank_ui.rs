use reqwest::Client;
use serde_json::json;
use std::error::Error;
use teloxide::Bot;

// Bitmask Flags
const PERM_DELETE: u16 = 1 << 0;
const PERM_BAN: u16 = 1 << 1;
const PERM_PIN: u16 = 1 << 2;
const PERM_INVITE: u16 = 1 << 3;
const PERM_VOICE: u16 = 1 << 4;
const PERM_MANAGE: u16 = 1 << 5;
const PERM_PROMOTE: u16 = 1 << 6;
const PERM_INFO: u16 = 1 << 7;
const PERM_STORIES: u16 = 1 << 8;

// Metadata tuple: (bitmask_flag, full_name_for_table, short_code_for_button)
const PERMISSIONS: [(u16, &str, &str); 9] = [
    (PERM_DELETE, "🗑 Delete Messages", "🗑️ DEL"),
    (PERM_BAN, "🚫 Ban Users", "🚫 BAN"),
    (PERM_PIN, "📌 Pin Messages", "📌 PIN"),
    (PERM_INVITE, "✉️ Invite Users", "✉️ INVITE"),
    (PERM_VOICE, "🎙 Voice Chat", "🎙️ VOICE"),
    (PERM_MANAGE, "⚙️ Manage Chat", "⚙️ MANAGE"),
    (PERM_PROMOTE, "👑 Add Administrators", "👑 PROMOTE"),
    (PERM_INFO, "ℹ️ Change Info", "ℹ️ INFO"),
    (PERM_STORIES, "📱 Manage Stories", "📱 STORIES"),
];

pub async fn render_colored_permission_panel(
    bot: &Bot,
    log_chat_id: i64,
    msg_id: i32,
    target_chat_id: i64,
    target_id: u64,
    mask: u16,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let url = format!(
        "https://api.telegram.org/bot{}/editMessageText",
        bot.token()
    );

    let is_on = |flag: u16| (mask & flag) != 0;
    let tgl_cb = |flag: u16| format!("p_tgl:{}:{}:{}:{}", target_chat_id, target_id, mask, flag);

    // 1. Generate tabel sel dinamis
    let mut cells = vec![
        json!([{"text": "Permission", "is_header": true}, {"text": "Status", "is_header": true}]),
    ];
    for (flag, name, _) in PERMISSIONS {
        let status = if is_on(flag) {
            "🟢 Granted"
        } else {
            "🔴 Denied"
        };
        cells.push(json!([{"text": name}, {"text": status}]));
    }

    // 2. Generate 3x3 tombol dinamis
    let mut inline_keyboard: Vec<Vec<serde_json::Value>> = PERMISSIONS
        .chunks(3)
        .map(|chunk| {
            chunk
                .iter()
                .map(|&(flag, _, short)| {
                    let icon = if is_on(flag) { "🟢" } else { "🔴" };
                    let style = if is_on(flag) { "success" } else { "danger" };
                    json!({
                        "text": format!("{} {}", short, icon),
                        "callback_data": tgl_cb(flag),
                        "style": style
                    })
                })
                .collect()
        })
        .collect();

    // 3. Tambahkan baris kontrol (APPLY & CLOSE)
    inline_keyboard.push(vec![
        json!({ "text": "🚀 APPLY", "callback_data": format!("p_app:{}:{}:{}", target_chat_id, target_id, mask), "style": "primary" }),
        json!({ "text": "❌ CLOSE", "callback_data": "p_cls" }),
    ]);

    // 4. Payload Akhir
    let payload = json!({
        "chat_id": log_chat_id,
        "message_id": msg_id,
        "rich_message": {
            "blocks": [
                { "type": "heading", "text": "🛡️ ADMIN CONTROL CENTER", "size": 2 },
                { "type": "divider" },
                { "type": "heading", "text": "👤 ADMINISTRATOR", "size": 3 },
                { "type": "paragraph", "text": format!("Target: {}\nRole: Administrator 👑\nExpiration: Permanent ∞", target_id) },
                { "type": "divider" },
                { "type": "heading", "text": "⚙️ PERMISSION MATRIX", "size": 3 },
                { "type": "table", "is_bordered": true, "is_striped": true, "cells": cells },
                { "type": "divider" },
                { "type": "paragraph", "text": "⚠️ Permission changes are pending.\nUse APPLY to update this administrator." },
                { "type": "footer", "text": "Permission Engine • @entcorporate" }
            ]
        },
        "reply_markup": { "inline_keyboard": inline_keyboard }
    });

    let res = client.post(&url).json(&payload).send().await?;

    if !res.status().is_success() {
        let status = res.status();
        let body_text = res.text().await?;
        eprintln!(
            "❌ [TELEGRAM API ERROR]\nStatus: {}\nBody: {}",
            status, body_text
        );
        return Err(format!("Telegram API error {}: {}", status, body_text).into());
    }

    println!("✅ [RICH PERMISSION PANEL] Updated successfully");
    Ok(())
}
