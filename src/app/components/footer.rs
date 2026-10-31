// Crates ────────────────────────────────────────────────────────
use ratatui::{
    layout::Alignment,
    prelude::{Buffer, Rect},
    widgets::{Paragraph, Widget},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    app::{
        app::App,
        structs::{Filter, Modal},
    },
    consts::{
        ACTION_ACTIVE, ACTION_BACKUP, ACTION_CLONE, ACTION_CLOSE, ACTION_DELETE, ACTION_DISABLE,
        ACTION_EDIT, ACTION_ENABLE, ACTION_ERASE, ACTION_LOGS, ACTION_MIRROR, ACTION_MOVE,
        ACTION_NEW, ACTION_QUIT, ACTION_UPDATE, ACTION_VIEW, SEPARATOR,
    },
};

// Footer ───────────────────────────────────────────────────────
pub fn footer(area: Rect, buf: &mut Buffer, app: &App) {
    let mut shortcuts = Vec::with_capacity(12); // Pre-allocate space for expected max number of shortcuts

    if app.active_modal == Some(Modal::Job) {
        shortcuts.push(ACTION_CLOSE);
        shortcuts.push(ACTION_MOVE);
        shortcuts.push(ACTION_UPDATE);
    } else if let Some(comp) = &app.active_component {
        if comp.is_field() {
            shortcuts.push(ACTION_ERASE);
            shortcuts.push(ACTION_CLOSE);
        } else if app.show_journal {
            if app.selected_log.is_some() {
                shortcuts.push(ACTION_BACKUP);
                shortcuts.push(ACTION_CLOSE);
                shortcuts.push(ACTION_QUIT);
            } else {
                shortcuts.push(ACTION_MOVE);
                shortcuts.push(ACTION_BACKUP);
                shortcuts.push(ACTION_VIEW);
                shortcuts.push(ACTION_QUIT);
            }
        } else {
            let stat = app.stats.get(comp.to_str()).unwrap();
            let count: u16 = match app.filter {
                Filter::All => stat.count,
                Filter::Active => stat.active_count,
                Filter::Inactive => stat.inactive_count,
            };

            // Base shortcuts for non-field components
            shortcuts.push(ACTION_NEW);
            shortcuts.push(ACTION_LOGS);
            shortcuts.push(ACTION_QUIT);

            if count > 0 {
                shortcuts.push(ACTION_MOVE);
                shortcuts.push(ACTION_DELETE);
                shortcuts.push(ACTION_EDIT);
                shortcuts.push(ACTION_MIRROR);
                shortcuts.push(ACTION_ACTIVE);
                shortcuts.push(ACTION_CLONE);

                if count > 1 {
                    shortcuts.push(ACTION_ENABLE);
                    shortcuts.push(ACTION_DISABLE);
                }
            }
            shortcuts.push(ACTION_CLOSE);
        }
    } else {
        shortcuts.push(ACTION_LOGS);
        shortcuts.push(ACTION_QUIT);
    }

    // Join shortcuts into a string and render them
    Paragraph::new(
        shortcuts
            .chunks(5)
            .map(|chunk| chunk.join(SEPARATOR))
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .alignment(Alignment::Center)
    .render(area, buf);
}
