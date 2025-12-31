// Standards ─────────────────────────────────────────────────────
use std::collections::HashSet;

// Crates ────────────────────────────────────────────────────────
use color_eyre::Result;

// mods ──────────────────────────────────────────────────────────
use super::super::{
    app::App,
    structs::{Component, Modal},
};
use crate::{
    consts::{DAILY, REAL_TIME, WEEK_DAYS, WEEKLY},
    db::db::{delete, insert, mass_replace, mass_update, update},
    structs::{Job, Log},
    utils::{capitalise, get_active_jobs},
};

impl App {
    // Handle record
    pub fn commit_record(&mut self) {
        if self.suggestion_state.active {
            if let Some(selected) = self.suggestion_state.state.selected() {
                let input = self.suggestion_state.paths[selected].clone();
                let input_count = input.chars().count();

                if let Some(active_input) = self.get_active_input() {
                    active_input.value = input;
                    active_input.index = input_count;
                }
            }
            self.suggestion_state.active = false;
            return;
        }

        if !self.is_record_valid() {
            return;
        }

        if let Some(job) = &mut self.selected_job {
            job.source = self.source.value.clone();
            job.target = self.target.value.clone();
            job.hour = self.hour.value.parse().unwrap();

            let day = &self.day.value;
            if !day.is_empty() {
                let capitalised_day = capitalise(&day);
                job.day = Some(capitalised_day);
            }

            let freq = job.frequency.as_str();
            let job_id = job.id;
            let res = match job_id {
                Some(_) => update(job),
                None => insert(job),
            };

            match res {
                Ok(id) => {
                    let stat = self.stats.get_mut(freq).unwrap();
                    let jobs = self.jobs.get_mut(freq).unwrap();

                    match job_id {
                        Some(job_id) => {
                            if let Some(iter_job) =
                                jobs.iter_mut().find(|iter_job| iter_job.id == Some(job_id))
                            {
                                *iter_job = job.clone();
                            }

                            let (active_count, inactive_count) =
                                jobs.iter().fold((0u16, 0u16), |(a, i), j| {
                                    if j.active == 1 {
                                        (a + 1, i)
                                    } else {
                                        (a, i + 1)
                                    }
                                });

                            stat.active_count = active_count;
                            stat.inactive_count = inactive_count;
                        }
                        None => {
                            job.id = Some(id as u16);
                            jobs.push(job.clone());

                            stat.count += 1;
                            stat.inactive_count += 1;
                        }
                    };

                    self.active_component = None;
                    self.reset_values();
                }
                Err(e) => println!("{e}"), //TODO: add popup for the error
            }
        }
    }

    pub fn clone_record(&mut self, mut job: Job) {
        job.id = None;

        self.source.value = job.source.clone();
        self.target.value = job.target.clone();
        self.hour.value = job.hour.to_string();
        self.day.value = job.day.clone().unwrap_or_default();

        self.selected_job = Some(job);

        self.commit_record();
    }

    pub fn delete_record(&mut self, job: Job) {
        //TODO: popup to confirm delete

        match delete(job.id.unwrap()) {
            Ok(_) => {
                let freq = job.frequency.as_str();

                self.jobs
                    .get_mut(freq)
                    .unwrap()
                    .retain(|iter_job| iter_job.id != job.id);

                let stat = self.stats.get_mut(freq).unwrap();
                stat.count -= 1;
                if job.active == 1 {
                    stat.active_count -= 1;
                } else {
                    stat.inactive_count -= 1;
                }
            }
            Err(e) => println!("{e}"), //TODO: add popup for the error
        }
    }

    pub fn toggle_record(&mut self) {
        if let Some(job) = &mut self.selected_job {
            job.active ^= 1;

            let freq = job.frequency.as_str();

            match update(job) {
                Ok(_) => {
                    let stat = self.stats.get_mut(freq).unwrap();
                    let jobs = self.jobs.get_mut(freq).unwrap();

                    if let Some(iter_job) = jobs
                        .iter_mut()
                        .find(|iter_job| iter_job.id == Some(job.id.unwrap()))
                    {
                        *iter_job = job.clone();
                    }

                    if job.active == 1 {
                        stat.active_count += 1;
                        stat.inactive_count -= 1;
                    } else {
                        stat.active_count -= 1;
                        stat.inactive_count += 1;
                    }
                }
                Err(e) => println!("{e}"), //TODO: add popup for the error
            }

            self.selected_job = None;
        }
    }

