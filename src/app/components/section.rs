// Crates ────────────────────────────────────────────────────────
use crossterm::event::MouseEvent;
use ratatui::{
    layout::{Constraint, Layout, Position},
    prelude::{Buffer, Rect},
    style::{Modifier, Stylize},
    text::Text,
    widgets::{Block, BorderType, Padding, Widget},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    app::{
        app::App,
        structs::{Component, Modal},
    },
    consts::{
        COL_BEIGE, COL_BORDER, COL_TITLE, DAILY, EMOJI_SECTION, JOURNAL, REAL_TIME, SHORTCUT_DAILY,
        SHORTCUT_REAL_TIME, SHORTCUT_WEEKLY, WEEKLY,
    },
};

use super::table::table;

// Section ───────────────────────────────────────────────────────
pub fn section(area: Rect, buf: &mut Buffer, app: &mut App) {
    if app.show_journal {
        table(area, buf, JOURNAL, app);
    } else {
        let vertical_layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
        let [real_time_area, daily_area, weekly_area] = vertical_layout.areas(area);

        block(real_time_area, buf, REAL_TIME, SHORTCUT_REAL_TIME, app);
        block(daily_area, buf, DAILY, SHORTCUT_DAILY, app);
        block(weekly_area, buf, WEEKLY, SHORTCUT_WEEKLY, app);
    }
}

fn block(area: Rect, buf: &mut Buffer, freq: &str, shortcut: char, app: &mut App) {
    let name = &app.stats.get(freq).unwrap().name;

    let mut block_style = COL_BORDER;

    if app.active_modal.is_none() {
        if let Some(MouseEvent { column, row, .. }) = app.event {
            if app.active_modal != Some(Modal::Job) {
                let pos = Position::new(column, row);

                // If the block is clicked, set it as active
                if area.contains(pos) {
                    app.active_component = Some(Component::from_str(freq));
                    block_style = COL_BEIGE;
                }
                // If clicked outside, disable the active component if it was previously active
                else if app
                    .active_component
                    .as_ref()
                    .map_or(false, |c| c == &Component::from_str(freq))
                {
                    app.active_component = None;
                }
            }
        }
        // Check if the component is already active from keymaps
        if let Some(comp) = app.active_component.as_ref() {
            if comp == &Component::from_str(freq) {
                block_style = COL_BEIGE;
            }
        }
    }

    let block = Block::bordered()
        .padding(Padding::new(1, 1, 0, 0))
        .border_style(block_style)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    let vertical_layout = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]);
    let [top, bottom] = vertical_layout.areas(block.inner(area));

    Text::from(format!("{} [{}] {}", EMOJI_SECTION, shortcut, name))
        .add_modifier(Modifier::BOLD)
        .fg(COL_TITLE)
        .render(top, buf);

    table(bottom, buf, freq, app);
}
