use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use std::path::{Path, PathBuf};

/// Moves/renames a directory or record.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Path to the source directory/record
    #[arg(add = ArgValueCompleter::new(super::complete_location))]
    source: PathBuf,

    /// Path to move the directory/record to
    #[arg(add = ArgValueCompleter::new(super::complete_location))]
    destination: PathBuf,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let source = store.location(&self.source);
        let destination = store.location(&self.destination);

        source.move_to(destination)?;

        Ok(())
    }
}
