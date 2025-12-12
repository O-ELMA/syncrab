// Standards ────────────────────────────────────────────────────
use std::{
    env::var,
    fmt,
    fs::read_dir,
    path::{MAIN_SEPARATOR, Path, PathBuf},
    str::FromStr,
};

// mods ─────────────────────────────────────────────────────────
use crate::consts::{
    ACTIVE, ALL, DAILY, DAY, DELIMITERS, HOUR, INACTIVE, JOURNAL, LOG, REAL_TIME, REPLACE_WITH,
    SEARCH, SOURCE, TARGET, TO_REPLACE, WEEKLY,
};

// Crates ───────────────────────────────────────────────────────
use arboard::Clipboard;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
    widgets::{ListState, ScrollbarState, TableState},
};

// Structs & Enums ──────────────────────────────────────────────

// Filter
#[derive(Debug, Default, PartialEq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Inactive,
}

impl Filter {
    pub fn next(&self) -> Self {
        match self {
            Filter::All => Filter::Active,
            Filter::Active => Filter::Inactive,
            Filter::Inactive => Filter::All,
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Filter::All => write!(f, "{}", ALL),
            Filter::Active => write!(f, "{}", ACTIVE),
            Filter::Inactive => write!(f, "{}", INACTIVE),
        }
    }
}

impl FromStr for Filter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            ALL => Ok(Filter::All),
            ACTIVE => Ok(Filter::Active),
            INACTIVE => Ok(Filter::Inactive),
            _ => Err(format!(
                "Could not parse the value [{}] to the enum Filter",
                s.trim()
            )),
        }
    }
}

// Component
#[derive(Debug, PartialEq, Clone)]
pub enum Component {
    Search,
    Journal,
    Log,
    Source,
    Target,
    Hour,
    Day,
    Daily,
    Weekly,
    RealTime,
    ToReplace,
    ReplaceWith,
}

impl Component {
    pub fn from_str(s: &str) -> Component {
        match s {
            SOURCE => Component::Source,
            TARGET => Component::Target,
            HOUR => Component::Hour,
            DAY => Component::Day,
            DAILY => Component::Daily,
            WEEKLY => Component::Weekly,
            REAL_TIME => Component::RealTime,
            TO_REPLACE => Component::ToReplace,
            REPLACE_WITH => Component::ReplaceWith,
            _ => panic!("❌ Could not parse the value [{}] to the enum Component", s),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Component::Search => SEARCH,
            Component::Journal => JOURNAL,
            Component::Log => LOG,
            Component::Source => SOURCE,
            Component::Target => TARGET,
            Component::Hour => HOUR,
            Component::Day => DAY,
            Component::Daily => DAILY,
            Component::Weekly => WEEKLY,
            Component::RealTime => REAL_TIME,
            Component::ToReplace => TO_REPLACE,
            Component::ReplaceWith => REPLACE_WITH,
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(
            &self,
            Component::Search
                | Component::Source
                | Component::Target
                | Component::Hour
                | Component::Day
                | Component::ToReplace
                | Component::ReplaceWith
        )
    }

    pub fn is_table(&self) -> bool {
        matches!(
            &self,
            Component::Daily
                | Component::Weekly
                | Component::RealTime
                | Component::Journal
                | Component::Log
        )
    }

    pub fn is_listable(&self) -> bool {
        matches!(&self, Component::Source | Component::Target)
    }

    pub fn next(self, freq: Option<Component>) -> Self {
        match (freq, &self) {
            (Some(_), Component::Source) => Component::Target,
            (Some(Component::RealTime), Component::Target) => Component::Source,
            (Some(Component::Daily | Component::Weekly), Component::Target) => Component::Hour,
            (Some(Component::Daily), Component::Hour) => Component::Source,
            (Some(Component::Weekly), Component::Hour) => Component::Day,
            (Some(Component::Weekly), Component::Day) => Component::Source,
            (None, Component::ReplaceWith) => Component::ToReplace,
            (None, Component::ToReplace) => Component::ReplaceWith,
            _ => self,
        }
    }

    pub fn previous(self, freq: Option<Component>) -> Self {
        match (freq, &self) {
            (Some(_), Component::Target) => Component::Source,
            (Some(_), Component::Hour) => Component::Target,
            (Some(Component::RealTime), Component::Source) => Component::Target,
            (Some(Component::Daily), Component::Source) => Component::Hour,
            (Some(Component::Weekly), Component::Day) => Component::Hour,
            (Some(Component::Weekly), Component::Source) => Component::Day,
            (None, Component::ReplaceWith) => Component::ToReplace,
            (None, Component::ToReplace) => Component::ReplaceWith,
            _ => self,
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Component::Search => write!(f, "{}", SEARCH),
            Component::Journal => write!(f, "{}", JOURNAL),
            Component::Log => write!(f, "{}", LOG),
            Component::Source => write!(f, "{}", SOURCE),
            Component::Target => write!(f, "{}", TARGET),
            Component::Hour => write!(f, "{}", HOUR),
            Component::Day => write!(f, "{}", DAY),
            Component::Daily => write!(f, "{}", DAILY),
            Component::Weekly => write!(f, "{}", WEEKLY),
            Component::RealTime => write!(f, "{}", REAL_TIME),

            Component::ToReplace => write!(f, "{}", TO_REPLACE),
            Component::ReplaceWith => write!(f, "{}", REPLACE_WITH),
        }
    }
}

// Modal
#[derive(Debug, PartialEq, Clone)]
pub enum Modal {
    Job,
    Log,
    Replace,
}

// Structs ────────────────────────────────────────────────────────

// SectionState
#[derive(Debug, Default)]
pub struct SectionState {
    pub table_state: TableState,
    pub scroll_state: ScrollbarState,
    pub scroll: usize,
}

impl SectionState {
    pub fn new(data_len: usize) -> Self {
        Self {
            table_state: TableState::default().with_selected(0),
            scroll_state: ScrollbarState::new(data_len.saturating_sub(1)),
            scroll: 0,
        }
    }
}

// InputField
#[derive(Debug, Default)]
pub struct InputField {
    pub value: String,
    pub index: usize,
    pub suggestions: Vec<String>,
}

impl InputField {
    pub fn insert_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.value.insert(index, new_char);
        self.move_cursor_right();

