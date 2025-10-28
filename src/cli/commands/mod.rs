mod config;
mod delete;
mod edit;
mod get;
mod git;
mod init;
mod interactive;
mod list;
mod r#move;
mod otp;
mod peek;
mod set;
mod sops;
mod update_keys;

use super::Run;
use crate::secret_store::Store;
use clap::Subcommand;
use clap_complete::CompletionCandidate;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[allow(private_interfaces)]
#[derive(Debug, Subcommand)]
pub(super) enum Command {
    Init(init::Command),
    Config(config::Command),

    #[clap(name = "ls")]
    List(list::Command),
    Peek(peek::Command),
    Edit(edit::Command),
    Set(set::Command),
    Get(get::Command),
    Otp(otp::Command),
    #[clap(name = "mv")]
    Move(r#move::Command),
    #[clap(name = "rm")]
    Delete(delete::Command),

    #[clap(name = "updatekeys")]
    UpdateKeys(update_keys::Command),

    #[clap(alias = "i")]
    Interactive(interactive::Command),

    Git(git::Command),
    Sops(sops::Command),
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        match self {
            Command::Init(cmd) => cmd.run(store_path),
            Command::Config(cmd) => cmd.run(store_path),
            Command::List(cmd) => cmd.run(store_path),
            Command::Peek(cmd) => cmd.run(store_path),
            Command::Edit(cmd) => cmd.run(store_path),
            Command::Set(cmd) => cmd.run(store_path),
            Command::Get(cmd) => cmd.run(store_path),
            Command::Otp(cmd) => cmd.run(store_path),
            Command::Move(cmd) => cmd.run(store_path),
            Command::Delete(cmd) => cmd.run(store_path),
            Command::UpdateKeys(cmd) => cmd.run(store_path),
            Command::Interactive(cmd) => cmd.run(store_path),
            Command::Git(cmd) => cmd.run(store_path),
            Command::Sops(cmd) => cmd.run(store_path),
        }
    }
}

fn complete_location(current: &OsStr) -> Vec<CompletionCandidate> {
    let records = match shellexpand::path::full(super::DEFAULT_STORE_LOCATION) {
        Ok(store_path) => match Store::open(&store_path) {
            Ok(store) => store.list_locations().unwrap_or(Vec::default()),
            Err(_) => Vec::default(),
        },
        Err(_) => Vec::default(),
    };

    do_complete(current, records)
}

fn complete_record(current: &OsStr) -> Vec<CompletionCandidate> {
    let records = match shellexpand::path::full(super::DEFAULT_STORE_LOCATION) {
        Ok(store_path) => match Store::open(&store_path) {
            Ok(store) => store.list_records(None).unwrap_or(Vec::default()),
            Err(_) => Vec::default(),
        },
        Err(_) => Vec::default(),
    };

    do_complete(current, records)
}

