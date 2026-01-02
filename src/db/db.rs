// Standards ─────────────────────────────────────────────────────
use std::{collections::HashMap, env, path::PathBuf};

// Crates ───────────────────────────────────────────────────────
use rusqlite::Connection;

// mods ──────────────────────────────────────────────────────────
use crate::{
    consts::{ACTIVE, ALL, DAILY, DB_NAME, INACTIVE, REAL_TIME, WEEKLY},
    structs::{Job, Log, LogResult},
};

// DB ───────────────────────────────────────────────────────────
pub fn db_path() -> PathBuf {
    let exe_path = env::current_exe().expect("❌ Failed to get current executable path");
    let exe_dir = exe_path
        .parent()
        .expect("❌ Executable must be in some directory");
    let cwd = env::current_dir().expect("❌ Failed to get current working directory");

    if exe_dir.ends_with("debug") || exe_dir.ends_with("release") {
        cwd.join(DB_NAME)
    } else {
        exe_dir.join(DB_NAME)
    }
}

fn db_connect() -> Connection {
    Connection::open(db_path())
        .unwrap_or_else(|e| panic!("❌ Failed to connect to the database because [{}]", e))
}

pub fn init_db() -> Connection {
    let conn: Connection = db_connect();

    // Create Jobs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS jobs (
            id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            frequency   TEXT NOT NULL,
            hour        NUMERIC,
            day         TEXT,
            source      TEXT NOT NULL,
            target      TEXT NOT NULL,
            mirror      INTEGER DEFAULT 1,
            active      INTEGER DEFAULT 0
        )",
        [],
    )
    .unwrap_or_else(|e| panic!("❌ Failed to create the jobs table because [{}]", e));

    // Create Logs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            id                      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            startstamp              TEXT NOT NULL,
            endstamp                TEXT NOT NULL,
            status                  TEXT NOT NULL,
            success_count           INTEGER NOT NULL,
            failed_count            INTEGER NOT NULL
        )",
        [],
    )
    .unwrap_or_else(|e| panic!("❌ Failed to create the logs table because [{}]", e));

    // Create LogResults table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS log_results (
            log_id           INTEGER NOT NULL,
            frequency        TEXT NOT NULL,
            message          TEXT NOT NULL,
            source           TEXT NOT NULL,
            target           TEXT NOT NULL
        )",
        [],
    )
    .unwrap_or_else(|e| panic!("❌ Failed to create the log_results table because [{}]", e));

    conn
}

fn get_jobs(conn: &Connection, sql: &str) -> HashMap<&'static str, Vec<Job>> {
    let mut jobs_by_freq: HashMap<&'static str, Vec<Job>> = HashMap::with_capacity(3);
    jobs_by_freq.insert(REAL_TIME, Vec::new());
    jobs_by_freq.insert(DAILY, Vec::new());
    jobs_by_freq.insert(WEEKLY, Vec::new());

    let mut stmt = conn
        .prepare(sql)
        .unwrap_or_else(|e| panic!("❌ Failed to prepare statement because [{}]", e));

    let job_iter = stmt
        .query_map([], |row| {
            Ok(Job {
                id: row.get("id")?,
                frequency: row.get("frequency")?,
                day: row.get("day")?,
                hour: row.get("hour")?,
                source: row.get("source")?,
                target: row.get("target")?,
                mirror: row.get("mirror")?,
                active: row.get("active")?,
            })
        })
        .unwrap_or_else(|e| panic!("❌ Failed to execute query because [{}]", e));

    for job_result in job_iter {
        let job =
            job_result.unwrap_or_else(|e| panic!("❌ Failed to map job from row because [{}]", e));
        jobs_by_freq
            .get_mut(job.frequency.as_str())
            .unwrap()
            .push(job);
    }

    jobs_by_freq
}

pub fn get_all_jobs(conn: &Connection) -> HashMap<&'static str, Vec<Job>> {
    let sql = "SELECT * FROM jobs;";
    get_jobs(&conn, &sql)
}

