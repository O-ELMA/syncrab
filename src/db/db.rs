// Standards ─────────────────────────────────────────────────────
use std::collections::HashMap;

// Crates ───────────────────────────────────────────────────────
use rusqlite::Connection;

// mods ──────────────────────────────────────────────────────────
use crate::structs::{Job, Log, LogResult};

// Constants ─────────────────────────────────────────────────────
const DB_PATH: &'static str = "syncrab.db";
 
// DB ───────────────────────────────────────────────────────────
fn db_connect() -> Connection {
    Connection::open(DB_PATH)
        .unwrap_or_else(|error| panic!(
            "❌ Failed to connect to the database because [{}]",
            error
        ))
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
            active      INTEGER DEFAULT 0
        )",
        []
    )
    .unwrap_or_else(|error| panic!("❌ Failed to create the jobs table because [{}]", error));

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
        []
    )
    .unwrap_or_else(|error| panic!("❌ Failed to create the logs table because [{}]", error));

    // Create LogResults table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS log_results (
            log_id           INTEGER NOT NULL,
            frequency        TEXT NOT NULL,
            message          TEXT NOT NULL,
            source           TEXT NOT NULL,
            target           TEXT NOT NULL
        )",
        []
    )
    .unwrap_or_else(|error| panic!("❌ Failed to create the log_results table because [{}]", error));

    conn
}

fn get_jobs(conn: &Connection, sql: &str) -> HashMap<&'static str, Vec<Job>> {
    let mut jobs_by_freq: HashMap<&'static str, Vec<Job>> = HashMap::with_capacity(3);
    jobs_by_freq.insert("daily", Vec::new());
    jobs_by_freq.insert("weekly", Vec::new());
    jobs_by_freq.insert("monthly", Vec::new());

    let mut stmt = conn
        .prepare(sql)
        .unwrap_or_else(|error| panic!("❌ Failed to prepare statement because [{}]", error));

    let job_iter = stmt
        .query_map([], |row| {
            Ok(Job {
                id: row.get("id")?,
                frequency: row.get("frequency")?,
                day: row.get("day")?,
                hour: row.get("hour")?,
                source: row.get("source")?,
                target: row.get("target")?,
                active: row.get("active")?,
            })
        })
        .unwrap_or_else(|error| panic!("❌ Failed to execute query because [{}]", error));

    for job_result in job_iter {
        let job = job_result.unwrap_or_else(|error| panic!("❌ Failed to map job from row because [{}]", error));
        jobs_by_freq.get_mut(job.frequency.as_str()).unwrap().push(job);
    }

    jobs_by_freq
}

pub fn get_all_jobs(conn: &Connection) -> HashMap<&'static str, Vec<Job>> {
    let sql = "SELECT * FROM jobs;";
    get_jobs(&conn, &sql)
}

pub fn get_jobs_to_run(conn: Connection, freq: Option<&str>, day_str: String, day_int: u8, hour: u8) -> HashMap<&'static str, Vec<Job>> {
    let sql = match freq {
        Some(freq @ ("daily" | "weekly" | "monthly")) => {
            format!(
                "SELECT * FROM jobs WHERE frequency = '{freq}';",
                freq = freq,
            )
        }
        Some("all") => "SELECT * FROM jobs;".to_string(),
        Some(_) | None => {
            format!(
                "SELECT * FROM jobs WHERE (frequency = 'daily' AND hour = {hour}) \
                OR (frequency = 'weekly' AND day = {day_int} AND hour = {hour}) \
                OR (frequency = 'monthly' AND day = '{day_str}' AND hour = {hour});",
                hour = hour,
                day_int = day_int,
                day_str = day_str,
            )
        }
    };

    get_jobs(&conn, &sql)
}

