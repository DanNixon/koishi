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
    let records = match super::get_store_location() {
        Ok(store_path) => match Store::open(&store_path) {
            Ok(store) => store.list_locations().unwrap_or(Vec::default()),
            Err(_) => Vec::default(),
        },
        Err(_) => Vec::default(),
    };

    do_complete(current, records)
}

fn complete_record(current: &OsStr) -> Vec<CompletionCandidate> {
    let records = match super::get_store_location() {
        Ok(store_path) => match Store::open(&store_path) {
            Ok(store) => store.list_records(None).unwrap_or(Vec::default()),
            Err(_) => Vec::default(),
        },
        Err(_) => Vec::default(),
    };

    do_complete(current, records)
}

fn do_complete(current: &OsStr, options: Vec<PathBuf>) -> Vec<CompletionCandidate> {
    let current = current.to_str().unwrap_or("");

    options
        .into_iter()
        .filter(|s| s.display().to_string().starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}