pub fn get_jobs_to_run(
    conn: Connection,
    args: Option<(String, Option<String>)>,
    day: String,
    hour: u8,
) -> HashMap<&'static str, Vec<Job>> {
    let sql = match args {
        None => {
            format!(
                "SELECT * FROM jobs WHERE active = 1 AND (
                    (frequency = '{daily}' AND hour = {hour}) 
                    OR (frequency = '{weekly}' AND day = '{day}' AND hour = {hour}) 
                );",
                daily = DAILY,
                weekly = WEEKLY,
                hour = hour,
                day = day,
            )
        }

        Some((arg1, arg2)) => {
            if arg1 == ALL {
                match arg2.as_deref() {
                    Some(ACTIVE) => "SELECT * FROM jobs WHERE active = 1;".to_string(),
                    Some(INACTIVE) => "SELECT * FROM jobs WHERE active = 0;".to_string(),
                    _ => "SELECT * FROM jobs;".to_string(),
                }
            } else {
                match arg2.as_deref() {
                    Some(ACTIVE) => format!(
                        "SELECT * FROM jobs WHERE frequency = '{freq}' AND active = 1;",
                        freq = arg1
                    ),
                    Some(INACTIVE) => format!(
                        "SELECT * FROM jobs WHERE frequency = '{freq}' AND active = 0;",
                        freq = arg1
                    ),
                    _ => format!(
                        "SELECT * FROM jobs WHERE frequency = '{freq}';",
                        freq = arg1
                    ),
                }
            }
        }
    };

    get_jobs(&conn, &sql)
}

pub fn get_logs(conn: &Connection) -> Vec<Log> {
    let mut stmt = conn
        .prepare("SELECT * FROM logs;")
        .unwrap_or_else(|e| panic!("❌ Failed to prepare statement because [{}]", e));

    let log_iter = stmt
        .query_map([], |row| {
            Ok(Log {
                id: row.get("id")?,
                startstamp: row.get("startstamp")?,
                endstamp: row.get("endstamp")?,
                status: row.get("status")?,
                success_count: row.get("success_count")?,
                failed_count: row.get("failed_count")?,
                log_results: None,
            })
        })
        .unwrap_or_else(|e| panic!("❌ Failed to execute query because [{}]", e));

    let mut logs: Vec<Log> = log_iter
        .map(|log_result| {
            log_result.unwrap_or_else(|e| panic!("❌ Failed to build log from row because [{}]", e))
        })
        .collect();

    let log_results: Vec<LogResult> = get_log_results(conn);
    let mut results_map: HashMap<u16, Vec<LogResult>> = HashMap::new();

    // Cache the log_results
    for log_result in log_results {
        results_map
            .entry(log_result.log_id.unwrap())
            .or_insert_with(Vec::new)
            .push(log_result);
    }

    for log in &mut logs {
        if let Some(results) = results_map.remove(&log.id.unwrap()) {
            log.log_results = Some(results);
        }
    }

    logs
}

pub fn get_log_results(conn: &Connection) -> Vec<LogResult> {
    let mut stmt = conn
        .prepare("SELECT * FROM log_results;")
        .unwrap_or_else(|e| panic!("❌ Failed to prepare statement because [{}]", e));

    let log_result_iter = stmt
        .query_map([], |row| {
            Ok(LogResult {
                log_id: row.get("log_id")?,
                frequency: row.get("frequency")?,
                message: row.get("message")?,
                source: row.get("source")?,
                target: row.get("target")?,
            })
        })
        .unwrap_or_else(|e| panic!("❌ Failed to execute query because [{}]", e));

    log_result_iter
        .map(|log_result_result| {
            log_result_result
                .unwrap_or_else(|e| panic!("❌ Failed to build log_result from row: [{}]", e))
        })
        .collect()
}

pub fn insert(job: &Job) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute(
        "INSERT INTO jobs (source, target, day, hour, mirror, active, frequency) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &job.source,
            &job.target,
            &job.day.as_ref(),
            &job.hour.clone(),
            &job.mirror,
            &job.active,
            &job.frequency,
        ),
    ) {
        Ok(_) => Ok(conn.last_insert_rowid() as usize),
        Err(e) => Err(format!(
            "Failed to create the job record because [{}]. Record: [{:?}]",
            e, job
        ))
    }
}

pub fn update(job: &Job) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute(
        "UPDATE jobs SET source = ?1, target = ?2, day = ?3, hour = ?4, mirror = ?5, active = ?6, frequency = ?7 WHERE id = ?8",
        (
            &job.source,
            &job.target,
            &job.day.as_ref(),
            &job.hour.clone(),
            &job.mirror,
            &job.active,
            &job.frequency,
            &job.id,
        ),
    ) {
        Ok(res) => Ok(res),
        Err(e) => Err(format!(
            "Failed to update the job record because [{}]. Record: [{:?}]",
            e, job
        ))
    }
}

