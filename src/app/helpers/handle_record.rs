// Crates ────────────────────────────────────────────────────────
use color_eyre::Result;

// mods ──────────────────────────────────────────────────────────
use super::super::{
    app::App,
    structs::{Component, Modal},
};
use crate::{
    consts::{DAILY, MONTHLY, WEEK_DAYS, WEEKLY},
    db::db::{delete, insert, mass_replace, mass_update, update},
    structs::{Job, Log},
    utils::capitalise,
};

impl App {
    // Handle record
    pub fn commit_record(&mut self) {
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
                                jobs.iter().fold((0u8, 0u8), |(a, i), j| {
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
                            job.id = Some(id as u8);
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
        }
    }

    pub fn mass_toggle(&mut self, section: &str, active: u8) {
        match mass_update(section, active) {
            Ok(_) => {
                let stat = self.stats.get_mut(section).unwrap();
                let jobs = self.jobs.get_mut(section).unwrap();

                for iter_job in jobs.iter_mut() {
                    iter_job.active = active;
                }

                if active == 1 {
                    stat.active_count = jobs.len() as u8;
                    stat.inactive_count = 0;
                } else {
                    stat.inactive_count = jobs.len() as u8;
                    stat.active_count = 0;
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
        if source.is_empty() || target.is_empty() || hour.is_empty() {
            return false;
        }
        // Parse and validate hour (0-23)
        let _: u8 = match hour.parse() {
            Ok(t) if t <= 23 => t,
            _ => return false,
        };

        match self.selected_job.as_ref().unwrap().frequency.as_str() {
            DAILY => true, // already validated above
            WEEKLY => WEEK_DAYS.contains(&day.as_str()),
            MONTHLY => match day.parse::<u8>() {
                Ok(day) if (1..=31).contains(&day) => true,
                _ => false,
            },
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

        let mut jobs_to_update_map = self.jobs.clone();
        let mut jobs_to_update: Vec<&mut Job> = jobs_to_update_map
            .values_mut()
            .flat_map(|jobs| jobs.iter_mut())
            .collect();

        for job in &mut jobs_to_update {
            if job.source.contains(to_replace) {
                job.source = job.source.replace(to_replace, replace_with);
            }
            if job.target.contains(to_replace) {
                job.target = job.target.replace(to_replace, replace_with);
            }
        }

        match mass_replace(jobs_to_update) {
            Ok(_) => {
                self.jobs = jobs_to_update_map;
                self.reset_values();
            },
            Err(e) => println!("{e}"), //TODO: add popup for the error
        };
    }

    pub fn open_log_modal(&mut self, log: Log) -> Result<()> {
        self.event = None;
        self.selected_log = Some(log);
        self.active_modal = Some(Modal::Log);
        Ok(())
    }
}
