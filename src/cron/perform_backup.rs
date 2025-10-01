// Standards ─────────────────────────────────────────────────────
use std::{
    collections::HashMap,
    env,
    fs::{OpenOptions, copy, create_dir_all, read_dir},
    io::Write,
    path::{Path, PathBuf},
    process,
};

// Crates ───────────────────────────────────────────────────────
use chrono::{DateTime, Datelike, Local, Timelike};

// mods ──────────────────────────────────────────────────────────
use syncrab::{
    consts::{FAILED, LOG_PATH, PARTIAL, SUCCESS, VALID_OPTS},
    db::db::{get_jobs_to_run, init_db, insert_log, insert_log_resuts},
    structs::{Job, Log, LogResult},
};

// Const ─────────────────────────────────────────────────────────

// Init ──────────────────────────────────────────────────────────
fn main() {
    let now: DateTime<Local> = Local::now();
    let mut log = Log::new(now.format("%d-%m-%Y %H:%M").to_string());

    let arg = prompt_user();

    let conn = init_db();
    let jobs: HashMap<&'static str, Vec<Job>> = get_jobs_to_run(
        conn,
        arg.as_deref(),
        now.weekday().to_string(),
        format!("{:02}", now.day()),
        now.hour() as u8,
    );

    let mut success_directories: Vec<LogResult> = Vec::new();
    let mut failed_directories: Vec<LogResult> = Vec::new();

    for (freq, jobs) in jobs.iter() {
        if let Some(ref a) = arg {
            if matches!(a.as_str(), "daily" | "weekly" | "monthly") && a != freq {
                continue;
            }
        }

        if jobs.is_empty() {
            failed_directories.push(LogResult::new(
                freq.to_string(),
                "No jobs found".into(),
                String::new(),
                String::new(),
            ));
            continue;
        }

        for job in jobs {
            let freq_str = freq.to_string();
            let source = normalise_path(&job.source);
            let target = normalise_path(&job.target);

            if source.exists() && !source.is_absolute() {
                failed_directories.push(LogResult::new(
                    freq_str,
                    "Source path must be absolute".into(),
                    job.source.clone(),
                    job.target.clone(),
                ));
                continue;
            } else if !source.exists() {
                failed_directories.push(LogResult::new(
                    freq_str,
                    "Source path does not exist".into(),
                    job.source.clone(),
                    job.target.clone(),
                ));
                continue;
            }

            if target.exists() {
                if !target.is_absolute() {
                    failed_directories.push(LogResult::new(
                        freq_str,
                        "Target path must be absolute".into(),
                        job.source.clone(),
                        job.target.clone(),
                    ));
                    continue;
                } else if target.is_file() {
                    failed_directories.push(LogResult::new(
                        freq_str,
                        "Target path must be a directory".into(),
                        job.source.clone(),
                        job.target.clone(),
                    ));
                    continue;
                }
            }

            match copy_dir(&source, &target) {
                Ok(_) => success_directories.push(LogResult::new(
                    freq_str,
                    "OK".into(),
                    job.source.clone(),
                    job.target.clone(),
                )),
                Err(error) => failed_directories.push(LogResult::new(
                    freq_str,
                    error,
                    job.source.clone(),
                    job.target.clone(),
                )),
            };
        }
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

    log.success_count = success_count as u8;
    log.failed_count = failed_count as u8;

    // Set log_results
    let mut all_results = Vec::with_capacity(success_count + failed_count);
    all_results.extend(success_directories);
    all_results.extend(failed_directories);

    log.endstamp = Local::now().format("%d-%m-%Y %H:%M").to_string();

    match insert_log(log.clone()) {
        Ok(id) => {
            if let Err(error) = insert_log_resuts(id as u8, all_results.clone()) {
                fallback_log(&log, &all_results, &error);
            }
        }
        Err(error) => {
            fallback_log(&log, &all_results, &error);
        }
    }
}

fn prompt_user() -> Option<String> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.len() {
        0 => None,
        1 => {
            let arg = args[0].to_lowercase();
            if VALID_OPTS.contains(&arg.as_str()) {
                Some(arg)
            } else {
                eprintln!(
                    "❌ Invalid argument: '{}'. Must be one of: all, daily, weekly, monthly",
                    arg
                );
                process::exit(1)
            }
        }
        _ => {
            eprintln!("❌ Too many arguments. Usage: syncrab_pb [daily|weekly|monthly]");
            process::exit(1)
        }
    }
}

fn normalise_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = env::var_os("HOME") {
            return Path::new(&home).join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn copy_dir(source: &PathBuf, target: &PathBuf) -> Result<(), String> {
    if source.is_file() {
        let file_name = source.file_name().ok_or_else(|| {
            format!(
                "Could not get file name for source file [{}]",
                source.display()
            )
        })?;
        let dest_path = target.join(file_name);

        create_dir_all(target).map_err(|e| {
            format!(
                "Could not create destination directory [{}] because {}",
                target.display(),
                e
            )
        })?;

        copy(source, &dest_path).map_err(|e| {
            format!(
                "Failed to copy file [{}] to [{}] because {}",
                source.display(),
                dest_path.display(),
                e
            )
        })?;

        return Ok(());
    }

    let dir_name = source
        .file_name()
        .ok_or_else(|| format!("Could not get directory name for [{}]", source.display()))?;

    let new_target = target.join(dir_name);
    create_dir_all(&new_target).map_err(|e| {
        format!(
            "Could not create directory [{}] because {}",
            new_target.display(),
            e
        )
    })?;

    for entry in read_dir(source).map_err(|e| {
        format!(
            "Could not read directory [{}] because {}",
            source.display(),
            e
        )
    })? {
        let entry =
            entry.map_err(|e| format!("Failed to read entry in [{}]: {}", source.display(), e))?;
        let path = entry.path();
        copy_dir(&path, &new_target)?; // Recursively copy into new_target
    }

    Ok(())
}

fn fallback_log(log: &Log, results: &Vec<LogResult>, error: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(LOG_PATH) {
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
