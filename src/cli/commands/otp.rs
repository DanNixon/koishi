use crate::{cli::Run, secret_store::Store};
use clap::Parser;
use std::path::{Path, PathBuf};

/// Generate TOTP codes from records.
#[derive(Debug, Parser)]
pub(super) struct Command {
    /// Part of the record that contains the OTP URL
    #[arg(long, default_value = "otp")]
    otp_selector: String,

    /// Path to a record
    path: PathBuf,
}

impl Run for Command {
    fn run(&self, store_path: &Path) -> miette::Result<()> {
        let store = Store::open(store_path)?;

        let record = store.get_record(&self.path)?;

        let otp_key = record.decrypt_and_extract(Some(self.otp_selector.as_str()))?;
        let otp_key = crate::utils::bytes_to_string(otp_key)?;

        let otp_pass = crate::utils::totp_from_otpauth(otp_key)?;

        println!("{}", *otp_pass);

        Ok(())
    }
}
