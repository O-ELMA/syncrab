// Crates ────────────────────────────────────────────────────────
use ratatui::{
    layout::{Alignment, Constraint},
};

// mods ──────────────────────────────────────────────────────────
use crate::consts::{
    DAILY, DAILY_COLS, JOURNAL, JOURNAL_COLS, LOG, LOG_COLS, REAL_TIME, REAL_TIME_COLS, WEEKLY,
    WEEKLY_COLS,
};

pub fn get_columns_info_by_key(
    key: &str,
) -> (
    &'static [&'static str],
    &'static [Constraint],
    &'static [Alignment],
) {
    match key {
        REAL_TIME => (
            REAL_TIME_COLS,
            &[
                Constraint::Length(3),
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                Constraint::Length(12),
            ],
            &[
                Alignment::Center,
                Alignment::Left,
                Alignment::Left,
                Alignment::Center,
            ],
        ),
        DAILY => (
            DAILY_COLS,
            &[
                Constraint::Length(3),
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
            ],
            &[
                Alignment::Center,
                Alignment::Left,
                Alignment::Left,
                Alignment::Center,
                Alignment::Center,
                Alignment::Center,
            ],
        ),
        WEEKLY => (
            WEEKLY_COLS,
            &[
                Constraint::Length(3),
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(8),
            ],
            &[
                Alignment::Center,
                Alignment::Left,
                Alignment::Left,
                Alignment::Center,
                Alignment::Center,
                Alignment::Center,
                Alignment::Center,
            ],
        ),
        JOURNAL => (
            JOURNAL_COLS,
            &[
                Constraint::Length(8),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
            &[
                Alignment::Center,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Center,
                Alignment::Center,
            ],
        ),
        LOG => (
            LOG_COLS,
            &[
                Constraint::Length(8),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
            &[
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
            ],
        ),
        _ => panic!("❌ Invalid key: {}", key),
    }
}