        self.auto_complete();
    }

    pub fn insert_paste(&mut self) {
        let index = self.byte_index();
        let last_copied = Clipboard::new().unwrap().get_text().unwrap_or_default();
        let pasted_char_count = last_copied.chars().count();
        self.value.insert_str(index, &last_copied);
        self.index += pasted_char_count;

        self.auto_complete();
    }

    pub fn delete_prev_word(&mut self) {
        let is_not_cursor_leftmost = self.index != 0;
        if is_not_cursor_leftmost {
            let mut chars: Vec<char> = self.value.chars().collect();
            let mut new_index = self.index;

            // Skip any whitespace before the word
            while new_index > 0 && chars[new_index - 1].is_whitespace() {
                new_index -= 1;
            }

            // Skip the word itself
            while new_index > 0 && !DELIMITERS.contains(&chars[new_index - 1]) {
                new_index -= 1;
            }

            // One extwa pwease.. oh, no... I'm becoming a rust dev..
            if new_index > 0 {
                new_index -= 1;
            }

            // Remove the word from the character vector
            chars.drain(new_index..self.index);

            // Update the string and cursor index
            self.value = chars.iter().collect();
            self.index = new_index;

            self.auto_complete();
        }
    }

    pub fn delete_prev_char(&mut self) {
        let is_not_cursor_leftmost = self.index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.value.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.value.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.value = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();

            self.auto_complete();
        }
    }

    pub fn delete_next_char(&mut self) {
        let is_not_cursor_rightmost = self.index < self.value.len();
        if is_not_cursor_rightmost {
            let current_index = self.index;
            let from_current_to_end = current_index + 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.value.chars().take(current_index);
            // Getting all characters after the selected character.
            let after_char_to_delete = self.value.chars().skip(from_current_to_end);

            // Put all characters together except the next one.
            // The character after the cursor is excluded, effectively deleting it.
            self.value = before_char_to_delete.chain(after_char_to_delete).collect();

            self.auto_complete();
        }
    }

    pub fn move_cursor_left(&mut self) {
        let moved = self.index.saturating_sub(1);
        self.index = self.clamp_cursor(moved);

        self.auto_complete();
    }

    pub fn move_cursor_right(&mut self) {
        let moved = self.index.saturating_add(1);
        self.index = self.clamp_cursor(moved);

        self.auto_complete();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.value.chars().count())
    }

    pub fn set_cursor_position(&mut self, area: Rect, buf: &mut Buffer) {
        let (cursor_x, cursor_y) = self.get_cursor_position(area);

        // Place a cursor at that position
        buf.cell_mut(Position::new(cursor_x, cursor_y))
            .unwrap()
            .set_bg(Color::White);

        self.auto_complete();
    }

    fn get_cursor_position(&mut self, area: Rect) -> (u16, u16) {
        let cursor_pos = self.byte_index();
        let cursor_x = area.left() + cursor_pos as u16 + 2;
        let cursor_y = area.top() + 1;

        (cursor_x, cursor_y)
    }

    pub fn byte_index(&self) -> usize {
        self.value
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.index)
            .unwrap_or(self.value.len())
    }

    fn auto_complete(&mut self) {
        if self.value.is_empty() {
            self.suggestions = vec![];
            return;
        }

        let input = &self.value;
        let input = if self.value.starts_with('~') {
            if let Ok(home) = var("HOME") {
                input.replacen("~", &home, 1)
            } else {
                input.clone()
            }
        } else {
            input.clone()
        };

        // Identify the Operating System's separator ('/' or '\')
        let sep = MAIN_SEPARATOR;

        // Determine the directory to search and the substring to match
        let (search_dir, query) = if input.ends_with(sep) {
            // Case 2: Typed a directory separator (e.g., "/usr/") -> List contents of that dir
            (std::path::PathBuf::from(input), String::new())
        } else {
            // Case 3: Typed partial name (e.g., "/usr/Downl" or "Downl")
            let path = Path::new(&input);
            let parent = path.parent();
            let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // If parent is empty, we search the root "/"
            let dir = match parent {
                Some(p) if p.as_os_str().is_empty() => PathBuf::from(format!("{}", sep)),
                Some(p) => p.to_path_buf(),
                None => PathBuf::from(format!("{}", sep)),
            };
            (dir, file_stem.to_string())
        };

        let mut suggestions = vec![];
        let query_lower = query.to_lowercase();

        // Read the directory and filter results
        if let Ok(entries) = read_dir(search_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // Get the filename to check if it contains the substring
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if file_name.to_lowercase().contains(&query_lower) {
                        suggestions.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        self.suggestions = suggestions;
    }
}

// SuggestionState
#[derive(Debug, Default)]
pub struct SuggestionState {
    pub state: ListState,
    pub paths: Vec<String>,
    pub len: usize,
    pub active: bool,
}
