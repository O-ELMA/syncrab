// Crates ────────────────────────────────────────────────────────
use color_eyre::{Result, eyre::Ok};
use crossterm::event::{
    KeyCode,
    KeyCode::{Backspace, Char, Delete, Down, Enter, Esc, Left, Right, Tab, Up},
    KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

// mods ──────────────────────────────────────────────────────────
use super::super::app::App;
use crate::{
    app::structs::{Component, Modal},
    consts::{
        ACTIVATE, DEACTIVATE, SCROLL_DOWN, SCROLL_UP, SHORTCUT_DAILY, SHORTCUT_FILTER,
        SHORTCUT_NEW, SHORTCUT_QUIT, SHORTCUT_REAL_TIME, SHORTCUT_SEARCH, SHORTCUT_WEEKLY,
    },
    structs::Job,
    utils::{get_active_jobs, get_active_logs},
};

impl App {
    // Handle key events
    pub fn handle_key(&mut self, event: KeyEvent) -> Result<()> {
        if let Some(job) = self.pending_delete.take() {
            if event.code == Delete {
                self.delete_record(job);
            }
            return Ok(());
        }

        if event.code == Esc {
            self.reset_values();
            return Ok(());
        }

        let modifiers = event.modifiers;
        let code = event.code;

        let mut should_show_suggestions = false;

        if self.get_active_input().is_some() {
            should_show_suggestions = self.handle_input_key(modifiers, code)?;
        } else if self.get_active_table().is_some() {
            self.handle_table_key(modifiers, code)?;
        } else {
            self.handle_global_key(modifiers, code)?;
        }

        if should_show_suggestions {
            self.suggestion_state.active = true;
            self.suggestion_state.state.select(None);
        }

        Ok(())
    }

    // Keys while an input field is focused
    fn handle_input_key(&mut self, modifiers: KeyModifiers, code: KeyCode) -> Result<bool> {
        let mut should_show_suggestions = false;

        if let Some(active_input) = self.get_active_input() {
            match (modifiers, code) {
                (KeyModifiers::CONTROL, Char('w')) => active_input.delete_prev_word(),
                (KeyModifiers::CONTROL, Char('v')) => active_input.insert_paste(),
                (_, Char(c)) => {
                    active_input.insert_char(c);
                    should_show_suggestions = true;
                }
                (_, Backspace) => active_input.delete_prev_char(),
                (_, Delete) => active_input.delete_next_char(),
                (_, Left) => active_input.move_cursor_left(),
                (_, Right) => active_input.move_cursor_right(),
                (_, Up) => self.handle_scroll(SCROLL_UP)?,
                (_, Down) => self.handle_scroll(SCROLL_DOWN)?,
                (_, Enter) => match self.active_modal {
                    Some(Modal::Replace) => self.replace_string(),
                    Some(Modal::Job) => self.commit_record(),
                    Some(_) | None => {}
                },
                _ => {}
            }
        }

        Ok(should_show_suggestions)
    }

    // Keys while a table is focused
    fn handle_table_key(&mut self, modifiers: KeyModifiers, code: KeyCode) -> Result<()> {
        let idx = match self.get_active_table() {
            Some(table) => table.scroll,
            None => return Ok(()),
        };

        match (modifiers, code) {
            (_, Up) => self.handle_scroll(SCROLL_UP)?,
            (_, Down) => self.handle_scroll(SCROLL_DOWN)?,
            (_, Enter) => {
                if self.show_journal {
                    if let Some(log) = self.get_active_log(idx).cloned() {
                        self.open_log_modal(log)?;
                    }
                } else if let Some(job) = self.get_active_job(idx).cloned() {
                    self.open_job_form(job)?;
                }
            }
            (_, Char(SHORTCUT_NEW)) => {
                if let Some(comp) = self.active_component.as_ref() {
                    let job = Job::new(comp.to_str());
                    self.open_job_form(job)?;
                }
            }
            (_, Delete) => {
                if let Some(job) = self.get_active_job(idx).cloned() {
                    self.pending_delete = Some(job);
                }
            }
            (KeyModifiers::CONTROL, Char('c')) => {
                if let Some(job) = self.get_active_job(idx).cloned() {
                    self.clone_record(job);
                }
            }
            (KeyModifiers::CONTROL, Char(' ')) => {
                if let Some(job) = self.get_active_job(idx).cloned() {
                    self.mass_active(job.frequency.as_str(), ACTIVATE);
                }
            }
            (KeyModifiers::ALT, Char(' ')) => {
                if let Some(job) = self.get_active_job(idx).cloned() {
                    self.mass_active(job.frequency.as_str(), DEACTIVATE);
                }
            }
            (_, Char(' ')) => {
                if let Some(job) = self.get_active_job(idx).cloned() {
                    self.set_selected_job(job);
                    self.toggle_active();
                }
            }
            (_, Char('m')) => {
                if let Some(job) = self.get_active_job(idx).cloned() {
                    self.set_selected_job(job);
                    self.toggle_mirror();
                }
            }
            (KeyModifiers::CONTROL, Char('r')) => {
                self.open_replace();
            }
            (_, Char(SHORTCUT_FILTER)) => {
                self.filter = self.filter.next();
            }
            (
                _,
                Char(
                    c @ (SHORTCUT_SEARCH | SHORTCUT_DAILY | SHORTCUT_WEEKLY
                    | SHORTCUT_REAL_TIME),
                ),
            ) => {
                self.enable_component(c);
            }
            (_, Tab) => self.toggle_journal(),
            (_, Char(SHORTCUT_QUIT)) => self.exit(),
            _ => {}
        }

        Ok(())
    }

    // Keys with no component focused
    fn handle_global_key(&mut self, modifiers: KeyModifiers, code: KeyCode) -> Result<()> {
        match (modifiers, code) {
            (_, Char(SHORTCUT_FILTER)) => {
                self.filter = self.filter.next();
            }
            (KeyModifiers::CONTROL, Char('r')) => {
                self.open_replace();
            }
            (
                _,
                Char(
                    c @ (SHORTCUT_SEARCH | SHORTCUT_DAILY | SHORTCUT_WEEKLY
                    | SHORTCUT_REAL_TIME),
                ),
            ) => {
                self.enable_component(c);
            }
            (_, Tab) => {
                self.toggle_journal();
            }
            (_, Char(SHORTCUT_QUIT)) => {
                self.exit();
            }
            _ => {}
        }

        Ok(())
    }

    // Handle mouse events
    pub fn handle_mouse(&mut self, event: MouseEvent) -> Result<()> {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.filter_clicked = false;
                self.event = Some(event);
            }
            MouseEventKind::ScrollDown => self.handle_scroll(SCROLL_DOWN)?,
            MouseEventKind::ScrollUp => self.handle_scroll(SCROLL_UP)?,
            _ => {}
        }
        Ok(())
    }

    pub fn handle_scroll(&mut self, direction: i32) -> Result<()> {
        let active_input_suggestions_len = self
            .get_active_input()
            .map(|input| input.suggestions.len())
            .unwrap_or(0);

        let (is_table, is_field) = match self.active_component.as_ref() {
            Some(comp) => (comp.is_table(), comp.is_field() && comp != &Component::Search),
            None => (false, false),
        };

        if is_table {
            self.scroll_table(direction);
        } else if is_field {
            self.navigate_field(direction, active_input_suggestions_len);
        }

        Ok(())
    }

    // Scroll the focused table by one position
    fn scroll_table(&mut self, direction: i32) {
        let Some(comp) = self.active_component.as_ref() else {
            return;
        };
        let comp_str = comp.to_str();

        let count = if comp == &Component::Journal {
            get_active_logs(&self.search.value.to_lowercase(), &self.logs).len()
        } else if comp == &Component::Log {
            self.selected_log
                .as_ref()
                .unwrap()
                .log_results
                .as_ref()
                .unwrap()
                .len()
        } else {
            get_active_jobs(
                &self.search.value.to_lowercase(),
                &self.filter,
                &self.jobs.get(comp_str).unwrap(),
            )
            .len()
        };

        if let Some(curr_state) = self.states.get_mut(comp_str) {
            // Calculate the new index based on direction
            let i = match curr_state.table_state.selected() {
                Some(i) if direction > 0 && i < count - 1 => i + 1, // Scroll down
                Some(i) if direction < 1 && i > 0 => i - 1,         // Scroll up
                _ if direction < 1 => count.saturating_sub(1), // If scrolling up and no selection, go to last item
                _ => 0, // If scrolling down and no selection, go to first item
            };

            curr_state.table_state.select(Some(i));
            curr_state.scroll_state = curr_state.scroll_state.position(i);
            curr_state.scroll = i;
        }
    }

    // Move focus between fields, or the suggestion dropdown when open
    fn navigate_field(&mut self, direction: i32, active_input_suggestions_len: usize) {
        let Some(comp) = self.active_component.as_ref() else {
            return;
        };

        if comp.is_listable() && self.suggestion_state.active && active_input_suggestions_len > 1 {
            if direction == 1 {
                self.suggestion_state.state.select_next();
            } else {
                self.suggestion_state.state.select_previous();
            };
        } else {
            let freq: Option<Component> = self
                .selected_job
                .as_ref()
                .map(|job| Component::from_str(job.frequency.as_str()));

            self.active_component = Some(if direction == 1 {
                comp.clone().next(freq)
            } else {
                comp.clone().previous(freq)
            });
        }
    }
}
