use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, User};

/// Mengubah teks mentah admin menjadi pesan akhir + Inline Keyboard
pub fn parse_welcome_message(
    raw_text: &str,
    user: &User,
    chat_title: &str,
) -> (String, InlineKeyboardMarkup) {
    // --- 1. SPREAD VARIABLE REPLACEMENT ---
    let first = user.first_name.clone();
    let last = user.last_name.clone().unwrap_or_default();
    let fullname = if last.is_empty() {
        first.clone()
    } else {
        format!("{} {}", first, last)
    };

    let username = match &user.username {
        Some(uname) => format!("@{}", uname),
        None => format!("<a href=\"tg://user?id={}\">{}</a>", user.id, first),
    };

    let mention = format!("<a href=\"tg://user?id={}\">{}</a>", user.id, first);
    let user_id = user.id.to_string();

    // Olah teks mentah: ganti semua variabel placeholder
    let processed_text = raw_text
        .replace("{first}", &first)
        .replace("{last}", &last)
        .replace("{fullname}", &fullname)
        .replace("{username}", &username)
        .replace("{mention}", &mention)
        .replace("{id}", &user_id)
        .replace("{chatname}", chat_title);

    // --- 2. PARSE BUTTON SYNTAX ---
    let mut clean_text_lines = Vec::new();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();

    for line in processed_text.lines() {
        let trimmed_line = line.trim();

        // Cek apakah baris ini mengandung sintaks tombol Rose
        if trimmed_line.contains('[') && trimmed_line.contains("](buttonurl://") {
            let mut remaining = trimmed_line;
            let mut current_row = Vec::new();

            while let Some(start_bracket) = remaining.find('[') {
                if let Some(end_bracket) = remaining[start_bracket..].find("](buttonurl://") {
                    let actual_end_bracket = start_bracket + end_bracket;
                    let btn_name = &remaining[start_bracket + 1..actual_end_bracket];

                    let url_start = actual_end_bracket + "](buttonurl://".len();
                    if let Some(end_parenthesis) = remaining[url_start..].find(')') {
                        let actual_end_parenthesis = url_start + end_parenthesis;
                        let raw_url = &remaining[url_start..actual_end_parenthesis];

                        // Cek apakah ada flag :same untuk tombol sejajar
                        let (clean_url, is_same) = if raw_url.ends_with(":same") {
                            (&raw_url[..raw_url.len() - 5], true)
                        } else {
                            (raw_url, false)
                        };

                        // Pastikan URL memiliki awalan http:// atau https://
                        let full_url = if !clean_url.starts_with("http://")
                            && !clean_url.starts_with("https://")
                        {
                            format!("https://{}", clean_url)
                        } else {
                            clean_url.to_string()
                        };

                        if let Ok(parsed_url) = reqwest::Url::parse(&full_url) {
                            let button = InlineKeyboardButton::url(btn_name, parsed_url);

                            if is_same {
                                if !current_row.is_empty() {
                                    current_row.push(button);
                                } else if let Some(last_row) = keyboard.last_mut() {
                                    last_row.push(button);
                                } else {
                                    current_row.push(button);
                                }
                            } else {
                                current_row.push(button);
                            }
                        }

                        remaining = &remaining[actual_end_parenthesis + 1..];
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if !current_row.is_empty() {
                keyboard.push(current_row);
            }
        } else {
            // Baris teks biasa (bukan tombol)
            clean_text_lines.push(line);
        }
    }

    let final_clean_text = clean_text_lines.join("\n");
    let markup = InlineKeyboardMarkup::new(keyboard);

    (final_clean_text, markup)
}
