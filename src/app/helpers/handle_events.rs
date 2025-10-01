// Constants ─────────────────────────────────────────────────────
const SCROLL_DOWN: i32 = 1;
const SCROLL_UP: i32 = -1;
const ACTIVATE: u8 = 1;
const DEACTIVATE: u8 = 0;

// Crates ────────────────────────────────────────────────────────
use color_eyre::{eyre::Ok, Result};
use crossterm::event::{
        KeyCode,
        KeyEvent,
        KeyModifiers,
        MouseButton,
        MouseEvent,
        MouseEventKind
};

// mods ──────────────────────────────────────────────────────────
use super::super::app::App;
use crate::{app::structs::Component, structs::Job};

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
            match code {
                KeyCode::Char(c) => {
                    if modifiers == KeyModifiers::CONTROL {
                        match c {
                            'w' => active_input.delete_prev_word(),
                            'v' => active_input.insert_paste(),
                            _ => {}
                        }
                    } else {
                        active_input.insert_char(c)
                    }
                },
                KeyCode::Backspace => active_input.delete_prev_char(),
                KeyCode::Delete => active_input.delete_next_char(),
                KeyCode::Left => active_input.move_cursor_left(),
                KeyCode::Right => active_input.move_cursor_right(),
                KeyCode::Down | KeyCode::Up if !matches!(self.active_component, Some(Component::Search)) => {
                    self.event = None;
                    if let Some(job) = self.selected_job.as_ref() {
                        let is_daily = job.frequency == "daily";
                        let component = self.active_component.clone().unwrap();
    
                        self.active_component = Some(if code == KeyCode::Down {
                            component.next(is_daily)
                        } else {
                            component.previous(is_daily)
                        });
                    }
                }
                KeyCode::Enter => self.commit_record(),
                _ => {}
            }
        }
        else if let Some(active_table) = self.get_active_table() {
            let idx = active_table.scroll;
    
            match code {
                KeyCode::Up => self.handle_scroll(SCROLL_UP)?,
                KeyCode::Down => self.handle_scroll(SCROLL_DOWN)?,
                KeyCode::Enter => {
                    if self.show_journal {
                        if let Some(log) = self.get_active_log(idx) {
                            self.open_log_modal(log)?;
                        }
                    } else {
                        if let Some(job) = self.get_active_job(idx) {
                            self.open_job_form(job)?;
                        }
                    }
                }
                KeyCode::Char('n') => {
                    if let Some(comp) = self.active_component.as_ref() {
                        let job = Job::new(comp.to_string());
                        self.open_job_form(job)?;
                    }
                }
                KeyCode::Delete => {
                    if let Some(job) = self.get_active_job(idx) {
                        self.delete_record(job);
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(job) = self.get_active_job(idx) {
                        if modifiers == KeyModifiers::CONTROL {
                            self.mass_toggle(job.frequency.as_str(), ACTIVATE);
                        }
                        else if modifiers == KeyModifiers::ALT {
                            self.mass_toggle(job.frequency.as_str(), DEACTIVATE);
                        }
                        else {
                            self.set_selected_job(job);
                            self.toggle_record();
                        }
                    }
                }
                KeyCode::Char('f') => self.filter = self.filter.next(),
                KeyCode::Char(c @ ('s' | 'd' | 'w' | 'm')) => self.enable_component(&c.to_string()),
                KeyCode::Tab => self.toggle_journal(),
                KeyCode::Char('q') => self.exit(),
                _ => {}
            }
        }
        else {
            match code {
                KeyCode::Char('f') => self.filter = self.filter.next(),
                KeyCode::Char(c @ ('s' | 'd' | 'w' | 'm')) => self.enable_component(&c.to_string()),
                KeyCode::Tab => self.toggle_journal(),
                KeyCode::Char('q') => self.exit(),
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
            },
            MouseEventKind::ScrollDown => { self.handle_scroll(SCROLL_DOWN)? },
            MouseEventKind::ScrollUp => { self.handle_scroll(SCROLL_UP)? },
            _ => {}
        }
        Ok(())
    }
    
    pub fn handle_scroll(&mut self, direction: i32) -> Result<()> {
        if let Some(comp) = self.active_component.as_ref() {
            if comp.is_table() {
                let comp_str = comp.to_str();
        
                if let Some(curr_state) = self.states.get_mut(comp_str) {
                    let count = if comp.is_journal() {
                        self.jobs.len()
                    } else {
                        self.stats.get(comp_str).unwrap().count as usize
                    };
                    
                    // Calculate the new index based on direction
                    let i = match curr_state.table_state.selected() {
                        Some(i) if direction > 0 && i < count - 1 => i + 1, // Scroll down
                        Some(i) if direction < 1 && i > 0 => i - 1,  // Scroll up
                        _ if direction < 1 => count.saturating_sub(1),   // If scrolling up and no selection, go to last item
                        _ => 0, // If scrolling down and no selection, go to first item
                    };
        
                    curr_state.table_state.select(Some(i));
                    curr_state.scroll_state = curr_state.scroll_state.position(i);
                    curr_state.scroll = i;
                }
            } else if comp.is_field() && comp != &Component::Search {
                if let Some(job) = self.selected_job.as_ref() {
                    let is_daily = job.frequency == "daily";

                    self.active_component = Some(if direction == 1 {
                        comp.clone().next(is_daily)
                    } else {
                        comp.clone().previous(is_daily)
                    });
                }
            }
        }
        Ok(())
    }
}