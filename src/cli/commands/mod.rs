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
use clap::Subcommand;
use std::path::Path;

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
