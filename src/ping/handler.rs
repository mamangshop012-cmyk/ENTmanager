use super::metrics::get_bot_metrics;
use std::time::Instant;
use teloxide::{
    prelude::*,
    types::{Message, ParseMode, ReplyParameters},
};

pub async fn cmd_ping(
    bot: Bot,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Ukur Latency API Telegram (Round Trip Time)
    let start_time = Instant::now();
    let _ = bot.get_me().await;
    let latency_ms = start_time.elapsed().as_millis();

    // 2. Ambil data statistik bot
    let metrics = get_bot_metrics();

    // 3. Template Teks Balasan
    let response_text = format!(
        "🏓 <b>PONG!</b>\n\
        <blockquote>┌─ <b>BOT METRICS</b> ────\n\
         ⚡ <b>Latency API</b> : <code>{} ms</code>\n\
         🧠 <b>RAM Usage</b> : <code>{:.2} MB</code>\n\
         ⚙️ <b>CPU Usage</b> : <code>{:.2} %</code>\n\
         ⏱️ <b>Uptime Bot</b> : <code>{}</code>\n\
         └─────────────</blockquote>",
        latency_ms, metrics.ram_mb, metrics.cpu_percent, metrics.uptime_str
    );

    bot.send_message(msg.chat.id, response_text)
        .parse_mode(ParseMode::Html)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;

    Ok(())
}
