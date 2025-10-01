// Crates ───────────────────────────────────────────────────────
use serde::{Deserialize, Serialize};

// mods ─────────────────────────────────────────────────────────
use crate::consts::{
    DAILY,
    WEEKLY,
    MONTHLY,
    EMOJI_ACTIVE,
    EMOJI_INACTIVE
};

// Structs & Enums ──────────────────────────────────────────────

// Job
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Job {
    pub id: Option<u8>,
    pub source: String,
    pub target: String,
    pub frequency: String,
    pub hour: u8,
    pub day: Option<String>,
    pub active: u8,
}

impl Job {
    pub fn new(freq: String) -> Self {
        Self {
            id: None,
            frequency: freq,
            day: Default::default(),
            hour: Default::default(),
            source: Default::default(),
            target: Default::default(),
            active: 0
        }
    }

    pub fn get_fields_data(&self) -> Vec<String> {
        let hour = self.hour.clone().to_string();
        let formatted_hour = if hour.len() == 1 { format!("0{}", hour) } else { hour };

        match self.frequency.as_str() {
            DAILY => vec![
                self.id.unwrap().to_string(),
                self.source.clone(),
                self.target.clone(),
                formatted_hour,
                if self.active == 1 { EMOJI_ACTIVE.to_string() } else { EMOJI_INACTIVE.to_string() },
            ],
            WEEKLY | MONTHLY => {
                let day = self.day.clone().unwrap_or_default();
                let formatted_day = if day.len() == 1 { format!("0{}", day) } else { day };

                vec![
                    self.id.unwrap().to_string(),
                    self.source.clone(),
                    self.target.clone(),
                    formatted_hour,
                    formatted_day,
                    if self.active == 1 { EMOJI_ACTIVE.to_string() } else { EMOJI_INACTIVE.to_string() },
                ]
            },
            _ => panic!("❌ Failed to get fields data from job record because [{}] is not a valid frequency", self.frequency),
        }
    }
}

// Stat
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Stat {
    pub name: String,
    pub count: u8,
    pub active_count: u8,
    pub inactive_count: u8,
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
    pub id: Option<u8>,
    pub startstamp: String,
    pub endstamp: String,
    pub status: String,
    pub success_count: u8,
    pub failed_count: u8,
    pub log_results: Option<Vec<LogResult>>,
}

impl Log {
    pub fn new(startstamp: String) -> Self {
        Self {
            id: None,
            startstamp,
            endstamp: String::new(),
            status: String::new(),
            success_count: 0,
            failed_count: 0,
            log_results: None
        }
    }

    pub fn get_fields_data(&self) -> Vec<String> {
        vec![
            self.id.unwrap().to_string(),
            self.startstamp.to_string(),
            self.endstamp.to_string(),
            self.status.clone(),
            self.success_count.to_string(),
            self.failed_count.to_string(),
        ]
    }
}

// LogResult
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogResult {
    pub log_id: Option<u8>,
    pub frequency: String,
    pub message: String,
    pub source: String,
    pub target: String,
}

impl LogResult {
    pub fn new(frequency: String, message: String, source: String, target: String) -> Self {
        Self {
            log_id: None,
            frequency,
            message,
            source,
            target,
        }
    }

    pub fn get_fields_data(&self) -> Vec<String> {
        vec![
            self.frequency.clone(),
            self.source.clone(),
            self.target.clone(),
            self.message.clone(),
        ]
    }
}