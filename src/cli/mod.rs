mod commands;

use clap::Parser;
use commands::Command;
use miette::{Context, IntoDiagnostic};
use std::path::{Path, PathBuf};

trait Run {
    fn run(&self, store_path: &Path) -> miette::Result<()>;
}

/// Koishi: the keeper of important secrets, hierarchically indexed.
#[derive(Debug, Parser)]
#[command(author, version = self::version(), about, long_about = None)]
struct Cli {
    #[arg(
        long = "store",
        short = 's',
        env = "KOISHI_STORE",
        default_value = "$XDG_DATA_HOME/koishi-store"
    )]
    store_path: PathBuf,

    #[command(subcommand)]
    command: Command,
}

pub(super) fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let store_path = shellexpand::path::full(&cli.store_path)
        .into_diagnostic()
        .wrap_err("Failed to perform shell expansion on store path")?;

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
