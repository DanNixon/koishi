pub(crate) mod clipboard;
pub(crate) mod git;
pub(crate) mod qr;
pub(crate) mod skim;
pub(crate) mod sops;
#[cfg(test)]
pub(crate) mod test;

use miette::{Context, IntoDiagnostic};
use std::{path::Path, process::Command};
use totp_rs::TOTP;
use zeroize::Zeroizing;

/// Opens a file in the default editor for interactive editing.
///
/// Will fallback to `vi` if the `EDITOR` environment variable is not set.
pub(crate) fn edit_file_interactive<P: AsRef<Path>>(file_path: P) -> miette::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let _ = Command::new(&editor)
        .arg(file_path.as_ref())
        .status()
        .into_diagnostic()
        .wrap_err(format!("Failed to edit file with {editor}"))?;

    Ok(())
}

pub(crate) fn bytes_to_string(bytes: Zeroizing<Vec<u8>>) -> miette::Result<Zeroizing<String>> {
    Ok(Zeroizing::new(
        String::from_utf8(bytes.to_vec())
            .into_diagnostic()
            .wrap_err("Failed to parse bytes as UTF8 string")?
            .trim()
            .to_string(),
    ))
}

pub(crate) fn totp_from_otpauth(otpauth: Zeroizing<String>) -> miette::Result<Zeroizing<String>> {
    let totp = TOTP::from_url_unchecked(otpauth).into_diagnostic()?;
    Ok(Zeroizing::new(totp.generate_current().into_diagnostic()?))
}
