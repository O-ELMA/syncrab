// Crates ────────────────────────────────────────────────────────
use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    style::Modifier,
    text::{Line, Text},
    widgets::Widget,
};

// mods ──────────────────────────────────────────────────────────
use crate::consts::{APP_SUBTITLE, APP_TITLE, COL_LBROWN, COL_ORANGE};

// Title ─────────────────────────────────────────────────────────
pub fn title(area: Rect, buf: &mut Buffer) {
    Text::from(vec![
        Line::styled(APP_TITLE, (COL_ORANGE, Modifier::BOLD)),
        Line::styled(APP_SUBTITLE, (COL_LBROWN, Modifier::ITALIC)),
    ])
    .alignment(Alignment::Center)
    .render(area, buf);
}
