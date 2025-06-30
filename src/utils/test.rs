use std::{path::Path, process::Command};

pub(crate) fn set_git_config() {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("GIT_AUTHOR_EMAIL", "test@example.com");
        std::env::set_var("GIT_COMMITTER_EMAIL", "test@example.com");
        std::env::set_var("GIT_AUTHOR_NAME", "Test");
        std::env::set_var("GIT_COMMITTER_NAME", "Test");
    }
}

/// Initialize a git repo for testing.
pub(crate) fn init_git_repo(path: &Path) {
    let _ = Command::new("git")
        .arg("init")
        .arg(path)
        .output()
        .expect("failed to init git");
}
