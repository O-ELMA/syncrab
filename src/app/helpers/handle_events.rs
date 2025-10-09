// Crates ────────────────────────────────────────────────────────
use color_eyre::{Result, eyre::Ok};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

// mods ──────────────────────────────────────────────────────────
use super::super::app::App;
use crate::{
    app::structs::{Component, Modal},
    consts::{
        ACTIVATE, DEACTIVATE, SCROLL_DOWN, SCROLL_UP, SHORTCUT_DAILY, SHORTCUT_FILTER,
        SHORTCUT_NEW, SHORTCUT_QUIT, SHORTCUT_REAL_TIME, SHORTCUT_SEARCH, SHORTCUT_WEEKLY,
    },
    structs::Job,
    utils::{get_active_jobs, get_active_logs}
};

impl App {
    // Handle key events
    pub fn handle_key(&mut self, event: KeyEvent) -> Result<()> {
        if event.code == KeyCode::Esc {
            self.reset_values();
            return Ok(());
        }

        let modifiers = event.modifiers;
        let code = event.code;

        if let Some(active_input) = self.get_active_input() {
            match (modifiers, code) {
                (KeyModifiers::CONTROL, KeyCode::Char('w')) => active_input.delete_prev_word(),
                (KeyModifiers::CONTROL, KeyCode::Char('v')) => active_input.insert_paste(),
                (_, KeyCode::Char(c)) => active_input.insert_char(c),
                (_, KeyCode::Backspace) => active_input.delete_prev_char(),
                (_, KeyCode::Delete) => active_input.delete_next_char(),
                (_, KeyCode::Left) => active_input.move_cursor_left(),
                (_, KeyCode::Right) => active_input.move_cursor_right(),
                (_, KeyCode::Down | KeyCode::Up)
                    if self.active_component != Some(Component::Search) =>
                {
                    self.event = None;
                    let freq: Option<Component> = match &self.selected_job {
                        Some(job) => Some(Component::from_str(job.frequency.as_str())),
                        None => None,
                    };

                    let comp = self.active_component.clone().unwrap();
                    self.active_component = Some(if code == KeyCode::Down {
                        comp.next(freq)
                    } else {
                        comp.previous(freq)
                    });
                }
                (_, KeyCode::Enter) => match self.active_modal {
                    Some(Modal::Replace) => self.replace_string(),
                    Some(Modal::Job) => self.commit_record(),
                    Some(_) | None => {}
                },
                _ => {}
            }
        } else if let Some(active_table) = self.get_active_table() {
            let idx = active_table.scroll;

            match (modifiers, code) {
                (_, KeyCode::Up) => self.handle_scroll(SCROLL_UP)?,
                (_, KeyCode::Down) => self.handle_scroll(SCROLL_DOWN)?,
                (_, KeyCode::Enter) => {
                    if self.show_journal {
                        if let Some(log) = self.get_active_log(idx).cloned() {
                            self.open_log_modal(log)?;
                        }
                    } else if let Some(job) = self.get_active_job(idx).cloned() {
                        self.open_job_form(job)?;
                    }
                }
                (_, KeyCode::Char(SHORTCUT_NEW)) => {
                    if let Some(comp) = self.active_component.as_ref() {
                        let job = Job::new(comp.to_string());
                        self.open_job_form(job)?;
                    }
                }
                (_, KeyCode::Delete) => {
                    if let Some(job) = self.get_active_job(idx).cloned() {
                        self.delete_record(job);
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char(' ')) => {
                    if let Some(job) = self.get_active_job(idx).cloned() {
                        self.mass_toggle(job.frequency.as_str(), ACTIVATE);
                    }
                }
                (KeyModifiers::ALT, KeyCode::Char(' ')) => {
                    if let Some(job) = self.get_active_job(idx).cloned() {
                        self.mass_toggle(job.frequency.as_str(), DEACTIVATE);
                    }
                }
                (_, KeyCode::Char(' ')) => {
                    if let Some(job) = self.get_active_job(idx).cloned() {
                        self.set_selected_job(job);
                        self.toggle_record();
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                    self.open_replace();
                }
                (_, KeyCode::Char(SHORTCUT_FILTER)) => {
                    self.filter = self.filter.next();
                }
                (
                    _,
                    KeyCode::Char(
                        c @ (SHORTCUT_SEARCH | SHORTCUT_DAILY | SHORTCUT_WEEKLY
                        | SHORTCUT_REAL_TIME),
                    ),
                ) => {
                    self.enable_component(c);
                }
                (_, KeyCode::Tab) => self.toggle_journal(),
                (_, KeyCode::Char(SHORTCUT_QUIT)) => self.exit(),
                _ => {}
            }
        } else {
            match (modifiers, code) {
                (_, KeyCode::Char(SHORTCUT_FILTER)) => {
                    self.filter = self.filter.next();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                    self.open_replace();
                }
                (
                    _,
                    KeyCode::Char(
                        c @ (SHORTCUT_SEARCH | SHORTCUT_DAILY | SHORTCUT_WEEKLY
                        | SHORTCUT_REAL_TIME),
                    ),
                ) => {
                    self.enable_component(c);
                }
                (_, KeyCode::Tab) => {
                    self.toggle_journal();
                }
                (_, KeyCode::Char(SHORTCUT_QUIT)) => {
                    self.exit();
                }
                _ => {}
            }
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
        if let Some(comp) = self.active_component.as_ref() {
            if comp.is_table() {
                let comp_str = comp.to_str();

                let count = if comp == &Component::Journal {
                    get_active_logs(
                        &self.search.value.to_lowercase(), 
                        &self.logs
                    ).len()
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
                        &self.jobs.get(comp_str).unwrap()
                    ).len()
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
            } else if comp.is_field() && comp != &Component::Search {
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
        Ok(())
    }
}
