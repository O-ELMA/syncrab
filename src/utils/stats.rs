// Standards ─────────────────────────────────────────────────────
use std::collections::HashMap;

// mods ──────────────────────────────────────────────────────────
use crate::{
    consts::{DAILY, DAILY_BACKUPS, REAL_TIME, REAL_TIME_BACKUPS, WEEKLY, WEEKLY_BACKUPS},
    structs::{Job, Stat},
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
