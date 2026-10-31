// mods ──────────────────────────────────────────────────────────
use crate::{app::structs::Filter, structs::{Job, Log}};

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
