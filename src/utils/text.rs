// Standardss ────────────────────────────────────────────────────
use ratatui::{
    style::{Color, Modifier, Stylize},
    widgets::{Block, BorderType, Padding},
};

// mods ──────────────────────────────────────────────────────────
use crate::consts::{EMOJI_ACTIVE, EMOJI_INACTIVE};

pub fn field(title: &str, title_style: Color, border_style: Color) -> Block<'_> {
    Block::bordered()
        .title(title)
        .padding(Padding::new(1, 1, 0, 0))
        .title_style(title_style)
        .add_modifier(Modifier::BOLD)
        .border_style(border_style)
        .border_type(BorderType::Rounded)
}

pub fn capitalise(s: &str) -> String {
    s.get(0..1).unwrap().to_uppercase() + &s[1..].to_lowercase()
}

// Dissect content into lines of 5 words max or 32 characters max
pub fn into_lines(content: &str) -> (u16, String) {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() == 1 {
        return (1, content.to_string());
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut word_count = 0;

    for word in words {
        let proposed = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if word_count == 5 || proposed.len() > 32 {
            // Push the current line and start a new one
            lines.push(current_line);
            current_line = word.to_string();
            word_count = 1;
        } else {
            current_line = proposed;
            word_count += 1;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    (lines.len() as u16, lines.join("\n"))
}

pub fn status_emoji(value: u8) -> &'static str {
    match value {
        1 => EMOJI_ACTIVE,
        _ => EMOJI_INACTIVE,
    }
}
