// Standards ─────────────────────────────────────────────────────
use std::{
    collections::HashSet,
    ffi::OsString,
    fs::metadata,
    fs::{copy, create_dir_all, read_dir, remove_dir_all, remove_file, DirEntry},
    io::Write,
    path::PathBuf,
};

// Progress passed through recursive copies
pub struct CopyProgress<'a> {
    pub total: usize,
    pub count: &'a mut usize,
    pub silent: bool,
}

pub fn copy_dir(
    source: &PathBuf,
    target: &PathBuf,
    mirror: u8,
    progress: &mut CopyProgress<'_>,
) -> Result<(), String> {
    if source.is_file() {
        copy_file(source, target, progress)?;
    } else if source.is_dir() {
        copy_directory(source, target, mirror, progress)?;
    } else {
        return Err(format!(
            "The source [{}] cannot be copied because it is neither a file nor a dir (probably a symlink)",
            source.display()
        ));
    }

    Ok(())
}

fn copy_file(
    source: &PathBuf,
    target: &PathBuf,
    progress: &mut CopyProgress<'_>,
) -> Result<(), String> {
    // Create parent dir if it doesn't exist
    if let Some(parent) = target.parent() {
        create_dir_all(parent).map_err(|e| {
            format!(
                "Could not create parent directory [{}] because {}",
                parent.display(),
                e
            )
        })?;
    }

    if should_copy(source, target)? {
        copy(source, target).map_err(|e| {
            format!(
                "Failed to copy file [{}] to [{}] because [{}]",
                source.display(),
                target.display(),
                e
            )
        })?;
    }

    *progress.count += 1;
    draw_progress_bar(progress);

    Ok(())
}

fn copy_directory(
    source: &PathBuf,
    target: &PathBuf,
    mirror: u8,
    progress: &mut CopyProgress<'_>,
) -> Result<(), String> {
    create_dir_all(target).map_err(|e| {
        format!(
            "Could not create destination directory [{}] because {}",
            target.display(),
            e
        )
    })?;

    *progress.count += 1;
    draw_progress_bar(progress);

    let source_entries = read_source_entries(source)?;

    if mirror == 1 {
        prune_orphans(&source_entries, target)?;
    }

    for entry in source_entries {
        let path = entry.path();
        let new_target = target.join(entry.file_name());
        copy_dir(&path, &new_target, mirror, progress)?;
    }

    Ok(())
}

fn read_source_entries(source: &PathBuf) -> Result<Vec<DirEntry>, String> {
    read_dir(source)
        .map_err(|e| {
            format!(
                "Could not read directory [{}] because {}",
                source.display(),
                e
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            format!(
                "Failed to read source entry in [{}] because [{}]",
                source.display(),
                e
            )
        })
}

// Remove anything in target that does NOT exist in source
fn prune_orphans(source_entries: &[DirEntry], target: &PathBuf) -> Result<(), String> {
    let source_filenames: HashSet<OsString> =
        source_entries.iter().map(|entry| entry.file_name()).collect();

    let Ok(target_itr) = read_dir(target) else {
        return Ok(());
    };

    for entry in target_itr.flatten() {
        if source_filenames.contains(&entry.file_name()) {
            continue;
        }

        let path = entry.path();
        let result = if path.is_dir() {
            remove_dir_all(&path)
        } else {
            remove_file(&path)
        };

        if let Err(e) = result {
            return Err(format!(
                "Failed to delete orphan file [{}] because {}",
                path.display(),
                e
            ));
        }
    }

    Ok(())
}

fn should_copy(source: &PathBuf, target: &PathBuf) -> Result<bool, String> {
    match metadata(target) {
        Ok(dest_metadata) => {
            let source_metadata = metadata(source).map_err(|e| {
                format!(
                    "Could not get metadata of the source [{}] because [{}]",
                    source.display(),
                    e
                )
            })?;

            let source_modif_time = source_metadata.modified().map_err(|e| {
                format!(
                    "Could not get modified time for source [{}] because [{}]",
                    source.display(),
                    e
                )
            })?;

            let dest_modif_time = dest_metadata.modified().map_err(|e| {
                format!(
                    "Could not get modified time for destination [{}] because [{}]",
                    target.display(),
                    e
                )
            })?;

            Ok(source_modif_time > dest_modif_time || source_metadata.len() != dest_metadata.len())
        }
        Err(_) => Ok(true), // If destination doesn't exist, copy
    }
}

pub fn count_children(path: &PathBuf) -> usize {
    let mut count = 1;
    let Ok(entries) = read_dir(path) else {
        return count;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        // Symlinks are counted, not followed, so a loop cannot hang the backup
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        count += if is_dir {
            count_children(&entry_path)
        } else {
            1
        };
    }

    count
}

fn draw_progress_bar(progress: &CopyProgress<'_>) {
    if progress.silent || progress.total == 0 || progress.total == 1 {
        return;
    }

    let ratio = (*progress.count as f32) / (progress.total as f32);
    let bar_width = 50;
    let progress_length = (ratio * bar_width as f32).round() as usize;
    let progress_length = progress_length.min(bar_width);
    let bar = "█".repeat(progress_length) + &"-".repeat(bar_width - progress_length);

    // Use print! instead of println! to avoid a newline after each update
    print!(
        "\r[{} / {}] |{}| {:.2}%",
        progress.count,
        progress.total,
        bar,
        ratio * 100.0
    );

    std::io::stdout().flush().unwrap();
}
