pub(in crate::cli::commands) mod auto_transforms;

use crate::{
    cli::Run,
    secret_store::{Record, Store},
};
use clap::Parser;
use inquire::{InquireError, Text};
use miette::IntoDiagnostic;
use skim::{SkimItem, SkimItemReceiver, SkimItemSender};
use std::{borrow::Cow, sync::Arc};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use zeroize::Zeroizing;

/// Query the store interactively.
#[derive(Debug, Parser)]
pub(super) struct Command {}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let records = store.list_records(None)?;

        if let Some(record) = pick_record(&records)? {
            let record = store.get_record(&record)?;
            let attributes = record.list_attributes()?;

            'interactive_selection: loop {
                let attribute = match pick_attribute(attributes.clone())? {
                    Some(v) => v,
                    None => break 'interactive_selection,
                };

                let lookup = match pick_lookup_mode()? {
                    Some(v) => v,
                    None => break 'interactive_selection,
                };

                do_query(&record, attribute, lookup)?;
            }
        }

        Ok(())
    }
}

fn do_query(record: &Record, attribute: String, lookup: LookupMode) -> miette::Result<()> {
    // Get the contents of the secret
    let secret = record.decrypt_and_extract(Some(&attribute))?;

    // Apply any automatic transformations
    let mut secret = auto_transforms::process(secret)?;

    // Output it according to the specified mode
    let wait_for_user_ready = match lookup {
        LookupMode::Copy => {
            eprintln!("Waiting for paste...");
            crate::utils::clipboard::copy(secret)?;
            false
        }
        LookupMode::QrCodeAscii => {
            let qr = crate::utils::qr::encode_ascii(secret)?;
            println!("{}", *qr);
            true
        }
        LookupMode::QrCodeUnicode => {
            let qr = crate::utils::qr::encode_unicode(secret)?;
            println!("{}", *qr);
            true
        }
        LookupMode::Get => {
            std::io::stdout()
                .write_all(secret.as_mut_slice())
                .into_diagnostic()?;
            println!();
            true
        }
        LookupMode::Characters => {
            let secret = crate::utils::bytes_to_string(secret)?;
            lookup_chars_from_secret(secret)?;
            false
        }
    };

    if wait_for_user_ready {
        // Ask the user to press enter to start looking at stuff again
        eprintln!("Press Enter to continue...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    Ok(())
}

fn pick_record(records: &[PathBuf]) -> miette::Result<Option<PathBuf>> {
    let (item_tx, item_rx): (SkimItemSender, SkimItemReceiver) = skim::prelude::unbounded();
    for record in records {
        let _ = item_tx.send(Arc::new(RecordLocationItem {
            inner: record.clone(),
        }));
    }
    drop(item_tx);

    Ok(crate::utils::skim::pick_single_item::<RecordLocationItem>(item_rx)?.map(|r| r.inner))
}

fn pick_attribute(attributes: Vec<String>) -> miette::Result<Option<String>> {
    let (item_tx, item_rx): (SkimItemSender, SkimItemReceiver) = skim::prelude::unbounded();
    for attribute in attributes {
        let _ = item_tx.send(Arc::new(attribute));
    }
    drop(item_tx);

    crate::utils::skim::pick_single_item::<String>(item_rx)
}

fn pick_lookup_mode() -> miette::Result<Option<LookupMode>> {
    let (item_tx, item_rx): (SkimItemSender, SkimItemReceiver) = skim::prelude::unbounded();
    for mode in LookupMode::iter() {
        let _ = item_tx.send(Arc::new(mode));
    }
    drop(item_tx);

    crate::utils::skim::pick_single_item::<LookupMode>(item_rx)
}

#[derive(Debug, Clone)]
struct RecordLocationItem {
    inner: PathBuf,
}

impl SkimItem for RecordLocationItem {
    fn text(&self) -> Cow<str> {
        Cow::Owned(format!("{}", self.inner.display()))
    }
}

#[derive(Debug, Clone, EnumIter)]
enum LookupMode {
    Copy,
    QrCodeUnicode,
    QrCodeAscii,
    Get,
    Characters,
}

impl SkimItem for LookupMode {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(match self {
            Self::Copy => "Copy to clipboard",
            Self::QrCodeUnicode => "Generate Unicode QR code",
            Self::QrCodeAscii => "Generate ASCII QR code",
            Self::Get => "Get and output to termainl",
            Self::Characters => "Choose specific characters from a string and output to terminal",
        })
    }
}

fn lookup_chars_from_secret(secret: Zeroizing<String>) -> miette::Result<()> {
    loop {
        let prompt = Text::new("Position of character to get:").prompt();

        let idx = match prompt {
            Ok(s) => s
                .trim()
                .parse::<usize>()
                .into_diagnostic()?
                .checked_sub(1)
                .ok_or(miette::miette!(
                    "Position must be a positive integer greater than 0"
                ))?,
            Err(InquireError::OperationCanceled) => return Ok(()),
            Err(e) => {
                return Err(e).into_diagnostic();
            }
        };

        match secret.get(idx..idx + 1) {
            Some(ss) => {
                println!("Character at index {idx}: {ss}");
            }
            None => {
                return Err(miette::miette!(
                    "Index {} is out of bounds for the secret string of length {}",
                    idx,
                    secret.len()
                ));
            }
        }
    }
}
