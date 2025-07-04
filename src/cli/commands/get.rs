use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use clap_complete::ArgValueCompleter;
use miette::IntoDiagnostic;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

/// Gets part or all of a record.
///
/// If no selector is given then the entire record is returned.
/// If a simple string is given then the corresponding attribute under the top level is returned.
/// If a path of string keys delimited by forward slashes (/) then a path selector is built from this.
/// Otherwise the selector is assumed to be a path specifier, as per `sops decrypt --extract`
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Copy the secret to the clipboard
    #[clap(short, long, conflicts_with_all = &["qr", "qr_ascii", "qr_unicode"])]
    copy: bool,

    /// Output the secret as a QR code in a PNG image
    #[clap(long, conflicts_with_all = &["copy", "qr_ascii", "qr_unicode"])]
    qr: bool,

    /// Output the secret as a QR code as text using ASCII characters
    #[clap(long, conflicts_with_all = &["copy", "qr", "qr_unicode"])]
    qr_ascii: bool,

    /// Output the secret as a QR code as nicer text using unicode characters
    #[clap(long, conflicts_with_all = &["copy", "qr", "qr_ascii"])]
    qr_unicode: bool,

    /// Path to a record
    #[arg(add = ArgValueCompleter::new(super::complete_secret))]
    path: PathBuf,

    /// Part of the record to get
    selector: Option<String>,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let record = store.get_record(&self.path)?;

        let mut secret = record.decrypt_and_extract(self.selector.as_deref())?;

        if self.copy {
            crate::utils::clipboard::copy(secret)?;
        } else if self.qr {
            let png = crate::utils::qr::encode_png(secret)?;
            std::io::stdout().write_all(&png).into_diagnostic()?;
        } else if self.qr_ascii {
            let qr = crate::utils::qr::encode_ascii(secret)?;
            println!("{}", *qr);
        } else if self.qr_unicode {
            let qr = crate::utils::qr::encode_unicode(secret)?;
            println!("{}", *qr);
        } else {
            std::io::stdout()
                .write_all(secret.as_mut_slice())
                .into_diagnostic()?;
        }

        Ok(())
    }
}
