mod commands;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use commands::Command;
use miette::{Context, IntoDiagnostic};
use std::{
    env::VarError,
    path::{Path, PathBuf},
};

fn get_store_location() -> miette::Result<PathBuf> {
    const DEFAULT_STORE_LOCATION: &str = "$XDG_DATA_HOME/koishi-store";

    let location = match std::env::var("KOISHI_STORE") {
        Ok(val) => val,
        Err(VarError::NotPresent) => DEFAULT_STORE_LOCATION.into(),
        Err(VarError::NotUnicode(s)) => {
            return Err(miette::miette!(
                "Failed to read KOISHI_STORE environment variable: {s:?}"
            ));
        }
    };

    Ok(shellexpand::path::full(&location)
        .into_diagnostic()
        .wrap_err("Failed to perform shell expansion on store path")?
        .into_owned())
}

trait Run {
    fn run(&self, store_path: &Path) -> miette::Result<()>;
}

/// Koishi: the keeper of important secrets, hierarchically indexed.
#[derive(Debug, Parser)]
#[command(name = "koishi", author, version = self::version(), about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub(super) fn main() -> miette::Result<()> {
    CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();

    let store_path = get_store_location().wrap_err("Failed to determine store location")?;
    cli.command.run(&store_path)
}

fn version() -> String {
    fn get_binary_version(binary: &str, args: &[&str]) -> String {
        std::process::Command::new(binary)
            .args(args)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or("unknown".into())
    }

    let git_binary_version = get_binary_version("git", &["--version"]);
    let sops_binary_version = get_binary_version("sops", &["--version", "--disable-version-check"]);

    format!(
        "v{}\n {git_binary_version}\n {sops_binary_version}",
        clap::crate_version!()
    )
}
