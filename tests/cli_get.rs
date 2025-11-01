use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn help_get() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("koishi")?;

    let _ = cmd.arg("get").arg("--help");

    let _ = cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: koishi get"))
        .stdout(predicate::str::contains("--raw"));

    Ok(())
}

#[test]
fn get_raw_flag_in_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("koishi")?;

    let _ = cmd.arg("get").arg("--help");

    let _ = cmd.assert().success().stdout(predicate::str::contains(
        "Return the raw value without applying auto transforms",
    ));

    Ok(())
}

#[test]
fn get_with_nonexistent_store() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = TempDir::new()?;
    let store_path = tmp_dir.path().join("nonexistent_store");

    let mut cmd = Command::cargo_bin("koishi")?;

    let _ = cmd
        .env("KOISHI_STORE", store_path.to_str().unwrap())
        .arg("get")
        .arg("some/path");

    let _ = cmd.assert().failure();

    Ok(())
}
