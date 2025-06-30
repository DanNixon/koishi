use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use std::path::Path;

/// Edit the SOPS configuration.
#[derive(Debug, Parser)]
pub(super) struct Command {}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        if !store.edit_config_interactive()? {
            eprintln!("No changes.");
        }

        Ok(())
    }
}
