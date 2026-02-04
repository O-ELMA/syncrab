// Crates ─────────────────────────────────────────────────────
use ratatui::style::Color;

// shortcuts ──────────────────────────────────────────────────
pub const SHORTCUT_FILTER: char = 'f'; // customisable
pub const SHORTCUT_SEARCH: char = 's'; // customisable
pub const SHORTCUT_REAL_TIME: char = 'r'; // customisable
pub const SHORTCUT_DAILY: char = 'd'; // customisable
pub const SHORTCUT_WEEKLY: char = 'w'; // customisable
pub const SHORTCUT_QUIT: char = 'q'; // customisable
pub const SHORTCUT_NEW: char = 'n'; // customisable

// table style ────────────────────────────────────────────────
pub const ARROW_UP: &str = "⏶"; // customisable
pub const ARROW_DOWN: &str = "⏷"; // customisable
pub const ACTIVE_SLIDER: &str = "┃"; // customisable
pub const SLIDER: &str = "│"; // customisable

// cron ───────────────────────────────────────────────────────
pub const LOG_PATH: &str = "$HOME/syncrab.log"; // customisable
pub const VALID_OPTS_1: [&str; 4] = [ALL, DAILY, WEEKLY, REAL_TIME];
pub const VALID_OPTS_2: [&str; 2] = [ACTIVE, INACTIVE];

// db ─────────────────────────────────────────────────────────
pub const DB_NAME: &str = "syncrab.db";

// components ─────────────────────────────────────────────────
pub const COL_GREEN: Color = Color::Rgb(125, 176, 136);
pub const COL_CYAN: Color = Color::Rgb(135, 173, 161);
pub const COL_BROWN: Color = Color::Rgb(118, 114, 104);
pub const COL_LBROWN: Color = Color::Rgb(189, 163, 124);
pub const COL_BEIGE: Color = Color::Rgb(210, 201, 174);
pub const COL_ORANGE: Color = Color::Rgb(247, 161, 108);
pub const COL_PURPLE: Color = Color::Rgb(159, 132, 181);
pub const COL_BLUE: Color = Color::Rgb(116, 142, 195);
pub const _COL_LBLUE: Color = Color::Rgb(140, 185, 201);
pub const COL_RED: Color = Color::Rgb(227, 113, 122);
pub const COL_MAGENTA: Color = Color::Rgb(196, 126, 145);
pub const COL_GRAY: Color = Color::Rgb(107, 114, 128);

pub const COL_TITLE: Color = COL_CYAN;
pub const COL_BORDER: Color = COL_BROWN;

pub const ACTION_NEW: &str = "🆕 [n] New";
pub const ACTION_MOVE: &str = "🧭 [↑↓] Move";
pub const ACTION_ERASE: &str = "🗑️ [Ctrl+W] Delete Word";
pub const ACTION_DELETE: &str = "🗑️ [Del] Delete";
pub const ACTION_MIRROR: &str = "🪞 [m] Toggle Mirror";
pub const ACTION_ACTIVE: &str = "⏯️ [Space] Toggle Active";
pub const ACTION_DISABLE: &str = "🛑 [Alt+Space] Disable All";
pub const ACTION_ENABLE: &str = "✅ [Ctrl+Space] Enable All";
pub const ACTION_CLONE: &str = "📄📄 [Ctrl+C] Clone";
pub const ACTION_UPDATE: &str = "💾 [Enter] Update";
pub const ACTION_EDIT: &str = "📝 [Enter] Edit";
pub const ACTION_VIEW: &str = "👀 [Enter] View";
pub const ACTION_QUIT: &str = "❌ [q] Quit";
pub const ACTION_CLOSE: &str = "❌ [Esc] Close";
pub const ACTION_LOGS: &str = "📜 [Tab] Logs";
pub const ACTION_BACKUP: &str = "📦 [Tab] Backup Menu";

pub const SEPARATOR: &str = " | ";

// structs ────────────────────────────────────────────────────
pub const DELIMITERS: [char; 5] = [' ', '/', '.', '-', ','];

// handle_events ──────────────────────────────────────────────
pub const SCROLL_DOWN: i32 = 1;
pub const SCROLL_UP: i32 = -1;
pub const ACTIVATE: u8 = 1;
pub const DEACTIVATE: u8 = 0;

// labels ─────────────────────────────────────────────────────
pub const APP_TITLE: &str = "🦀 Syncrab";
pub const APP_SUBTITLE: &str = "Manage and monitor your backup jobs";

pub const REAL_TIME_BACKUPS: &str = "Real-time Backups";
pub const DAILY_BACKUPS: &str = "Daily Backups";
pub const WEEKLY_BACKUPS: &str = "Weekly Backups";

pub const SEARCH: &str = "search";
pub const FILTER: &str = "filter";

pub const REAL_TIME: &str = "realtime";
pub const DAILY: &str = "daily";
pub const WEEKLY: &str = "weekly";

pub const JOURNAL: &str = "journal";
pub const LOG: &str = "log";

pub const SUCCESS: &str = "success";
pub const FAILED: &str = "failed";
pub const PARTIAL: &str = "partial";

pub const ID: &str = "id";
pub const SOURCE: &str = "source";
pub const TARGET: &str = "target";
pub const HOUR: &str = "hour";
pub const DAY: &str = "day";

pub const REPLACE: &str = "replace";
pub const TO_REPLACE: &str = "text to replace";

pub const REPLACE_WITH: &str = "replace with";

pub const ALL: &str = "all";
pub const ACTIVE: &str = "active";
pub const INACTIVE: &str = "inactive";

pub const REAL_TIME_COLS: &[&str; 4] = &["Id", "Source", "Target", "Active"];
pub const DAILY_COLS: &[&str; 6] = &["Id", "Source", "Target", "Hour", "Mirror", "Active"];
pub const WEEKLY_COLS: &[&str; 7] = &["Id", "Source", "Target", "Hour", "Day", "Mirror", "Active"];
pub const JOURNAL_COLS: &[&str; 6] = &[
    "Id",
    "Started at",
    "Ended at",
    "Status",
    "Jobs Succeeded",
    "Jobs Failed",
];
pub const LOG_COLS: &[&str; 4] = &["Type", "Source", "Target", "Message"];

// emojis ─────────────────────────────────────────────────────
pub const EMOJI_ACTIVE: &str = "✅";
pub const EMOJI_INACTIVE: &str = "❌";

pub const EMOJI_STATUS_SUCCESS: &str = "✅";
pub const EMOJI_STATUS_FAILED: &str = "❌";
pub const EMOJI_STATUS_PARTIAL: &str = "⚠️";
pub const EMOJI_STATUS_OTHER: &str = "📊";

pub const EMOJI_STATS: &str = "🗓️";
pub const EMOJI_SECTION: &str = "🕐";
pub const EMOJI_SEARCH: &str = "🔭";
pub const EMOJI_FILTER: &str = "🔍";

// week days ──────────────────────────────────────────────────
pub const WEEK_DAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
