// Standards ─────────────────────────────────────────────────────
use std::{collections::HashMap, io::stdout};

// Crates ────────────────────────────────────────────────────────
use color_eyre::{Result, eyre::WrapErr};
use crossterm::{
    event::{self, EnableMouseCapture, Event, KeyEventKind, MouseEvent},
    execute,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    widgets::Widget,
};

// mods ──────────────────────────────────────────────────────────
use super::{
    components::{footer, header, modal, search, section, title},
    structs::{Component, Filter, InputField, Modal, SectionState, SuggestionState},
};
use crate::{
    consts::{DAILY, JOURNAL, LOG, REAL_TIME, WEEKLY},
    structs::{Job, Log, Stat},
};

// App ───────────────────────────────────────────────────────────
#[derive(Debug, Default)]
pub struct App {
    pub exit: bool,

    pub jobs: HashMap<&'static str, Vec<Job>>,
    pub stats: HashMap<&'static str, Stat>,
    pub logs: Vec<Log>,

    pub search: InputField,
    pub filter: Filter,
    pub filter_clicked: bool,

    pub source: InputField,
    pub target: InputField,
    pub hour: InputField,
    pub day: InputField,

    pub suggestion_state: SuggestionState,

    pub to_replace: InputField,
    pub replace_with: InputField,

    pub active_modal: Option<Modal>,
    pub active_component: Option<Component>,

    pub states: HashMap<&'static str, SectionState>,
    pub event: Option<MouseEvent>,

    pub show_journal: bool,

    pub selected_job: Option<Job>,
    pub selected_log: Option<Log>,
}

impl App {
    // Init
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        jobs: HashMap<&'static str, Vec<Job>>,
        logs: Vec<Log>,
        stats: HashMap<&'static str, Stat>,
    ) -> Result<()> {
        // Assign records
        self.jobs = jobs;
        self.logs = logs;
        self.stats = stats;

        // Assign tables states
        self.states = HashMap::with_capacity(5);
        self.states.insert(
            REAL_TIME,
            SectionState::new(self.stats.get(REAL_TIME).unwrap().count as usize),
        );
        self.states.insert(
            DAILY,
            SectionState::new(self.stats.get(DAILY).unwrap().count as usize),
        );
        self.states.insert(
            WEEKLY,
            SectionState::new(self.stats.get(WEEKLY).unwrap().count as usize),
        );
        self.states
            .insert(JOURNAL, SectionState::new(self.logs.len()));
        self.states.insert(LOG, SectionState::new(0));

        // Enable mouse event listener
        execute!(stdout(), EnableMouseCapture).unwrap();
        // Init

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("Handle events failed")?;
        }
        Ok(())
    }

    // Render components
    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(event) if event.kind == KeyEventKind::Press => self
                .handle_key(event)
                .wrap_err_with(|| format!("handling key event failed:\n{event:#?}")),
            Event::Mouse(event) => self
                .handle_mouse(event)
                .wrap_err_with(|| format!("handling mouse event failed:\n{event:#?}")),
            _ => Ok(()),
        }
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [
            title_area,
            header_area,
            search_area,
            section_area,
            footer_area,
        ] = vertical_layout.areas(area);

        title(title_area, buf);
        header(header_area, buf, &self.stats);
        search(search_area, buf, self);
        section(section_area, buf, self);
        modal(area, buf, self);
        footer(footer_area, buf, self);
    }
}
