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

// mods ──────────────────────────────────────────────────────────
use syncrab::{
    consts::{FAILED, REAL_TIME},
    db::db::{db_path, get_jobs_to_run, init_db, insert_log, insert_log_resuts},
    structs::{Job, Log, LogResult, WatchedJob},
    utils::{are_paths_valid, copy_dir, fallback_log, log_results, normalise_path},
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

    let active_watchers: Arc<Mutex<HashMap<u16, WatchedJob>>> = Arc::new(Mutex::new(HashMap::new()));

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

    let conn = init_db();
    let jobs: Vec<Job> = get_jobs_to_run(
        conn,
        Some(REAL_TIME),
        now.weekday().to_string(),
        now.hour() as u8,
    )
    .get(REAL_TIME)
    .unwrap()
    .clone();

    let job_ids: Vec<u16> = jobs.iter().map(|job| job.id.unwrap()).collect();
    let mut watchers = active_watchers.lock().unwrap();
    let inactive_ids: Vec<u16> = watchers
        .keys()
        .filter(|id| !job_ids.contains(id))
        .cloned()
        .collect();

    let mut failed_directories: Vec<LogResult> = Vec::new();

    // job no longer exists or inactive in the DB
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

    if jobs.is_empty() {
        return;
    }

    for job in jobs {
        // already watching the job
        if watchers.contains_key(&job.id.unwrap()) {
            continue;
        }

        let source = normalise_path(&job.source);
        let target = normalise_path(&job.target);

        if !are_paths_valid(
            &REAL_TIME.into(),
            &job,
            &source,
            &target,
            &mut failed_directories,
        ) {
            continue;
        }

        let (job_tx, job_rx) = channel();

        let mut job_watcher =
            match RecommendedWatcher::new(job_tx.clone(), notify::Config::default()) {
                Ok(w) => w,
                Err(e) => {
                    failed_directories.push(LogResult::new(
                        REAL_TIME.into(),
                        format!("Failed to create file watcher because [{}]", e),
                        job.source.clone(),
                        job.target.clone(),
                    ));
                    continue;
                }
            };

        if let Err(e) = job_watcher.watch(source.as_path(), RecursiveMode::Recursive) {
            failed_directories.push(LogResult::new(
                REAL_TIME.into(),
                format!("Failed to create file watcher because [{}]", e),
                job.source.clone(),
                job.target.clone(),
            ));
            continue;
        }

        watchers.insert(
            job.id.unwrap(),
            WatchedJob {
                job: job.clone(),
                job_watcher,
                job_tx,
            },
        );

        // Spawn a thread for this job watcher
        std::thread::spawn(move || {
            for res in job_rx {
                match res {
                    Ok(event) => sync_file_event(&event, &job),
                    Err(e) => eprintln!("Job watch error: {}\n\n", e),
                }
            }
        });
    }

    if failed_directories.is_empty() {
        return;
    }

    let log = Log {
        id: None,
        startstamp: now.format("%d-%m-%Y %H:%M").to_string(),
        status: FAILED.into(),
        success_count: 0,
        failed_count: failed_directories.len() as u16,
        endstamp: Local::now().format("%d-%m-%Y %H:%M").to_string(),
        log_results: None,
    };
    match insert_log(log.clone()) {
        Ok(id) => {
            if let Err(error) = insert_log_resuts(id as u16, failed_directories.clone()) {
                fallback_log(&log, &failed_directories, &error);
            }
        }
        Err(error) => {
            fallback_log(&log, &failed_directories, &error);
        }
    }
}

fn sync_file_event(event: &Event, job: &Job) {
    if !(event.kind.is_modify() || event.kind.is_create()) {
        return;
    }

    let log = Log::new();

    let mut success_directories: Vec<LogResult> = Vec::new();
    let mut failed_directories: Vec<LogResult> = Vec::new();

    for path in &event.paths {
        let files_names = path.to_str().unwrap().replace(&job.source, "");
        let files_names = files_names.strip_prefix('/').unwrap_or(&files_names);
        let dest_path = Path::new(&job.target).join(files_names);

        match event.kind {
            // Sync into target (create/update/move in)
            Create(File) | Create(Folder) | Modify(Data(Any)) | Modify(Name(To)) => {
                // Copy or overwrite from path to dest_path
                match copy_dir(&path, &dest_path) {
                    Ok(_) => success_directories.push(LogResult::new(
                        REAL_TIME.into(),
                        "OK".into(),
                        job.source.clone(),
                        job.target.clone(),
                    )),
                    Err(error) => failed_directories.push(LogResult::new(
                        REAL_TIME.into(),
                        error,
                        job.source.clone(),
                        job.target.clone(),
                    )),
                };
            }
            // Delete from target (delete/move out)
            Modify(Name(From)) => {
                // Delete dest_path
                if dest_path.is_dir() {
                    match remove_dir_all(dest_path) {
                        Ok(_) => success_directories.push(LogResult::new(
                            REAL_TIME.into(),
                            "OK".into(),
                            job.source.clone(),
                            job.target.clone(),
                        )),
                        Err(error) => failed_directories.push(LogResult::new(
                            REAL_TIME.into(),
                            error.to_string(),
                            job.source.clone(),
                            job.target.clone(),
                        )),
                    };
                } else if dest_path.is_file() {
                    match remove_file(dest_path) {
                        Ok(_) => success_directories.push(LogResult::new(
                            REAL_TIME.into(),
                            "OK".into(),
                            job.source.clone(),
                            job.target.clone(),
                        )),
                        Err(error) => failed_directories.push(LogResult::new(
                            REAL_TIME.into(),
                            error.to_string(),
                            job.source.clone(),
                            job.target.clone(),
                        )),
                    };
                }
            }
            _ => {}
        }
    }

    log_results(log, success_directories, failed_directories);
}
