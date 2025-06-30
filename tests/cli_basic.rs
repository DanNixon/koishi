use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn help_root() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("koishi")?;

    let _ = cmd.arg("--help");

    let _ = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: koishi"));

    Ok(())
}
