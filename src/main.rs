// Standards ─────────────────────────────────────────────────────
use std::collections::HashMap;

// Crates ────────────────────────────────────────────────────────
use color_eyre::Result;

// mods ──────────────────────────────────────────────────────────
use syncrab::app::app::App;
use syncrab::app::tui;
use syncrab::db::db::{get_all_jobs, get_logs, init_db};
use syncrab::structs::{Job, Log, Stat};
use syncrab::utils::get_stats;

// Init ──────────────────────────────────────────────────────────
fn main() -> Result<()> {
    let db = init_db();
    let jobs: HashMap<&'static str, Vec<Job>> = get_all_jobs(&db);
    let stats: HashMap<&'static str, Stat> = get_stats(&jobs);
    let logs: Vec<Log> = get_logs(&db);

    color_eyre::install()?;
    let mut terminal = tui::init()?;
    let result = App::default().run(&mut terminal, jobs, logs, stats);
    if let Err(err) = tui::restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {err}"
        );
    }
    Ok(result?)
}

