// Standards ─────────────────────────────────────────────────────
use std::collections::HashMap;

// Crates ────────────────────────────────────────────────────────
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Modifier, Stylize},
    text::Text,
    widgets::{Block, BorderType, Padding, Widget},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    consts::{
        ACTIVE, COL_BEIGE, COL_BORDER, COL_GRAY, COL_GREEN, COL_TITLE, DAILY, EMOJI_STATS,
        INACTIVE, REAL_TIME, WEEKLY,
    },
    structs::Stat,
};

// Header ────────────────────────────────────────────────────────
pub fn header(area: Rect, buf: &mut Buffer, stats: &HashMap<&'static str, Stat>) {
    let horizotal_layout = Layout::horizontal(vec![Constraint::Fill(1); 3]);
    let [real_time_area, daily_area, weekly_area] = horizotal_layout.areas(area);

    card(real_time_area, buf, stats.get(REAL_TIME).unwrap());
    card(daily_area, buf, stats.get(DAILY).unwrap());
    card(weekly_area, buf, stats.get(WEEKLY).unwrap());
}

fn card(area: Rect, buf: &mut Buffer, stat: &Stat) {
    let block = Block::bordered()
        .padding(Padding::new(1, 1, 0, 0))
        .border_style(COL_BORDER)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    let vertical_layout = Layout::vertical(vec![Constraint::Fill(1); 3]);
    let [top, middle, bottom] = vertical_layout.areas(block.inner(area));

    Text::from(format!("{} {}", EMOJI_STATS, stat.name))
        .add_modifier(Modifier::BOLD)
        .fg(COL_TITLE)
        .render(top, buf);

    Text::from(stat.count.to_string())
        .add_modifier(Modifier::BOLD)
        .fg(COL_BEIGE)
        .render(middle, buf);

    let horizontal_layout = Layout::horizontal(vec![Constraint::Fill(1); 2]);
    let [left, right] = horizontal_layout.areas(bottom);

    Text::from(format!("{} {}", stat.active_count, ACTIVE))
        .fg(COL_GREEN)
        .render(left, buf);

    Text::from(format!("{} {}", stat.inactive_count, INACTIVE))
        .fg(COL_GRAY)
        .render(right, buf);
}
