// Standards ─────────────────────────────────────────────────────
use std::collections::HashMap;

// Crates ────────────────────────────────────────────────────────
use ratatui::{layout::{Alignment, Constraint}, style::{Color, Modifier, Stylize}, widgets::{Block, BorderType, Padding}};

// mods ──────────────────────────────────────────────────────────
use crate::structs::{Job, Stat};

pub fn get_stats(jobs_by_freq: &HashMap<&'static str, Vec<Job>>) -> HashMap<&'static str, Stat> {
    let mut stats_by_freq: HashMap<&'static str, Stat> = HashMap::with_capacity(3);
    stats_by_freq.insert("daily", Stat::new("Daily Backups"));
    stats_by_freq.insert("weekly", Stat::new("Weekly Backups"));
    stats_by_freq.insert("monthly", Stat::new("Monthly Backups"));

    for (freq, jobs) in jobs_by_freq {
        let stat = stats_by_freq.get_mut(freq).unwrap();
        stat.count = jobs.len().try_into().unwrap();

        for job in jobs {
            if job.active == 1 { stat.active_count += 1 } else { stat.inactive_count += 1 };
        }
    }

    stats_by_freq
}

pub fn get_columns_info_by_key(key: &str) -> (&'static [&'static str], &'static [Constraint], &'static [Alignment]) {
    match key {
        "daily" => (
            &["Id", "Source", "Target", "Hour", "Active"],
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
        "weekly" | "monthly" => (
            &["Id", "Source", "Target", "Hour", "Day", "Active"],
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
        "journal" => (
            &["Id", "Started at", "Ended at", "Status", "Jobs Succeeded", "Jobs Failed"],
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
        "log" => (
            &["Type", "Source", "Target", "Message"],
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