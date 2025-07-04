use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use std::path::{Path, PathBuf};

/// Delete a record from the store.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Path to a record
    #[arg(add = ArgValueCompleter::new(super::complete_secret))]
    path: PathBuf,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let record = store.get_record(&self.path)?;

        record.delete()?;

        Ok(())
    }
}
