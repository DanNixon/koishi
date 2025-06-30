use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use miette::IntoDiagnostic;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

/// Show the SOPS encrypted contents of a record.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Path to a record
    path: PathBuf,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let record = store.get_record(&self.path)?;

        // Read the encrypted contents of the record
        let encrypted_contents = std::fs::read_to_string(record.filename()).into_diagnostic()?;

        if std::io::stdout().is_terminal() {
            // Output the records encrypted contents to the terminal via PAGER
            let pager = std::env::var("PAGER").unwrap_or_else(|_| "less".into());

            let mut pager_process = std::process::Command::new(pager)
                .stdin(std::process::Stdio::piped())
                .spawn()
                .into_diagnostic()?;

            if let Some(mut stdin) = pager_process.stdin.take() {
                stdin
                    .write_all(encrypted_contents.as_bytes())
                    .into_diagnostic()?;
            }

            let _ = pager_process.wait().into_diagnostic()?;
        } else {
            // Or just print the contents to stdout if it is not a TTY
            print!("{encrypted_contents}");
        }

        Ok(())
    }
}
