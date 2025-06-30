use miette::{Context, IntoDiagnostic, miette};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum GitOperationResult {
    Commit,
    NoChanges,
}

/// Performs an operation that will modify files, committing those files to a Git repository only
/// when the operation succeeds.
pub(crate) fn git_operation<F: Fn() -> miette::Result<()>>(
    repo_dir: &Path,
    commit_msg: &str,
    op: F,
) -> miette::Result<GitOperationResult> {
    // TODO: replace this with use of gix

    if !is_clean(repo_dir)? {
        return Err(miette!(
            "Git repository at `{}` is not clean. Please commit or stash your changes.",
            repo_dir.display()
        ));
    }

    match op() {
        Ok(_) => {
            if is_clean(repo_dir)? {
                Ok(GitOperationResult::NoChanges)
            } else {
                // Stage all changes (this should only be those changed as a result of `op()`, however
                // this is not a perfectly clean test)
                run_git_command(repo_dir, &["add", "."]).wrap_err("Failed to stage file")?;

                // Commit with the supplied message
                run_git_command(repo_dir, &["commit", "-m", commit_msg])
                    .wrap_err("Failed to commit changes")?;

                Ok(GitOperationResult::Commit)
            }
        }
        Err(e) => Err(e),
    }
}

fn run_git_command(repo_dir: &Path, args: &[&str]) -> miette::Result<()> {
    let result = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(args)
        .output()
        .into_diagnostic()
        .wrap_err("Failed to run git executable")?;

    if !result.status.success() {
        Err(miette::miette!(
            "Git command failed (status {}): {}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        ))
    } else {
        Ok(())
    }
}

/// Checks if a Git repository is clean, i.e. it has no staged or unstaged changes and no untracked
/// files.
fn is_clean(repo_dir: &Path) -> miette::Result<bool> {
    // TODO: replace this with use of gix

    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .arg("status")
        .arg("--porcelain")
        .output()
        .into_diagnostic()?;

    Ok(output.stdout.is_empty())
}
