// Standards ─────────────────────────────────────────────────────
use std::{env, fs::OpenOptions, io::Write, path::PathBuf};

// Crates ────────────────────────────────────────────────────────
use rusqlite::Connection;
use chrono::Local;

// mods ──────────────────────────────────────────────────────────
use super::paths::home_dir;
use crate::{
    consts::{FAILED, LOG_PATH, PARTIAL, SUCCESS},
    db::db::{insert_log, insert_log_resuts},
    structs::{Job, Log, LogResult},
};

pub fn fallback_log(log: &Log, results: &[LogResult], error: &str) {
    let log_path = match home_dir() {
        Some(home) => LOG_PATH.replace("$HOME", &home.to_string_lossy()),
        None => env::temp_dir()
            .join("syncrab_log.log")
            .to_string_lossy()
            .into_owned(),
    };
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "=== Cron job failed at {} ===", log.endstamp);
        let _ = writeln!(file, "Status: {}", log.status);
        let _ = writeln!(file, "Success count: {}", log.success_count);
        let _ = writeln!(file, "Failed count: {}", log.failed_count);
        let _ = writeln!(file, "Error: {}", error);

        for res in results {
            let _ = writeln!(
                file,
                "[{}] {} | From: {} -> {}",
                res.frequency, res.message, res.source, res.target
            );
        }

        let _ = writeln!(file, "----------------------------");
    } else {
        // Well, there isn't much I can do more lol
        // Happy debugging!
        eprintln!("Critical: Failed to write fallback log");
    }
}

pub fn are_paths_valid(
    freq_str: &String,
    job: &Job,
    source: &PathBuf,
    target: &PathBuf,
    failed_directories: &mut Vec<LogResult>,
) -> bool {
    if source.exists() && !source.is_absolute() {
        failed_directories.push(LogResult::new(
            freq_str,
            "Source path must be absolute",
            &job.source,
            &job.target,
        ));
        return false;
    } else if !source.exists() {
        failed_directories.push(LogResult::new(
            freq_str,
            "Source path does not exist",
            &job.source,
            &job.target,
        ));
        return false;
    }

    if target.exists() {
        if !target.is_absolute() {
            failed_directories.push(LogResult::new(
                freq_str,
                "Target path must be absolute",
                &job.source,
                &job.target,
            ));
            return false;
        } else if target.is_file() {
            failed_directories.push(LogResult::new(
                freq_str,
                "Target path must be a directory",
                &job.source,
                &job.target,
            ));
            return false;
        }
    }

    true
}

pub fn log_results(
    conn: &mut Connection,
    mut log: Log,
    success_directories: Vec<LogResult>,
    failed_directories: Vec<LogResult>,
) {
    if success_directories.is_empty() && failed_directories.is_empty() {
        return;
    }

    // Set status
    let status = match (
        success_directories.is_empty(),
        failed_directories.is_empty(),
    ) {
        (false, true) => SUCCESS,
        (true, false) => FAILED,
        _ => PARTIAL,
    };
    log.status = status.into();

    // Set counts
    let success_count = success_directories.len();
    let failed_count = failed_directories.len();

    log.success_count = success_count as u16;
    log.failed_count = failed_count as u16;

    // Set log_results
    let mut all_results = Vec::with_capacity(success_count + failed_count);
    all_results.extend(success_directories);
    all_results.extend(failed_directories);

    log.endstamp = Local::now().format("%d-%m-%Y %H:%M").to_string();

    persist_log(conn, &log, &all_results);
}

pub fn persist_log(conn: &mut Connection, log: &Log, results: &[LogResult]) {
    match insert_log(conn, log.clone()) {
        Ok(id) => {
            if let Err(error) = insert_log_resuts(conn, id as u16, results) {
                fallback_log(log, results, &error);
            }
        }
        Err(error) => {
            fallback_log(log, results, &error);
        }
    }
}