pub fn delete(id: u16) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute("DELETE FROM jobs WHERE id = ?1", (&id,)) {
        Ok(res) => Ok(res),
        Err(e) => Err(format!(
            "Failed to delete the job record because [{}]. Record Id: [{:?}]",
            e, id
        )),
    }
}

pub fn mass_update(ids: &[u16], update_field: &str, toggle_value: u8) -> Result<usize, String> {
    let mut conn: Connection = db_connect();
    let transaction = conn.transaction().map_err(|e| {
        format!(
            "❌ Failed to initiate a transaction to update jobs because [{}]",
            e
        )
    })?;

    // Scoping stmt to avoid borrowing issue at the transaction
    {
        let query = format!("UPDATE jobs SET {} = ? WHERE id = ?", update_field);

        let mut stmt = transaction.prepare(&query).map_err(|e| {
            format!(
                "❌ Failed to prepare statement to update jobs because [{}]",
                e
            )
        })?;

        for id in ids {
            stmt.execute((toggle_value, id)).map_err(|e| {
                format!(
                    "❌ Failed to execute the statement for job [{}] because [{}]",
                    id, e
                )
            })?;
        }
    }

    transaction.commit().map_err(|e| {
        format!(
            "❌ Failed to commit the statement to update jobs because [{}]",
            e
        )
    })?;

    Ok(1)
}

pub fn mass_replace(jobs: Vec<&mut Job>) -> Result<usize, String> {
    let mut conn: Connection = db_connect();
    let transaction = conn.transaction().map_err(|e| {
        format!(
            "❌ Failed to initiate a transaction to update jobs because [{}]",
            e
        )
    })?;

    // Scoping stmt to avoid borrowing issue at the transaction
    {
        let mut stmt = transaction
            .prepare("UPDATE jobs SET source = ?, target = ? WHERE id = ?")
            .map_err(|e| {
                format!(
                    "❌ Failed to prepare statement to update jobs because [{}]",
                    e
                )
            })?;

        for job in jobs {
            let id = job.id.unwrap();
            stmt.execute((&job.source, &job.target, &id)).map_err(|e| {
                format!(
                    "❌ Failed to execute the statement for job [{}] because [{}]",
                    id, e
                )
            })?;
        }
    }

    transaction.commit().map_err(|e| {
        format!(
            "❌ Failed to commit the statement to update jobs because [{}]",
            e
        )
    })?;

    Ok(1)
}

pub fn insert_log(log: Log) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute(
        "INSERT INTO logs (startstamp, endstamp, status, success_count, failed_count) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &log.startstamp,
            &log.endstamp,
            &log.status,
            &log.success_count,
            &log.failed_count,
        ),
    ) {
        Ok(_) => Ok(conn.last_insert_rowid() as usize),
        Err(e) => Err(format!(
            "❌ Failed to create the log record because [{}]. Record: [{:?}]",
            e, log
        ))
    }
}

pub fn insert_log_resuts(log_id: u16, log_results: Vec<LogResult>) -> Result<usize, String> {
    let mut conn: Connection = db_connect();
    let transaction = conn.transaction()
        .map_err(|e| format!(
            "❌ Failed to initiate a transaction to insert log_results for log_id [{}] because [{}]", log_id, e
        ))?;

    // Scoping stmt to avoid borrowing issue at the transaction
    {
        let mut stmt = transaction.prepare("INSERT INTO log_results (log_id, frequency, message, source, target) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(|e| format!(
                "❌ Failed to prepare statement for log_id [{}] because [{}]", log_id, e
            ))?;

        for log_result in log_results {
            stmt.execute((
                &log_id,
                &log_result.frequency,
                &log_result.message,
                &log_result.source,
                &log_result.target,
            ))
            .map_err(|e| {
                format!(
                    "❌ Failed to execute the statement for log_id [{}] because [{}]",
                    log_id, e
                )
            })?;
        }
    }

    transaction.commit().map_err(|e| {
        format!(
            "❌ Failed to commit the statement for log_id [{}] because [{}]",
            log_id, e
        )
    })?;

    Ok(1)
}
