// Standards ─────────────────────────────────────────────────────
use std::{
    sync::mpsc::Sender,
    borrow::Cow,
};

// Crates ───────────────────────────────────────────────────────
use chrono::Local;
use notify::{Event, RecommendedWatcher};
use serde::{Deserialize, Serialize};

// mods ─────────────────────────────────────────────────────────
use crate::{
    consts::{DAILY, REAL_TIME, WEEKLY},
    utils::status_emoji,
};

// Structs & Enums ──────────────────────────────────────────────

// Job
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Job {
    pub id: Option<u16>,
    pub source: String,
    pub target: String,
    pub frequency: String,
    pub hour: u8,
    pub day: Option<String>,
    pub mirror: u8,
    pub active: u8,
}

impl Job {
    pub fn new(frequency: &str) -> Self {
        Self {
            id: None,
            source: String::new(),
            target: String::new(),
            frequency: frequency.to_string(),
            hour: 0,
            day: None,
            mirror: 0,
            active: 0,
        }
    }

    pub fn get_fields_data(&self) -> Vec<Cow<'_, str>> {
        let formatted_hour = format!("{:02}", self.hour);

        match self.frequency.as_str() {
            REAL_TIME => vec![
                Cow::Owned(self.id.unwrap().to_string()),
                Cow::Borrowed(&self.source),
                Cow::Borrowed(&self.target),
                Cow::Borrowed(status_emoji(self.mirror)),
                Cow::Borrowed(status_emoji(self.active)),
            ],
            DAILY => vec![
                Cow::Owned(self.id.map(|id| id.to_string()).unwrap_or_default()),
                Cow::Borrowed(&self.source),
                Cow::Borrowed(&self.target),
                Cow::Owned(formatted_hour),
                Cow::Borrowed(status_emoji(self.mirror)),
                Cow::Borrowed(status_emoji(self.active)),
            ],
            WEEKLY => {
                let day = self.day.as_deref().unwrap_or_default();
                let formatted_day = format!("{:02}", day);

                vec![
                    Cow::Owned(self.id.map(|id| id.to_string()).unwrap_or_default()),
                    Cow::Borrowed(&self.source),
                    Cow::Borrowed(&self.target),
                    Cow::Owned(formatted_hour),
                    Cow::Owned(formatted_day),
                    Cow::Borrowed(status_emoji(self.mirror)),
                    Cow::Borrowed(status_emoji(self.active)),
                ]
            }
            _ => panic!(
                "❌ Failed to get fields data from job record because [{}] is not a valid frequency",
                self.frequency
            ),
        }
    }
}

// Stat
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Stat {
    pub name: String,
    pub count: u16,
    pub active_count: u16,
    pub inactive_count: u16,
}

impl Stat {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            count: 0,
            active_count: 0,
            inactive_count: 0,
        }
    }
}

// Log
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Log {
    pub id: Option<u16>,
    pub startstamp: String,
    pub endstamp: String,
    pub status: String,
    pub success_count: u16,
    pub failed_count: u16,
    pub log_results: Option<Vec<LogResult>>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            id: None,
            startstamp: Local::now().format("%d-%m-%Y %H:%M").to_string(),
            endstamp: String::new(),
            status: String::new(),
            success_count: 0,
            failed_count: 0,
            log_results: None,
        }
    }

    pub fn get_fields_data(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Owned(self.id.map(|id| id.to_string()).unwrap_or_default()),
            Cow::Borrowed(&self.startstamp),
            Cow::Borrowed(&self.endstamp),
            Cow::Borrowed(&self.status),
            Cow::Owned(self.success_count.to_string()),
            Cow::Owned(self.failed_count.to_string()),
        ]
    }
}

// LogResult
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogResult {
    pub log_id: Option<u16>,
    pub frequency: String,
    pub message: String,
    pub source: String,
    pub target: String,
}

impl LogResult {
    pub fn new(frequency: &str, message: &str, source: &str, target: &str) -> Self {
        Self {
            log_id: None,
            frequency: frequency.to_string(),
            message: message.to_string(),
            source: source.to_string(),
            target: target.to_string(),
        }
    }

    pub fn get_fields_data(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Borrowed(&self.frequency),
            Cow::Borrowed(&self.source),
            Cow::Borrowed(&self.target),
            Cow::Borrowed(&self.message),
        ]
    }
}

// WatchedJob
pub struct WatchedJob {
    pub job: Job,
    pub job_watcher: RecommendedWatcher,
    pub job_tx: Sender<Result<Event, notify::Error>>,
}
