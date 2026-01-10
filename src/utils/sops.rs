use miette::{Context, IntoDiagnostic};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use zeroize::Zeroizing;

pub(crate) fn interactive_command<F: Fn(&mut Command)>(
    workdir: &Path,
    configure: F,
) -> miette::Result<()> {
    let mut command = Command::new("sops");
    configure(&mut command);

    let status = command
        .current_dir(workdir)
        .status()
        .into_diagnostic()
        .wrap_err("Failed to run sops executable")?;

    if !status.success() {
        Err(miette::miette!(
            "SOPS command failed with status: {}",
            status
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn edit(workdir: &Path, file: &Path) -> miette::Result<()> {
    interactive_command(workdir, |cmd| {
        let _ = cmd.arg("edit").arg(file);
    })
}

pub(crate) fn decrypt(
    workdir: &Path,
    file: &Path,
    extract: Option<&str>,
) -> miette::Result<Zeroizing<Vec<u8>>> {
    let mut command = Command::new("sops");

    let _ = command.current_dir(workdir).arg("decrypt");

    if let Some(extract) = extract {
        let _ = command.arg("--extract").arg(extract);
    }

    let result = command
        .arg(file)
        .stdout(Stdio::piped())
        .output()
        .into_diagnostic()
        .wrap_err("Failed to run sops executable")?;

    if result.status.success() {
        Ok(Zeroizing::new(result.stdout))
    } else {
        Err(miette::miette!(
            "SOPS command failed with status: {}",
            result.status
        ))
    }
}

pub(crate) fn encrypt(
    workdir: &Path,
    file: &Path,
    mut contents: Zeroizing<Vec<u8>>,
) -> miette::Result<()> {
    let mut command = Command::new("sops");

    let mut proc = command
        .current_dir(workdir)
        .arg("encrypt")
        .arg("--filename-override")
        .arg(file)
        .arg("--output")
        .arg(file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .into_diagnostic()
        .wrap_err("Failed to run sops executable")?;

    proc.stdin
        .as_mut()
        .unwrap()
        .write_all(contents.as_mut_slice())
        .into_diagnostic()
        .wrap_err("Failed to write to sops stdin")?;

    let result = proc
        .wait()
        .into_diagnostic()
        .wrap_err("Failed to run sops executable")?;

    if result.success() {
        Ok(())
    } else {
        Err(miette::miette!(
            "SOPS command failed with status: {}",
            result
        ))
    }
}

pub(crate) fn set(
    workdir: &Path,
    file: &Path,
    selector: &str,
    contents: Zeroizing<String>,
) -> miette::Result<()> {
    let contents = Zeroizing::new(format!("\"{}\"", *contents));

    let mut command = Command::new("sops");

    let result = command
        .current_dir(workdir)
        .arg("set")
        .arg(file)
        .arg(selector)
        .arg(contents)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .status()
        .into_diagnostic()
        .wrap_err("Failed to run sops executable")?;

    if result.success() {
        Ok(())
    } else {
        Err(miette::miette!(
            "SOPS command failed with status: {}",
            result
        ))
    }
}

pub(crate) fn update_keys(workdir: &Path, file: &Path, yes: bool) -> miette::Result<()> {
    let mut command = Command::new("sops");

    let _ = command.current_dir(workdir).arg("updatekeys");

    if yes {
        let _ = command.arg("--yes");
    }

    let result = command
        .arg(file)
        .status()
        .into_diagnostic()
        .wrap_err("Failed to run sops executable")?;

    if result.success() {
        Ok(())
    } else {
        Err(miette::miette!(
            "SOPS command failed with status: {}",
            result
        ))
    }
}
