use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use std::path::{Path, PathBuf};

/// Re-encrypt secrets under a given path.
///
/// Essentially a batch equivalent of `sops updatekeys`.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Pre-approve all changes and run non-interactively
    #[arg(short, long)]
    yes: bool,

    /// Path under which to rekey records
    path: Option<PathBuf>,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        // Get a list of records under the specified path
        let records = store.list_records(self.path.as_deref())?;

        let _ = crate::utils::git::git_operation(
            store.root(),
            &match &self.path {
                Some(path) => format!("Update keys for records in `{}`", path.display()),
                None => "Update keys for all records".into(),
            },
            || {
                for record in &records {
                    crate::utils::sops::update_keys(store.root(), record, self.yes)?;
                }
                Ok(())
            },
        )?;

        Ok(())
    }
}
