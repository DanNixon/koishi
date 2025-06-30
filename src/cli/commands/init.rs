use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use std::path::Path;

/// Initialise the secret store.
#[derive(Debug, Parser)]
pub(super) struct Command {}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::init(store_path)?;

        let _ = store.edit_config_interactive()?;

        Ok(())
    }
}
