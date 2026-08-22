mod config;
mod delete;
mod edit;
mod get;
mod git;
mod init;
mod list;
mod r#move;
mod otp;
mod peek;
mod set;
mod sops;
mod update_keys;

use super::Run;
use crate::secret_store::Store;
use clap::{Parser, Subcommand, builder::StyledStr};
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
            Command::Git(cmd) => cmd.run(store_path),
            Command::Sops(cmd) => cmd.run(store_path),
        }
    }
}

fn complete_location(current: &OsStr) -> Vec<CompletionCandidate> {
    let records = match super::get_store_location() {
        Ok(store_path) => match Store::open(&store_path) {
            Ok(store) => store.list_locations().unwrap_or(Vec::default()),
            Err(_) => Vec::default(),
        },
        Err(_) => Vec::default(),
    };

    do_complete_paths(current, records)
}

fn complete_record_path(current: &OsStr) -> Vec<CompletionCandidate> {
    let records = match super::get_store_location() {
        Ok(store_path) => match Store::open(&store_path) {
            Ok(store) => store.list_records(None).unwrap_or(Vec::default()),
            Err(_) => Vec::default(),
        },
        Err(_) => Vec::default(),
    };

    do_complete_paths(current, records)
}

fn complete_record_selector(current: &OsStr) -> Vec<CompletionCandidate> {
    let args = std::env::args_os().skip(2);
    let args = match super::Cli::try_parse_from(args) {
        Ok(args) => args,
        Err(_) => return Vec::default(),
    };

    let record_path = match args.command {
        Command::Set(command) => command.path,
        Command::Get(command) => command.path,
        _ => return Vec::default(),
    };

    let store_path = match super::get_store_location() {
        Ok(store_path) => store_path,
        Err(_) => return Vec::default(),
    };

    complete_selector_for_record(current, &store_path, &record_path)
}

fn complete_selector_for_record(
    current: &OsStr,
    store_path: &Path,
    record_path: &Path,
) -> Vec<CompletionCandidate> {
    let selectors = match Store::open(store_path) {
        Ok(store) => match store.get_record(record_path) {
            Ok(record) => record.list_attributes().unwrap_or(Vec::default()),
            Err(_) => Vec::default(),
        },
        Err(_) => Vec::default(),
    };

    do_complete_strings(current, selectors, Some("Entry in record".into()))
}

fn do_complete_paths(current: &OsStr, options: Vec<PathBuf>) -> Vec<CompletionCandidate> {
    do_complete_strings(
        current,
        options
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        None,
    )
}

fn do_complete_strings(
    current: &OsStr,
    options: Vec<String>,
    help: Option<StyledStr>,
) -> Vec<CompletionCandidate> {
    let current = current.to_str().unwrap_or("");

    options
        .into_iter()
        .filter(|option| option.starts_with(current))
        .map(|s| CompletionCandidate::new(s).help(help.clone()))
        .collect()
}
