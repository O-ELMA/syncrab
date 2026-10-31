// Standards ─────────────────────────────────────────────────────
use std::{
    collections::HashMap,
    fs::{remove_dir_all, remove_file},
    path::Path,
    sync::{Arc, Mutex, mpsc::channel},
};

// Crates ───────────────────────────────────────────────────────
use chrono::{DateTime, Datelike, Local, Timelike};
use notify::{
    Event,
    EventKind::{Create, Modify},
    RecommendedWatcher, RecursiveMode, Watcher,
    event::{
        CreateKind::{File, Folder},
        DataChange::Any,
        ModifyKind::{Data, Name},
        RenameMode::{From, To},
    },
};
use rusqlite::Connection;

// mods ──────────────────────────────────────────────────────────
use syncrab::{
    consts::{ACTIVE, FAILED, REAL_TIME},
    db::db::{db_path, get_jobs_to_run, init_db},
    structs::{Job, Log, LogResult, WatchedJob},
    utils::{are_paths_valid, copy_dir, log_results, normalise_path, persist_log, CopyProgress},
};

// Init ──────────────────────────────────────────────────────────
fn main() {
    let db_path = db_path();
    let db_path = db_path.as_path();

    let (tx, rx) = channel();

    let mut watcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("❌ Failed to create file watcher because [{}]", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(db_path, RecursiveMode::NonRecursive) {
        eprintln!(
            "❌ Failed to start watching the DB [{}] because [{}]",
            db_path.display(),
            e
        );
        return;
    }

    let active_watchers: Arc<Mutex<HashMap<u16, WatchedJob>>> =
        Arc::new(Mutex::new(HashMap::new()));

    handle_db_event(None, Arc::clone(&active_watchers));

    for res in rx {
        match res {
            Ok(event) => handle_db_event(Some(event), Arc::clone(&active_watchers)),
            Err(e) => eprintln!("❌ Error occured while watching the DB: [{}]", e),
        }
    }
}

fn handle_db_event(event: Option<Event>, active_watchers: Arc<Mutex<HashMap<u16, WatchedJob>>>) {
    if let Some(event) = event {
        if !event.kind.is_modify() {
            return;
        }
    }

    let now: DateTime<Local> = Local::now();

    let mut conn = init_db();
    let jobs = active_real_time_jobs(&conn, &now);
    let job_ids: Vec<u16> = jobs.iter().map(|job| job.id.unwrap()).collect();
    let mut watchers = active_watchers.lock().unwrap();

    // job no longer exists or inactive in the DB
    unwatch_inactive(&mut watchers, &job_ids);

    if jobs.is_empty() {
        return;
    }

    let mut failed_directories: Vec<LogResult> = Vec::new();

    for job in jobs {
        // already watching the job
        if watchers.contains_key(&job.id.unwrap()) {
            continue;
        }

        if let Err(error) = watch_job(&job, &mut watchers) {
            failed_directories.push(LogResult::new(REAL_TIME, &error, &job.source, &job.target));
        }
    }

    if !failed_directories.is_empty() {
        persist_failed_run(&mut conn, now, &failed_directories);
    }
}

fn active_real_time_jobs(conn: &Connection, now: &DateTime<Local>) -> Vec<Job> {
    get_jobs_to_run(
        conn,
        Some((REAL_TIME.to_owned(), Some(ACTIVE.to_owned()))),
        now.weekday().to_string(),
        now.hour() as u8,
    )
    .get(REAL_TIME)
    .unwrap()
    .clone()
}

// Stop watching jobs that became inactive or were deleted from the DB
fn unwatch_inactive(watchers: &mut HashMap<u16, WatchedJob>, job_ids: &[u16]) {
    let inactive_ids: Vec<u16> = watchers
        .keys()
        .filter(|id| !job_ids.contains(id))
        .cloned()
        .collect();

    for id in inactive_ids {
        if let Some(watched_job) = watchers.remove(&id) {
            let WatchedJob {
                mut job_watcher,
                job,
                job_tx,
            } = watched_job;
            let _ = job_watcher.unwatch(&Path::new(&job.source)); // cleanly stop watching
            drop(job_tx); // drop the sender
        }
    }
}

// Validate, create and register the watcher for a single job, then spawn its sync thread
fn watch_job(job: &Job, watchers: &mut HashMap<u16, WatchedJob>) -> Result<(), String> {
    let source = normalise_path(&job.source);
    let target = normalise_path(&job.target);

    let mut invalid_paths = Vec::new();
    if !are_paths_valid(
        &REAL_TIME.to_string(),
        job,
        &source,
        &target,
        &mut invalid_paths,
    ) {
        return Err(invalid_paths
            .pop()
            .map(|result| result.message)
            .unwrap_or_default());
    }

    let (job_tx, job_rx) = channel();

    let mut job_watcher = RecommendedWatcher::new(job_tx.clone(), notify::Config::default())
        .map_err(|e| format!("Failed to create file watcher because [{}]", e))?;

    job_watcher
        .watch(source.as_path(), RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to create file watcher because [{}]", e))?;

    watchers.insert(
        job.id.unwrap(),
        WatchedJob {
            job: job.clone(),
            job_watcher,
            job_tx,
        },
    );

    // Spawn a thread for this job watcher
    let job_clone = job.clone();
    std::thread::spawn(move || {
        for res in job_rx {
            match res {
                Ok(event) => sync_file_event(&event, &job_clone),
                Err(e) => eprintln!("Job watch error: {}\n\n", e),
            }
        }
    });

    Ok(())
}

// Log the reason each job watcher could not be started
fn persist_failed_run(
    conn: &mut Connection,
    now: DateTime<Local>,
    failed_directories: &[LogResult],
) {
    let log = Log {
        id: None,
        startstamp: now.format("%d-%m-%Y %H:%M").to_string(),
        status: FAILED.into(),
        success_count: 0,
        failed_count: failed_directories.len() as u16,
        endstamp: Local::now().format("%d-%m-%Y %H:%M").to_string(),
        log_results: None,
    };

    persist_log(conn, &log, failed_directories);
}

fn sync_file_event(event: &Event, job: &Job) {
    if !(event.kind.is_modify() || event.kind.is_create()) {
        return;
    }

    let log = Log::new();

    let mut success_directories: Vec<LogResult> = Vec::new();
    let mut failed_directories: Vec<LogResult> = Vec::new();

    // Clone once before loop to avoid repeated cloning
    let frequency = REAL_TIME;
    let source = job.source.to_string();
    let target = job.target.to_string();

    for path in &event.paths {
        let files_names = path.to_str().unwrap().replace(&source, "");
        let files_names = files_names.strip_prefix('/').unwrap_or(&files_names);
        let dest_path = Path::new(&target).join(files_names);

        match event.kind {
            // Sync into target (create/update/move in)
            Create(File) | Create(Folder) | Modify(Data(Any)) | Modify(Name(To)) => {
                let mut count = 1;
                // Copy or overwrite from path to dest_path
                match copy_dir(
                    &path,
                    &dest_path,
                    0,
                    &mut CopyProgress {
                        total: 1,
                        count: &mut count,
                        silent: false,
                    },
                ) {
                    Ok(_) => {
                        success_directories.push(LogResult::new(frequency, "OK", &source, &target))
                    }
                    Err(error) => {
                        failed_directories.push(LogResult::new(frequency, &error, &source, &target))
                    }
                };
            }
            // Delete from target (delete/move out)
            Modify(Name(From)) => {
                // Delete dest_path
                if dest_path.is_dir() {
                    match remove_dir_all(dest_path) {
                        Ok(_) => success_directories
                            .push(LogResult::new(frequency, "OK", &source, &target)),
                        Err(error) => failed_directories.push(LogResult::new(
                            frequency,
                            &error.to_string(),
                            &source,
                            &target,
                        )),
                    };
                } else if dest_path.is_file() {
                    match remove_file(dest_path) {
                        Ok(_) => success_directories
                            .push(LogResult::new(frequency, "OK", &source, &target)),
                        Err(error) => failed_directories.push(LogResult::new(
                            frequency,
                            &error.to_string(),
                            &source,
                            &target,
                        )),
                    };
                }
            }
            _ => {}
        }
    }

    let mut conn = init_db();
    log_results(&mut conn, log, success_directories, failed_directories);
}
