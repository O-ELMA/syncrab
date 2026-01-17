// Standards ─────────────────────────────────────────────────────
use std::{
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    fs::metadata,
    fs::{OpenOptions, copy, create_dir_all, read_dir, remove_dir_all, remove_file},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

// Crates ────────────────────────────────────────────────────────
use rusqlite::Connection;
use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Stylize},
    widgets::{Block, BorderType, Padding},
};

// mods ──────────────────────────────────────────────────────────
use crate::{
    app::structs::Filter,
    consts::{
        DAILY, DAILY_BACKUPS, DAILY_COLS, FAILED, JOURNAL, JOURNAL_COLS, LOG, LOG_COLS, LOG_PATH,
        PARTIAL, REAL_TIME, REAL_TIME_BACKUPS, REAL_TIME_COLS, SUCCESS, WEEKLY, WEEKLY_BACKUPS,
        WEEKLY_COLS, EMOJI_ACTIVE, EMOJI_INACTIVE
    },
    db::db::{insert_log, insert_log_resuts},
    structs::{Job, Log, LogResult, Stat},
};

pub fn get_stats(jobs_by_freq: &HashMap<&'static str, Vec<Job>>) -> HashMap<&'static str, Stat> {
    let mut stats_by_freq: HashMap<&'static str, Stat> = HashMap::with_capacity(3);
    stats_by_freq.insert(DAILY, Stat::new(DAILY_BACKUPS));
    stats_by_freq.insert(WEEKLY, Stat::new(WEEKLY_BACKUPS));
    stats_by_freq.insert(REAL_TIME, Stat::new(REAL_TIME_BACKUPS));

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

pub fn field(title: &str, title_style: Color, border_style: Color) -> Block<'_> {
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

pub fn normalise_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = env::var_os("HOME") {
            return Path::new(&home).join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

pub fn copy_dir(
    source: &PathBuf,
    target: &PathBuf,
    mirror: u8,
    total: usize,
    count: &mut usize,
) -> Result<(), String> {
    if source.is_file() {
        // Create parent dir if it doesn't exist
        if let Some(parent) = target.parent() {
            create_dir_all(parent).map_err(|e| {
                format!(
                    "Could not create parent directory [{}] because {}",
                    parent.display(),
                    e
                )
            })?;
        }

        if should_copy(source, target)? {
            copy(source, &target).map_err(|e| {
                format!(
                    "Failed to copy file [{}] to [{}] because [{}]",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        }

        *count += 1;
        draw_progress_bar(&count, &total);
    } else if source.is_dir() {
        create_dir_all(target).map_err(|e| {
            format!(
                "Could not create destination directory [{}] because {}",
                target.display(),
                e
            )
        })?;

        let source_entries: Vec<_> = read_dir(source)
            .map_err(|e| {
                format!(
                    "Could not read directory [{}] because {}",
                    source.display(),
                    e
                )
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                format!(
                    "Failed to read source entry in [{}] because [{}]",
                    source.display(),
                    e
                )
            })?;

        if mirror == 1 {
            let source_filenames: HashSet<OsString> = source_entries
                .iter()
                .map(|entry| entry.file_name())
                .collect();

            if let Ok(target_itr) = read_dir(target) {
                for entry in target_itr {
                    if let Ok(entry) = entry {
                        // If the target file/dir does NOT exist in source, delete it
                        if !source_filenames.contains(&entry.file_name()) {
                            let path = entry.path();
                            let result = if path.is_dir() {
                                remove_dir_all(&path)
                            } else {
                                remove_file(&path)
                            };

                            if let Err(e) = result {
                                return Err(format!(
                                    "Failed to delete orphan file [{}] because {}",
                                    path.display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }

        for entry in source_entries {
            let path = entry.path();
            let new_target = target.join(entry.file_name());
            copy_dir(&path, &new_target, mirror, total, count)?;
        }
    } else {
        return Err(format!(
            "The source [{}] cannot be copied because it is neither a file nor a dir (probably a symlink)",
            source.display()
        ));
    }

    Ok(())
}

fn should_copy(source: &PathBuf, target: &PathBuf) -> Result<bool, String> {
    match metadata(target) {
        Ok(dest_metadata) => {
            let source_metadata = metadata(source).map_err(|e| {
                format!(
                    "Could not get metadata of the source [{}] because [{}]",
                    source.display(),
                    e
                )
            })?;

            let source_modif_time = source_metadata.modified().map_err(|e| {
                format!(
                    "Could not get modified time for source [{}] because [{}]",
                    source.display(),
                    e
                )
            })?;

            let dest_modif_time = dest_metadata.modified().map_err(|e| {
                format!(
                    "Could not get modified time for destination [{}] because [{}]",
                    target.display(),
                    e
                )
            })?;

            Ok(source_modif_time > dest_modif_time || source_metadata.len() != dest_metadata.len())
        }
        Err(_) => Ok(true), // If destination doesn't exist, copy
    }
}

pub fn count_children(path: &PathBuf) -> usize {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("find {} | wc -l", path.display()))
        .output()
        .expect("Failed to execute command");

    let count_str = String::from_utf8_lossy(&output.stdout);
    count_str.trim().parse::<usize>().unwrap_or(0)
}

fn draw_progress_bar(current: &usize, total: &usize) {
    // Don't draw the bar if the total is one item or no item
    if *total == 0 || *total == 1 {
        return;
    }

    let progress = (*current as f32) / (*total as f32);
    let bar_width = 50;
    let progress_length = (progress * bar_width as f32).round() as usize;
    let progress_length = progress_length.min(bar_width);
    let bar = "█".repeat(progress_length) + &"-".repeat(bar_width - progress_length);

    // Use print! instead of println! to avoid a newline after each update
    print!(
        "\r[{} / {}] |{}| {:.2}%",
        current,
        total,
        bar,
        progress * 100.0
    );

    std::io::stdout().flush().unwrap();
}
pub fn fallback_log(log: &Log, results: &Vec<LogResult>, error: &str) {
    let log_path = if let Ok(home) = env::var("HOME") {
        LOG_PATH.replace("$HOME", &home)
    } else {
        "/tmp/syncrab_log.log".to_string()
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
            freq_str.into(),
            "Source path must be absolute".into(),
            job.source.clone(),
            job.target.clone(),
        ));
        return false;
    } else if !source.exists() {
        failed_directories.push(LogResult::new(
            freq_str.into(),
            "Source path does not exist".into(),
            job.source.clone(),
            job.target.clone(),
        ));
        return false;
    }

    if target.exists() {
        if !target.is_absolute() {
            failed_directories.push(LogResult::new(
                freq_str.into(),
                "Target path must be absolute".into(),
                job.source.clone(),
                job.target.clone(),
            ));
            return false;
        } else if target.is_file() {
            failed_directories.push(LogResult::new(
                freq_str.into(),
                "Target path must be a directory".into(),
                job.source.clone(),
                job.target.clone(),
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
        (true, true) => SUCCESS,
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

    match insert_log(conn, log.clone()) {
        Ok(id) => {
            if let Err(error) = insert_log_resuts(conn, id as u16, all_results.clone()) {
                fallback_log(&log, &all_results, &error);
            }
        }
        Err(error) => {
            fallback_log(&log, &all_results, &error);
        }
    }
}

// Dissect content into lines of 5 words max or 32 characters max
pub fn into_lines(content: String) -> (u16, String) {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() == 1 {
        return (1, content);
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut word_count = 0;

    for word in words {
        let proposed = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if word_count == 5 || proposed.len() > 32 {
            // Push the current line and start a new one
            lines.push(current_line);
            current_line = word.to_string();
            word_count = 1;
        } else {
            current_line = proposed;
            word_count += 1;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    (lines.len() as u16, lines.join("\n"))
}

pub fn get_active_jobs<'a>(search_term: &str, filter: &Filter, jobs: &'a [Job]) -> Vec<&'a Job> {
    let jobs = if search_term.is_empty() {
        jobs.iter()
            .filter(|job| match filter {
                Filter::All => true,
                Filter::Active => job.active == 1,
                Filter::Inactive => job.active == 0,
            })
            .collect()
    } else {
        jobs.iter()
            .filter(|job| {
                job.source.to_lowercase().contains(search_term)
                    || job.target.to_lowercase().contains(search_term)
                    || job.id.unwrap().to_string().contains(search_term)
            })
            .filter(|job| match filter {
                Filter::All => true,
                Filter::Active => job.active == 1,
                Filter::Inactive => job.active == 0,
            })
            .collect()
    };

    jobs
}

pub fn get_active_logs<'a>(search_term: &str, logs: &'a [Log]) -> Vec<&'a Log> {
    let logs = if search_term.is_empty() {
        logs.iter().collect()
    } else {
        logs.into_iter()
            .filter(|log| {
                log.startstamp.to_lowercase().contains(&search_term)
                    || log.endstamp.to_lowercase().contains(&search_term)
                    || log.status.to_lowercase().contains(&search_term)
                    || log.success_count.to_string().contains(&search_term)
                    || log.failed_count.to_string().contains(&search_term)
                    || log.id.unwrap().to_string().contains(&search_term)
            })
            .collect()
    };

    logs
}

pub fn status_emoji(value: u8) -> String {
    match value {
        1 => EMOJI_ACTIVE.to_string(),
        _ => EMOJI_INACTIVE.to_string(),
    }
}