use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use miette::{IntoDiagnostic, WrapErr, miette};
use std::{path::Path, process::Stdio};

/// Run Git commands inside the secret store.
#[derive(Debug, Parser)]
pub(super) struct Command {
    args: Vec<String>,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        // Try to open the store, this is just to provide validation
        let _ = Store::open(store_path)?;

        // Run the Git command with the provided arguments
        let res = std::process::Command::new("git")
            .arg("-C")
            .arg(store_path)
            .args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .into_diagnostic()
            .wrap_err("Failed to run git command")?;

        if res.success() {
            Ok(())
        } else {
            Err(miette!("Git exited with non-success error code: {}", res))
        }
    }
}