fn complete_selector(current: &OsStr) -> Vec<CompletionCandidate> {
    // Try to extract the path argument from the command line
    // We parse COMP_LINE to get the previously entered path argument
    let path = extract_path_from_cmdline();
    
    // Try to extract custom store location from command line, or use default
    let store_location = extract_store_from_cmdline()
        .unwrap_or_else(|| PathBuf::from(super::DEFAULT_STORE_LOCATION));
    
    let attributes = match path {
        Some(path) => {
            match shellexpand::path::full(&store_location) {
                Ok(store_path) => match Store::open(&store_path) {
                    Ok(store) => match store.get_record(&path) {
                        Ok(record) => record.list_attributes().unwrap_or(Vec::default()),
                        Err(_) => Vec::default(),
                    },
                    Err(_) => Vec::default(),
                },
                Err(_) => Vec::default(),
            }
        }
        None => Vec::default(),
    };

    let current = current.to_str().unwrap_or("");
    attributes
        .into_iter()
        .filter(|s| s.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}

fn extract_store_from_cmdline() -> Option<PathBuf> {
    let comp_line = std::env::var("COMP_LINE").ok()?;
    extract_store_from_line(&comp_line)
}

fn extract_store_from_line(comp_line: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = comp_line.split_whitespace().collect();
    
    // Look for -s or --store options
    for (i, part) in parts.iter().enumerate() {
        if *part == "-s" || *part == "--store" {
            // The next part should be the store path
            if i + 1 < parts.len() {
                return Some(PathBuf::from(parts[i + 1]));
            }
        } else if part.starts_with("--store=") {
            // Handle --store=/path/to/store
            return Some(PathBuf::from(part.trim_start_matches("--store=")));
        }
    }
    
    None
}

fn extract_path_from_cmdline() -> Option<PathBuf> {
    // Try to get the command line from environment variables set by the shell
    // COMP_LINE is set by bash during completion
    // Note: This currently only supports bash. Other shells like zsh and fish have
    // different completion systems, but clap_complete handles the translation
    let comp_line = std::env::var("COMP_LINE").ok()?;
    extract_path_from_line(&comp_line)
}

fn extract_path_from_line(comp_line: &str) -> Option<PathBuf> {
    // Parse the command line to extract arguments
    // We're looking for the path argument which comes after the subcommand "get" or "set"
    let parts: Vec<&str> = comp_line.split_whitespace().collect();
    
    // Find the subcommand position
    let subcommand_idx = parts.iter().position(|&s| s == "get" || s == "set")?;
    
    // The path should be the next non-option argument after the subcommand
    // Simple heuristic: look for the first argument that doesn't start with '-'
    // This works because during shell completion, the user would have already typed
    // the path before starting to type the selector
    for part in parts.iter().skip(subcommand_idx + 1) {
        if !part.starts_with('-') {
            return Some(PathBuf::from(part));
        }
    }
    
    None
}

fn do_complete(current: &OsStr, options: Vec<PathBuf>) -> Vec<CompletionCandidate> {
    let current = current.to_str().unwrap_or("");

    options
        .into_iter()
        .filter(|s| s.display().to_string().starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_from_line_get() {
        let result = extract_path_from_line("koishi get mypath");
        assert_eq!(result, Some(PathBuf::from("mypath")));
    }

    #[test]
    fn test_extract_path_from_line_set() {
        let result = extract_path_from_line("koishi set mypath");
        assert_eq!(result, Some(PathBuf::from("mypath")));
    }

    #[test]
    fn test_extract_path_from_line_with_options() {
        let result = extract_path_from_line("koishi get -c mypath");
        assert_eq!(result, Some(PathBuf::from("mypath")));
    }

    #[test]
    fn test_extract_path_from_line_with_store_option() {
        let result = extract_path_from_line("koishi -s /path/to/store get mypath");
        assert_eq!(result, Some(PathBuf::from("mypath")));
    }
    
    #[test]
    fn test_extract_path_from_line_with_store_option_after_subcommand() {
        // Even though this is unusual, we should handle options after the subcommand
        let result = extract_path_from_line("koishi get -c mypath");
        assert_eq!(result, Some(PathBuf::from("mypath")));
    }
    
    #[test]
    fn test_extract_path_from_line_with_long_option() {
        let result = extract_path_from_line("koishi get --copy mypath");
        assert_eq!(result, Some(PathBuf::from("mypath")));
    }

    #[test]
    fn test_extract_path_from_line_no_subcommand() {
        let result = extract_path_from_line("koishi --help");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_path_from_line_no_path() {
        let result = extract_path_from_line("koishi get");
        assert_eq!(result, None);
    }
    
    #[test]
    fn test_extract_store_from_line_short_option() {
        let result = extract_store_from_line("koishi -s /custom/store get mypath");
        assert_eq!(result, Some(PathBuf::from("/custom/store")));
    }
    
    #[test]
    fn test_extract_store_from_line_long_option() {
        let result = extract_store_from_line("koishi --store /custom/store get mypath");
        assert_eq!(result, Some(PathBuf::from("/custom/store")));
    }
    
    #[test]
    fn test_extract_store_from_line_long_option_with_equals() {
        let result = extract_store_from_line("koishi --store=/custom/store get mypath");
        assert_eq!(result, Some(PathBuf::from("/custom/store")));
    }
    
    #[test]
    fn test_extract_store_from_line_no_store() {
        let result = extract_store_from_line("koishi get mypath");
        assert_eq!(result, None);
    }
}
