use teloxide::types::{MessageEntity, MessageEntityKind};

/// Melakukan HTML escape pada karakter khusus
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Mengonversi teks + Telegram Entities UI menjadi String HTML
pub fn telegram_entities_to_html(text: &str, entities: &[MessageEntity]) -> String {
    let utf16: Vec<u16> = text.encode_utf16().collect();
    if utf16.is_empty() {
        return String::new();
    }

    // Cari batas akhir perintah `/setwelcome` (dalam UTF-16 code units)
    let mut start_idx = 0;

    // Skip kata pertama (/setwelcome)
    while start_idx < utf16.len() {
        let ch = String::from_utf16_lossy(&utf16[start_idx..start_idx + 1]);
        if ch.chars().next().map_or(false, |c| c.is_whitespace()) {
            break;
        }
        start_idx += 1;
    }

    // Skip spasi setelah perintah
    while start_idx < utf16.len() {
        let ch = String::from_utf16_lossy(&utf16[start_idx..start_idx + 1]);
        if !ch.chars().next().map_or(false, |c| c.is_whitespace()) {
            break;
        }
        start_idx += 1;
    }

    if start_idx >= utf16.len() {
        return String::new();
    }

    let content_utf16 = &utf16[start_idx..];

    // Filter dan sesuaikan offset entities setelah perintah dipotong
    let mut events: Vec<(usize, bool, String)> = Vec::new();

    for entity in entities {
        if matches!(entity.kind, MessageEntityKind::BotCommand) {
            continue;
        }

        if entity.offset >= start_idx {
            let rel_start = entity.offset - start_idx;
            let rel_end = rel_start + entity.length;

            if rel_end > content_utf16.len() {
                continue;
            }

            let (open_tag, close_tag) = match &entity.kind {
                MessageEntityKind::Bold => ("<b>".to_string(), "</b>".to_string()),
                MessageEntityKind::Italic => ("<i>".to_string(), "</i>".to_string()),
                MessageEntityKind::Underline => ("<u>".to_string(), "</u>".to_string()),
                MessageEntityKind::Strikethrough => ("<s>".to_string(), "</s>".to_string()),
                MessageEntityKind::Spoiler => {
                    ("<tg-spoiler>".to_string(), "</tg-spoiler>".to_string())
                }
                MessageEntityKind::Code => ("<code>".to_string(), "</code>".to_string()),
                MessageEntityKind::Pre { language } => {
                    if let Some(lang) = language {
                        (
                            format!("<pre><code class=\"language-{}\">", html_escape(lang)),
                            "</code></pre>".to_string(),
                        )
                    } else {
                        ("<pre>".to_string(), "</pre>".to_string())
                    }
                }
                MessageEntityKind::TextLink { url } => (
                    format!("<a href=\"{}\">", html_escape(&url.to_string())),
                    "</a>".to_string(),
                ),
                MessageEntityKind::TextMention { user } => (
                    format!("<a href=\"tg://user?id={}\">", user.id),
                    "</a>".to_string(),
                ),
                MessageEntityKind::Blockquote => {
                    ("<blockquote>".to_string(), "</blockquote>".to_string())
                }
                _ => continue,
            };

            events.push((rel_start, false, open_tag));
            events.push((rel_end, true, close_tag));
        }
    }

    // Urutkan event tag: posisi terkecil dulu, jika posisi sama maka tag penutup (close) didahulukan
    events.sort_by(|a, b| {
        if a.0 != b.0 {
            a.0.cmp(&b.0)
        } else {
            b.1.cmp(&a.1)
        }
    });

    // Reconstruct teks dengan tag HTML
    let mut result = String::new();
    let mut last_pos = 0;

    for (pos, _is_close, tag) in events {
        if pos > last_pos && pos <= content_utf16.len() {
            let slice = String::from_utf16_lossy(&content_utf16[last_pos..pos]);
            result.push_str(&html_escape(&slice));
            last_pos = pos;
        }
        result.push_str(&tag);
    }

    if last_pos < content_utf16.len() {
        let slice = String::from_utf16_lossy(&content_utf16[last_pos..]);
        result.push_str(&html_escape(&slice));
    }

    result
}
