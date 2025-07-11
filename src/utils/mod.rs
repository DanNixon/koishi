pub(crate) mod clipboard;
pub(crate) mod file;
pub(crate) mod git;
pub(crate) mod qr;
pub(crate) mod skim;
pub(crate) mod sops;
#[cfg(test)]
pub(crate) mod test;

use miette::{Context, IntoDiagnostic};
use totp_rs::TOTP;
use zeroize::Zeroizing;

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
