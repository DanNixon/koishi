use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use std::path::Path;

/// Run SOPS commands from the root of the store.
#[derive(Debug, Parser)]
pub(super) struct Command {
    args: Vec<String>,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let _ = crate::utils::git::git_operation(
            store.root(),
            &format!("Perform SOPS command: `sops {}`", self.args.join(" ")),
            || {
                crate::utils::sops::interactive_command(store.root(), |cmd| {
                    let _ = cmd.args(&self.args);
                })
            },
        )?;

        Ok(())
    }
}
