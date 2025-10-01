// Standards ────────────────────────────────────────────────────
use std::{fmt, str::FromStr};

// Constants ────────────────────────────────────────────────────
const DELIMITERS: [char; 4] = [' ', '/', '.', '-'];

// Crates ───────────────────────────────────────────────────────
use ratatui::{buffer::Buffer, layout::{Position, Rect}, style::Color, widgets::{ScrollbarState, TableState}};
use arboard::Clipboard;

// Structs & Enums ──────────────────────────────────────────────

// Filter
#[derive(Debug, Default, PartialEq)]
pub enum Filter {
    #[default] All,
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
            Filter::All => write!(f, "All"),
            Filter::Active => write!(f, "Active"),
            Filter::Inactive => write!(f, "Inactive"),
        }
    }
}

impl FromStr for Filter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "All" => Ok(Filter::All),
            "Active" => Ok(Filter::Active),
            "Inactive" => Ok(Filter::Inactive),
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
    Source,
    Target,
    Hour,
    Day,
    Daily,
    Weekly,
    Monthly,
}

impl Component {
    pub fn from_str(s: &str) -> Component {
        match s {
            "source" => Component::Source,
            "target" => Component::Target,
            "hour" => Component::Hour,
            "day" => Component::Day,
            "daily" => Component::Daily,
            "weekly" => Component::Weekly,
            "monthly" => Component::Monthly,
            _ => panic!("❌ Could not parse the value [{}] to the enum Component", s),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Component::Search => "search",
            Component::Journal => "journal",
            Component::Source => "source",
            Component::Target => "target",
            Component::Hour => "hour",
            Component::Day => "day",
            Component::Daily => "daily",
            Component::Weekly => "weekly",
            Component::Monthly => "monthly",
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(&self, Component::Search | Component::Source | Component::Target | Component::Hour | Component::Day)
    }

    pub fn is_table(&self) -> bool {
        matches!(&self, Component::Daily | Component::Weekly | Component::Monthly | Component::Journal)
    }

    pub fn is_journal(&self) -> bool {
        self == &Component::Journal
    }

    pub fn next(self, is_daily: bool) -> Self {
        match (is_daily, &self) {
            (true, Component::Source) => Component::Target,
            (true, Component::Target) => Component::Hour,
            (true, Component::Hour) => Component::Source,

            (false, Component::Source) => Component::Target,
            (false, Component::Target) => Component::Hour,
            (false, Component::Hour) => Component::Day,
            (false, Component::Day) => Component::Source,
            _ => self,
        }
    }

    pub fn previous(self, is_daily: bool) -> Self {
        match (is_daily, &self) {
            (true, Component::Hour) => Component::Target,
            (true, Component::Target) => Component::Source,
            (true, Component::Source) => Component::Hour,

            (false, Component::Day) => Component::Hour,
            (false, Component::Hour) => Component::Target,
            (false, Component::Target) => Component::Source,
            (false, Component::Source) => Component::Day,
            _ => self,
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Component::Search => write!(f, "search"),
            Component::Journal => write!(f, "journal"),
            Component::Source => write!(f, "source"),
            Component::Target => write!(f, "target"),
            Component::Hour => write!(f, "hour"),
            Component::Day => write!(f, "day"),
            Component::Daily => write!(f, "daily"),
            Component::Weekly => write!(f, "weekly"),
            Component::Monthly => write!(f, "monthly"),
        }
    }
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
            scroll_state: ScrollbarState::new((data_len.saturating_sub(1)) * 1),
            scroll: 0,
        }
    }
}

// InputField
#[derive(Debug, Default)]
pub struct InputField {
    pub value: String,
    pub index: usize,
}

impl InputField {
    pub fn insert_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.value.insert(index, new_char);
        self.move_cursor_right();
    }

    pub fn insert_paste(&mut self) {
        let index = self.byte_index();
        let last_copied = Clipboard::new().unwrap().get_text().unwrap_or_default();
        let pasted_char_count = last_copied.chars().count();
        self.value.insert_str(index, &last_copied);
        self.index += pasted_char_count;
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

            // One exwa pwease.. oh, no... I'm becoming a rust dev..
            if new_index > 0 {
                new_index -= 1;
            }
    
            // Remove the word from the character vector
            chars.drain(new_index..self.index);
    
            // Update the string and cursor index
            self.value = chars.iter().collect();
            self.index = new_index;
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
        }
    }

    pub fn move_cursor_left(&mut self) {
        let moved = self.index.saturating_sub(1);
        self.index = self.clamp_cursor(moved);
    }

    pub fn move_cursor_right(&mut self) {
        let moved = self.index.saturating_add(1);
        self.index = self.clamp_cursor(moved);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.value.chars().count())
    }

    pub fn set_cursor_position(&mut self, area: Rect, buf: &mut Buffer) {
        let cursor_pos = self.byte_index();
        let cursor_x = area.left() + cursor_pos as u16 + 2;
        let cursor_y = area.top() + 1;

        // Place a cursor at that position
        buf
            .cell_mut(Position::new(cursor_x, cursor_y))
            .unwrap()
            .set_bg(Color::White);
    }

    pub fn byte_index(&self) -> usize {
        self.value
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.index)
            .unwrap_or(self.value.len())
    }
}