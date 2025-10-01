// mods ──────────────────────────────────────────────────────────
use super::super::app::App;
use crate::{
    app::structs::{Component, InputField, SectionState},
    structs::{Job, Log},
};

impl App {
    // Getters
    pub fn get_active_input(&mut self) -> Option<&mut InputField> {
        match self.active_component.as_ref()? {
            Component::Search => Some(&mut self.search),
            Component::Source => Some(&mut self.source),
            Component::Target => Some(&mut self.target),
            Component::Hour => Some(&mut self.hour),
            Component::Day => Some(&mut self.day),
            Component::ReplaceWith => Some(&mut self.replace_with),
            Component::ToReplace => Some(&mut self.to_replace),
            _ => None,
        }
    }

    pub fn get_active_table(&mut self) -> Option<&SectionState> {
        if let Some(comp) = self.active_component.as_ref() {
            if comp.is_table() {
                return self.states.get(comp.to_str());
            }
        }
        None
    }

    pub fn get_active_job(&mut self, idx: usize) -> Option<Job> {
        let key = self.active_component.as_ref()?.to_str();
        self.jobs.get_mut(key)?.get(idx).cloned()
    }

    pub fn get_active_log(&mut self, idx: usize) -> Option<Log> {
        self.logs.get(idx).cloned()
    }
}
