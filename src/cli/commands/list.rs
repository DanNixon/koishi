use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use std::path::{Path, PathBuf};

/// List records in the store.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Path to list
    path: Option<PathBuf>,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        for r in store.list_records(self.path.as_deref())? {
            println!("{}", r.display());
        }

        Ok(())
    }
}
