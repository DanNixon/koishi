use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use std::path::{Path, PathBuf};

/// Moves/renames a record.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Path to the source record
    #[arg(add = ArgValueCompleter::new(super::complete_secret))]
    source: PathBuf,

    /// Path to move the record to
    #[arg(add = ArgValueCompleter::new(super::complete_secret))]
    destination: PathBuf,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let mut record = store.get_record(&self.source)?;

        let destination = store.location(&self.destination);

        record.move_to(destination)?;

        Ok(())
    }
}