    pub fn mass_toggle(&mut self, section: &str, active: u8) {
        let jobs = match self.jobs.get(section) {
            Some(j) => j,
            None => return,
        };

        let found_jobs = get_active_jobs(&self.search.value.to_lowercase(), &self.filter, jobs);
        if found_jobs.is_empty() {
            return;
        }

        let ids_to_update: Vec<u16> = found_jobs.iter().filter_map(|j| j.id).collect();
        let ids_set: HashSet<u16> = ids_to_update.iter().cloned().collect();

        match mass_update(active, &ids_to_update) {
            Ok(_) => {
                if let Some(jobs) = self.jobs.get_mut(section) {
                    let mut new_active_count: u16 = 0;
                    let mut new_inactive_count: u16 = 0;

                    for job in jobs.iter_mut() {
                        if let Some(id) = job.id {
                            if ids_set.contains(&id) {
                                job.active = active;
                            }
                        }

                        if job.active == 1 {
                            new_active_count += 1;
                        } else {
                            new_inactive_count += 1;
                        }
                    }

                    if let Some(stat) = self.stats.get_mut(section) {
                        stat.active_count = new_active_count;
                        stat.inactive_count = new_inactive_count;
                    }
                }
            }
            Err(e) => println!("{e}"), //TODO: add popup for the error
        }
    }

    fn is_record_valid(&self) -> bool {
        let source = self.source.value.as_str();
        let target = self.target.value.as_str();
        let hour = self.hour.value.as_str();
        let day = self.day.value.to_lowercase();

        // Check if essential fields are non-empty
        if source.is_empty() || target.is_empty() {
            return false;
        }

        match self.selected_job.as_ref().unwrap().frequency.as_str() {
            REAL_TIME => true,
            DAILY => self.is_hour_valid(hour),
            WEEKLY => self.is_hour_valid(hour) && WEEK_DAYS.contains(&day.as_str()),
            _ => false,
        }
    }

    // Parse and validate hour (0-23)
    fn is_hour_valid(&self, hour: &str) -> bool {
        match hour.parse::<u8>() {
            Ok(h) if h <= 23 => true,
            _ => false,
        }
    }

    pub fn open_job_form(&mut self, job: Job) -> Result<()> {
        self.event = None;
        self.active_component = Some(Component::Source);
        self.set_selected_job(job);
        self.active_modal = Some(Modal::Job);
        Ok(())
    }

    pub fn set_selected_job(&mut self, job: Job) {
        self.selected_job = Some(job.clone());

        self.source.value = job.source;
        self.source.index = self.source.value.len();

        self.target.value = job.target;
        self.target.index = self.target.value.len();

        self.hour.value = job.hour.to_string();
        self.hour.index = self.hour.value.len();

        self.day.value = job.day.unwrap_or_default();
        self.day.index = self.day.value.len();
    }

    pub fn replace_string(&mut self) {
        let (to_replace, replace_with): (&String, &String) =
            (&self.to_replace.value, &self.replace_with.value);

        if to_replace.is_empty() || replace_with.is_empty() {
            return;
        }

        let mut ids: HashSet<u16> = HashSet::new();
        let mut jobs_to_update: Vec<&mut Job> = Vec::new();
        for job_list in self.jobs.values_mut() {
            let found_jobs =
                get_active_jobs(&self.search.value.to_lowercase(), &self.filter, job_list);
            if found_jobs.is_empty() {
                continue;
            }

            ids.clear();

            for job in found_jobs {
                ids.insert(job.id.unwrap());
            }

            for job in job_list.iter_mut() {
                if ids.contains(&job.id.unwrap()) {
                    if job.source.contains(to_replace) {
                        job.source = job.source.replace(to_replace, replace_with);
                    }
                    if job.target.contains(to_replace) {
                        job.target = job.target.replace(to_replace, replace_with);
                    }

                    jobs_to_update.push(job);
                }
            }
        }

        if jobs_to_update.is_empty() {
            return;
        }

        match mass_replace(jobs_to_update) {
            Ok(_) => {
                self.reset_values();
            }
            Err(e) => println!("{e}"), //TODO: add popup for the error
        };
    }

    pub fn open_log_modal(&mut self, log: Log) -> Result<()> {
        self.event = None;
        self.active_component = Some(Component::Log);
        self.selected_log = Some(log);
        self.active_modal = Some(Modal::Log);
        Ok(())
    }
}