pub fn get_logs(conn: &Connection) -> Vec<Log> {
    let mut stmt = conn
        .prepare("SELECT * FROM logs;")
        .unwrap_or_else(|error| panic!("❌ Failed to prepare statement because [{}]", error));

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
        .unwrap_or_else(|error| panic!("❌ Failed to execute query because [{}]", error));

    let mut logs: Vec<Log> = log_iter
        .map(|log_result| log_result.unwrap_or_else(|error| panic!("❌ Failed to build log from row: [{}]", error)))
        .collect();

    let log_results: Vec<LogResult> = get_log_results(conn);
    let mut results_map: HashMap<u8, Vec<LogResult>> = HashMap::new();
    
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
        .unwrap_or_else(|error| panic!("❌ Failed to prepare statement because [{}]", error));

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
        .unwrap_or_else(|error| panic!("❌ Failed to execute query because [{}]", error));

    log_result_iter
        .map(|log_result_result| log_result_result.unwrap_or_else(|error| panic!("❌ Failed to build log_result from row: [{}]", error)))
        .collect()
}

pub fn insert(job: &Job) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute(
        "INSERT INTO jobs (source, target, day, hour, active, frequency) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &job.source,
            &job.target,
            &job.day.as_ref(),
            &job.hour.clone(),
            &job.active,
            &job.frequency,
        ),
    ) {
        Ok(_) => Ok(conn.last_insert_rowid() as usize),
        Err(error) => Err(format!(
            "Failed to create the job record because [{}]. Record: [{:?}]",
            error, job
        ))
    }
}

pub fn update(job: &Job) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute(
        "UPDATE jobs SET source = ?1, target = ?2, day = ?3, hour = ?4, active = ?5, frequency = ?6 WHERE id = ?7",
        (
            &job.source,
            &job.target,
            &job.day.as_ref(),
            &job.hour.clone(),
            &job.active,
            &job.frequency,
            &job.id,
        ),
    ) {
        Ok(res) => Ok(res),
        Err(error) => Err(format!(
            "Failed to update the job record because [{}]. Record: [{:?}]",
            error, job
        ))
    }
}

pub fn delete(id: u8) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute(
        "DELETE FROM jobs WHERE id = ?1",
        (&id,)
    ) {
        Ok(res) => Ok(res),
        Err(error) => Err(format!(
            "Failed to delete the job record because [{}]. Record Id: [{:?}]",
            error, id
        ))
    }
}

pub fn mass_update(section: &str, active: u8) -> Result<usize, String> {
    let conn: Connection = db_connect();
    match conn.execute(
        "UPDATE jobs SET active = ?1 WHERE frequency = ?2",
        (active, section),
    ) {
        Ok(res) => Ok(res),
        Err(error) => Err(format!(
            "Failed to update job records because [{}].",
            error
        ))
    }
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
        Err(error) => Err(format!(
            "❌ Failed to create the log record because [{}]. Record: [{:?}]",
            error, log
        ))
    }
}

pub fn insert_log_resuts(log_id: u8, log_results: Vec<LogResult>) -> Result<usize, String> {
    let mut conn: Connection = db_connect();
    let transaction = conn.transaction()
        .map_err(|error| format!(
            "❌ Failed to initiate a transaction to insert log_results for log_id [{}] because [{}]", log_id, error
        ))?;

    // Scoping stmt to avoid borrowing issue at the transaction
    {
        let mut stmt = transaction.prepare("INSERT INTO log_results (log_id, frequency, message, source, target) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(|error| format!(
                "❌ Failed to prepare statement for log_id [{}] because [{}]", log_id, error
            ))?;

        for log_result in log_results {
            stmt.execute((
                &log_id,
                &log_result.frequency,
                &log_result.message,
                &log_result.source,
                &log_result.target,
            ))
            .map_err(|error| { format!(
                "❌ Failed to execute the statement for log_id [{}] because [{}]", log_id, error
            )})?;
        }
    }

    transaction.commit().map_err(|error| format!(
        "❌ Failed to commit the statement for log_id [{}] because [{}]", log_id, error
    ))?;

    Ok(1)
}