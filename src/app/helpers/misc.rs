// Standards ─────────────────────────────────────────────────────
use std::io::stdout;

// Crates ────────────────────────────────────────────────────────
use crossterm::{
    event::DisableMouseCapture,
    execute
};

// mods ──────────────────────────────────────────────────────────
use super::super::app::App;
use crate::app::structs::Component;

impl App {
    // Misc
    pub fn reset_values(&mut self) {
        if self.selected_log.is_some() {
            self.selected_log = None;
        }

        let is_active = self.active_component.as_ref();
        let (is_table, is_search): (bool, bool) = is_active
            .map_or((false, false), |comp| (comp.is_table(), comp == &Component::Search));

        if is_search || is_table {
            self.event = None;
            self.filter_clicked = false;
            self.active_component = match self.show_journal {
                true => Some(Component::Journal),
                false => None
            };
            return;
        }

        if self.show_form {
            if let Some(job) = &self.selected_job {
                self.active_component = Some(Component::from_str(job.frequency.as_str()));
            }

            self.event = None;
            self.filter_clicked = false;
            self.show_form = false;
            self.selected_job = None;

            for field in [
                &mut self.search,
                &mut self.source,
                &mut self.target,
                &mut self.hour,
                &mut self.day,
            ] {
                field.value.clear();
                field.index = 0;
            }
        }
    }
    
    pub fn enable_component(&mut self, c: &str) {
        self.event = None;
        self.active_component = match c {
            "s" => Some(Component::Search),
            "d" => Some(Component::Daily),
            "m" => Some(Component::Monthly),
            "w" => Some(Component::Weekly),
            _ => None,
        };
    }
    
    pub fn toggle_journal(&mut self) {
        if self.show_journal {
            self.show_journal = false;
            self.active_component = None;
        } else {
            self.show_journal = true;
            self.active_component = Some(Component::Journal);
        }
    }

    pub fn exit(&mut self) {
        // Disable mouse event listener
        execute!(stdout(), DisableMouseCapture).unwrap();
        self.exit = true;
    }
}