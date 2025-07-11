use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use std::path::{Path, PathBuf};

/// Opens a record for editing in the default editor.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Path to a record
    #[arg(add = ArgValueCompleter::new(super::complete_record))]
    path: PathBuf,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        // Get the (maybe empty) record
        let record = store.get_record_unchecked(&self.path)?;

        if !record.edit_interactive()? {
            eprintln!("No changes.");
        }

        Ok(())
    }
}
