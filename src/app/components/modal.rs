// Crates ────────────────────────────────────────────────────────
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout},
    prelude::{Buffer, Rect},
    style::Stylize,
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Widget},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    app::{
        app::App,
        structs::Modal,
    },
    consts::{
        COL_BORDER, COL_RED, EMOJI_STATUS_FAILED, EMOJI_STATUS_OTHER, EMOJI_STATUS_PARTIAL,
        EMOJI_STATUS_SUCCESS, FAILED, LOG, PARTIAL, REAL_TIME, SUCCESS,
    },
};

use super::{form::form, table::table};

// Modal
pub fn modal(area: Rect, buf: &mut Buffer, app: &mut App) {
    if app.active_modal.is_none() {
        return;
    }

    let vertical_const: Constraint = match app.active_modal.as_ref().unwrap() {
        Modal::Log => Constraint::Percentage(80),
        Modal::Replace => Constraint::Length(6),
        Modal::Job => match app.selected_job.as_ref().unwrap().frequency.as_str() {
            REAL_TIME => Constraint::Length(6),
            _ => Constraint::Length(9),
        },
    };

    let vertical = Layout::vertical([vertical_const]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(80)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    Clear.render(area, buf);
    let block = Block::bordered()
        .padding(Padding::new(1, 1, 1, 1))
        .border_style(COL_BORDER)
        .border_type(BorderType::Rounded);
    block.clone().render(area, buf);

    // Render Log & LogResult table
    if app.selected_log.is_some() {
        log_modal(area, buf, &block, app);
    }
    // Render Job form | Replace string form
    else if matches!(app.active_modal, Some(Modal::Job) | Some(Modal::Replace)) {
        form(area, buf, app);
    }
}

fn log_modal(area: Rect, buf: &mut Buffer, block: &Block<'_>, app: &mut App) {
    let Some(log) = &app.selected_log else {
        return;
    };

    let status_emoji = match log.status.as_str() {
        SUCCESS => EMOJI_STATUS_SUCCESS,
        FAILED => EMOJI_STATUS_FAILED,
        PARTIAL => EMOJI_STATUS_PARTIAL,
        _ => EMOJI_STATUS_OTHER,
    };

    let inner_vertical = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]);
    let [first, second] = inner_vertical.areas(block.inner(area));

    Paragraph::new(format!(
        "⏱️ {}  |  ⏹️ {}  |  {} {}",
        log.startstamp, log.endstamp, status_emoji, log.status
    ))
    .alignment(Alignment::Center)
    .render(first, buf);

    table(second, buf, LOG, app);
}

pub fn delete_confirmation(area: Rect, buf: &mut Buffer, app: &App) {
    if app.pending_delete.is_none() {
        return;
    }
    let vertical = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(30)]).flex(Flex::Center);
    let [popup_area] = vertical.areas(area);
    let [popup_area] = horizontal.areas(popup_area);

    Clear.render(popup_area, buf);

    Block::bordered()
        .border_style(COL_RED)
        .border_type(BorderType::Rounded)
        .render(popup_area, buf);

    Paragraph::new("Press DEL again to confirm")
        .fg(COL_RED)
        .alignment(Alignment::Center)
        .render(
            Rect::new(popup_area.x + 1, popup_area.y + 1, popup_area.width - 2, 1),
            buf,
        );
}
