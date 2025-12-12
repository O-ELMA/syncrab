// Standards ─────────────────────────────────────────────────────
use std::io::stdout;

// Crates ────────────────────────────────────────────────────────
use crossterm::{event::DisableMouseCapture, execute};

// mods ──────────────────────────────────────────────────────────
use super::super::app::App;
use crate::{
    app::structs::{Component, Modal},
    consts::{SHORTCUT_DAILY, SHORTCUT_REAL_TIME, SHORTCUT_SEARCH, SHORTCUT_WEEKLY},
};

impl App {
    // Misc
    pub fn reset_values(&mut self) {
        if self.active_modal == Some(Modal::Log) {
            self.selected_log = None;
            self.active_modal = None;
            self.active_component = Some(Component::Journal);
            return;
        }

        if self.active_modal == Some(Modal::Replace) {
            for field in [&mut self.replace_with, &mut self.to_replace] {
                field.value.clear();
                field.index = 0;
            }

            self.active_component = None;
            self.active_modal = None;
            return;
        }

        let is_active = self.active_component.as_ref();
        let (is_table, is_search): (bool, bool) = is_active.map_or((false, false), |comp| {
            (comp.is_table(), comp == &Component::Search)
        });

        if is_search || is_table {
            self.event = None;
            self.filter_clicked = false;
            self.active_component = match self.show_journal {
                true => Some(Component::Journal),
                false => None,
            };
            return;
        }

        if self.active_modal == Some(Modal::Job) {
            if self.suggestion_state.active {
                self.suggestion_state.active = false;
                return;
            }

            if let Some(job) = &self.selected_job {
                self.active_component = Some(Component::from_str(job.frequency.as_str()));
            }

            self.event = None;
            self.filter_clicked = false;
            self.active_modal = None;
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

    pub fn enable_component(&mut self, c: char) {
        self.event = None;
        self.active_component = match c {
            SHORTCUT_SEARCH => Some(Component::Search),
            SHORTCUT_REAL_TIME => Some(Component::RealTime),
            SHORTCUT_DAILY => Some(Component::Daily),
            SHORTCUT_WEEKLY => Some(Component::Weekly),
            _ => None,
        };
    }

    pub fn toggle_journal(&mut self) {
        self.active_modal = None;

        if self.show_journal {
            self.show_journal = false;
            self.active_component = None;
        } else {
            self.show_journal = true;
            self.active_component = Some(Component::Journal);
        }
    }

    pub fn open_replace(&mut self) {
        self.active_modal = Some(Modal::Replace);
        self.active_component = Some(Component::ToReplace);
    }

    pub fn exit(&mut self) {
        // Disable mouse event listener
        execute!(stdout(), DisableMouseCapture).unwrap();
        self.exit = true;
    }
}
