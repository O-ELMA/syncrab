// Standards ─────────────────────────────────────────────────────
use std::collections::HashMap;

// Crates ────────────────────────────────────────────────────────
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Stylize},
    widgets::{Block, BorderType, Padding},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    consts::{
        DAILY, DAILY_BACKUPS, DAILY_COLS, JOURNAL, JOURNAL_COLS, LOG, LOG_COLS, MONTHLY,
        MONTHLY_BACKUPS, WEEKLY, WEEKLY_BACKUPS, WEEKLY_MONTHLY_COLS,
    },
    structs::{Job, Stat},
};

pub fn get_stats(jobs_by_freq: &HashMap<&'static str, Vec<Job>>) -> HashMap<&'static str, Stat> {
    let mut stats_by_freq: HashMap<&'static str, Stat> = HashMap::with_capacity(3);
    stats_by_freq.insert(DAILY, Stat::new(DAILY_BACKUPS));
    stats_by_freq.insert(WEEKLY, Stat::new(WEEKLY_BACKUPS));
    stats_by_freq.insert(MONTHLY, Stat::new(MONTHLY_BACKUPS));

    for (freq, jobs) in jobs_by_freq {
        let stat = stats_by_freq.get_mut(freq).unwrap();
        stat.count = jobs.len().try_into().unwrap();

        for job in jobs {
            if job.active == 1 {
                stat.active_count += 1
            } else {
                stat.inactive_count += 1
            };
        }
    }

    stats_by_freq
}

pub fn get_columns_info_by_key(
    key: &str,
) -> (
    &'static [&'static str],
    &'static [Constraint],
    &'static [Alignment],
) {
    match key {
        DAILY => (
            DAILY_COLS,
            &[
                Constraint::Length(3),
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                Constraint::Length(12),
                Constraint::Length(12),
            ],
            &[
                Alignment::Center,
                Alignment::Left,
                Alignment::Left,
                Alignment::Center,
                Alignment::Center,
            ],
        ),
        WEEKLY | MONTHLY => (
            WEEKLY_MONTHLY_COLS,
            &[
                Constraint::Length(3),
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
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
                Constraint::Length(12),
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

pub fn field(title: &str, title_style: Color, border_style: Color) -> Block {
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

