// Crates ────────────────────────────────────────────────────────
use crossterm::event::MouseEvent;
use ratatui::{
    layout::{Constraint, Layout, Position},
    prelude::{Buffer, Rect},
    style::Color,
    widgets::{Paragraph, Widget},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    app::{app::App, structs::Component},
    consts::{
        COL_BEIGE, COL_BORDER, COL_TITLE, EMOJI_FILTER, EMOJI_SEARCH, FILTER, SEARCH,
        SHORTCUT_FILTER, SHORTCUT_SEARCH,
    },
    utils::{capitalise, field},
};

// Search ────────────────────────────────────────────────────────
pub fn search(area: Rect, buf: &mut Buffer, app: &mut App) {
    let horizontal_layout =
        Layout::horizontal(vec![Constraint::Ratio(1, 1), Constraint::Ratio(1, 3)]);
    let [left, right] = horizontal_layout.areas(area);

    let mut search_style = (COL_TITLE, COL_BORDER);

    if app.active_modal.is_none() {
        if let Some(MouseEvent { column, row, .. }) = app.event {
            let pos = Position::new(column, row);

            if left.contains(pos) {
                search_style = enable_search(app, left, buf);
            } else if right.contains(pos) && !app.filter_clicked {
                app.filter_clicked = true;
                app.filter = app.filter.next();
            }
        } else if app.active_component == Some(Component::Search) {
            search_style = enable_search(app, left, buf);
        }
    }

    Paragraph::new(app.search.value.as_str())
        .block(field(
            &format!(
                "{} [{}] {}",
                EMOJI_SEARCH,
                SHORTCUT_SEARCH,
                capitalise(SEARCH)
            ),
            search_style.0,
            search_style.1,
        ))
        .render(left, buf);

    Paragraph::new(capitalise(&app.filter.to_string()))
        .block(field(
            &format!(
                "{} [{}] {}",
                EMOJI_FILTER,
                SHORTCUT_FILTER,
                capitalise(FILTER)
            ),
            COL_TITLE,
            COL_BORDER,
        ))
        .render(right, buf);
}

fn enable_search(app: &mut App, left: Rect, buf: &mut Buffer) -> (Color, Color) {
    app.active_component = Some(Component::Search);
    app.search.set_cursor_position(left, buf);
    (COL_TITLE, COL_BEIGE)
}
