// Standards ─────────────────────────────────────────────────────
use std::{collections::HashMap, env, process};

// Crates ───────────────────────────────────────────────────────
use chrono::{DateTime, Datelike, Local, Timelike};

// mods ──────────────────────────────────────────────────────────
use syncrab::{
    consts::{ALL, DAILY, REAL_TIME, VALID_OPTS, WEEKLY},
    db::db::{get_jobs_to_run, init_db},
    structs::{Job, Log, LogResult},
    utils::{are_paths_valid, copy_dir, count_children, log_results, normalise_path},
};

// Init ──────────────────────────────────────────────────────────
fn main() {
    let now: DateTime<Local> = Local::now();
    let log = Log::new();

    let arg = prompt_user();

    let conn = init_db();
    let jobs: HashMap<&'static str, Vec<Job>> = get_jobs_to_run(
        conn,
        arg.as_deref(),
        now.weekday().to_string(),
        now.hour() as u8,
    );

    let mut success_directories: Vec<LogResult> = Vec::new();
    let mut failed_directories: Vec<LogResult> = Vec::new();

    for (freq, jobs) in jobs.iter() {
        if jobs.is_empty() {
            continue;
        }

        println!("\n🚀 Performing {} backups...", freq);
        println!("──────────────────────────────────────────────────────────\n");

        let jobs_count = jobs.len();
        let jobs_width = jobs_count.to_string().len();

        for (i, job) in jobs.iter().enumerate() {
            let frequency = freq.to_string();
            let source = normalise_path(&job.source);
            let target = normalise_path(&job.target);

            println!(
                "[{}/{}] - Copying [{}] 👉 [{}]",
                format!("{:0width$}", i + 1, width = jobs_width),
                jobs_count,
                source.display(),
                target.display()
            );

            if !are_paths_valid(&frequency, job, &source, &target, &mut failed_directories) {
                continue;
            }

            let dest_path = match source.file_name() {
                Some(name) => target.join(name),
                None => {
                    failed_directories.push(LogResult::new(
                        frequency,
                        format!("Source path [{}] has no file name", source.display()),
                        job.source.clone(),
                        job.target.clone(),
                    ));
                    continue;
                }
            };

            let children_count = count_children(&source);

            match copy_dir(&source, &dest_path, children_count, &mut 0) {
                Ok(_) => success_directories.push(LogResult::new(
                    frequency,
                    "OK".into(),
                    job.source.clone(),
                    job.target.clone(),
                )),
                Err(error) => failed_directories.push(LogResult::new(
                    frequency,
                    error,
                    job.source.clone(),
                    job.target.clone(),
                )),
            };

            if children_count > 1 {
                println!("");
            }
        }

        println!("\n──────────────────────────────────────────────────────────");
        println!("✅ Backups completed successfully!");
    }

    log_results(log, success_directories, failed_directories);
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
                    "❌ Invalid argument: '{}'. Must be one of: {}, {}, {}, {}",
                    ALL, REAL_TIME, DAILY, WEEKLY, arg
                );
                process::exit(1)
            }
        }
        _ => {
            eprintln!(
                "❌ Too many arguments. Usage: syncrab_b [{},{}|{}|{}]",
                ALL, REAL_TIME, DAILY, WEEKLY
            );
            process::exit(1)
        }
    }
}
