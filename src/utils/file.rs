use miette::{Context, IntoDiagnostic};
use std::{cmp::Ordering, path::Path, process::Command};
use walkdir::DirEntry;

/// Opens a file in the default editor for interactive editing.
///
/// Will fallback to `vi` if the `EDITOR` environment variable is not set.
pub(crate) fn edit_file_interactive<P: AsRef<Path>>(file_path: P) -> miette::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let _ = Command::new(&editor)
        .arg(file_path.as_ref())
        .status()
        .into_diagnostic()
        .wrap_err(format!("Failed to edit file with {editor}"))?;

    Ok(())
}

pub(crate) fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub(crate) fn sort_by_name_files_before_dirs(a: &DirEntry, b: &DirEntry) -> Ordering {
    if a.path().is_dir() && b.path().is_file() {
        Ordering::Greater
    } else if a.path().is_file() && b.path().is_dir() {
        Ordering::Less
    } else {
        a.file_name().cmp(b.file_name())
    }
}
