use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use miette::IntoDiagnostic;
use std::{
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

/// Set part or all of a record.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Path to a record
    #[arg(add = ArgValueCompleter::new(super::complete_secret))]
    path: PathBuf,

    /// Part of the record to set
    selector: Option<String>,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let record = if self.selector.is_some() {
            // Need an existing secret when using a selector
            store.get_record(&self.path)?
        } else {
            // If setting the entire contents, try to get a record, otherwise create a new one
            match store.get_record(&self.path) {
                Ok(r) => r,
                Err(_) => store.create_record(&self.path)?,
            }
        };

        let contents = read_secret()?;

        match &self.selector {
            Some(selector) => record.encrypt_set(selector, contents),
            None => record.encrypt_entire_file(contents),
        }
    }
}

fn read_secret() -> miette::Result<Zeroizing<Vec<u8>>> {
    if std::io::stdin().is_terminal() {
        // Prompt for the secret
        Ok(inquire::Password::new("Secret:")
            .with_display_toggle_enabled()
            .prompt()
            .into_diagnostic()?
            .as_bytes()
            .to_vec()
            .into())
    } else {
        // Read the secret from stdin
        let mut buff = Zeroizing::new(Vec::new());
        let _ = std::io::stdin().read_to_end(&mut buff).into_diagnostic()?;
        Ok(buff)
    }
}

// koishi record set path/to/a/secret.ext somekey
//  (fails if secret not found)
//  (prompts for string, asks for confirmation)
//  (updates a key in a structured file)
//
// koishi record set path/to/a/secret.ext '["somekey"][0]'
//  (fails if secret not found)
//  (prompts for string, asks for confirmation)
//  (updates a complex path in a structured file)
//
// echo "secret" | koishi record set path/to/a/secret.ext somekey
//  (fails if secret not found)
//  (updates a key in a structured file)
//
// echo "secret" | koishi record set path/to/a/secret.ext '["somekey"][0]'
//  (fails if secret not found)
//  (updates a complex path in a structured file)
