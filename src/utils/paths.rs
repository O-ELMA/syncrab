// Standards ─────────────────────────────────────────────────────
use std::{env, path::{Path, PathBuf}};

pub fn normalise_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = env::var_os("HOME") {
            return Path::new(&home).join(&path[2..]);
        }
    }
    PathBuf::from(path)
}
