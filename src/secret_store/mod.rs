mod record;
pub(crate) use record::Record;

use crate::utils::git::GitOperationResult;
use miette::{Context, IntoDiagnostic, miette};
use std::path::{Path, PathBuf};

pub(super) const SOPS_CONFIG_FILENAME: &str = ".sops.yaml";

const DEFAULT_SOPS_CONFIG: &str = r#"# Reference: https://github.com/getsops/sops?tab=readme-ov-file#using-sops-yaml-conf-to-select-kms-pgp-and-age-for-new-files
keys:
  - &key1: abc
  - &key2: def

creation_rules:
  - key_groups:
      - age:
          - *key1
          - *key2
"#;

#[derive(Debug)]
pub(crate) struct Store {
    root: PathBuf,
}

impl Store {
    /// Initialises a new secret store at a specified location.
    pub(crate) fn init(root: &Path) -> miette::Result<Self> {
        // Ensure the directories up to and including the requested root of the store exist
        std::fs::create_dir_all(root)
            .into_diagnostic()
            .wrap_err(format!(
                "Failed to create directories: `{}`",
                root.display()
            ))?;

        // Init a Git repository in the root directory
        let _ = gix::init(root).into_diagnostic().wrap_err(format!(
            "Failed to init Git repository in `{}`",
            root.display()
        ))?;

        // Populate the default SOPS config file
        let _ = crate::utils::git::git_operation(root, "Write default SOPS config", || {
            std::fs::write(root.join(SOPS_CONFIG_FILENAME), DEFAULT_SOPS_CONFIG)
                .into_diagnostic()
                .wrap_err("Failed to write SOPS config file")
        })?;

        Ok(Self { root: root.into() })
    }

    /// Opens a secret store, verifying that it is valid.
    pub(crate) fn open(root: &Path) -> miette::Result<Self> {
        check_store_is_valid(root).wrap_err(format!("Store at `{}` is invalid", root.display()))?;

        Ok(Self { root: root.into() })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Opens the SOPS config file in EDITOR for interactive editing, committing the changes after
    /// the editor is closed.
    pub(crate) fn edit_config_interactive(&self) -> miette::Result<bool> {
        Ok(
            crate::utils::git::git_operation(&self.root, "Edit SOPS config", || {
                crate::utils::edit_file_interactive(self.root.join(SOPS_CONFIG_FILENAME))
                    .wrap_err("Failed to edit SOPS config file")
            })
            .wrap_err("Failed to edit SOPS config file")?
                == GitOperationResult::Commit,
        )
    }
}

fn check_store_is_valid(root: &Path) -> miette::Result<()> {
    // Check that the root of the store exists at all
    if !root.exists() {
        return Err(miette!("Store root does not exist at `{}`", root.display()));
    }

    // Check that the root of the store is a directory
    if !root.is_dir() {
        return Err(miette!(
            "Store root `{}` is not a directory",
            root.display()
        ));
    }

    // Check for a SOPS config file, which should always be present in the root of a store
    let root_config_filename = root.join(SOPS_CONFIG_FILENAME);
    let root_config = Path::new(&root_config_filename);
    if !root_config.exists() || !root_config.is_file() {
        return Err(miette!(
            "Store root does not contain a SOPS config file at `{}`",
            root_config.display()
        ));
    }

    // Check that the store directory is a Git repository
    let _ = gix::discover(root).map_err(|_| {
        miette!(
            "Store directory `{}` is not a Git repository",
            root.display()
        )
    })?;

    Ok(())
}

#[derive(Debug)]
pub(crate) struct StoreLocation<'a> {
    root: &'a Path,
    store_filename: PathBuf,
}

impl<'a> StoreLocation<'a> {
    pub(crate) fn from_path(root: &'a Path, store_path: &Path) -> Self {
        StoreLocation {
            root,
            store_filename: store_path.into(),
        }
    }

    pub(crate) fn filename(&self) -> PathBuf {
        self.root.join(&self.store_filename)
    }

    pub(crate) fn store_filename(&self) -> &Path {
        &self.store_filename
    }

    fn create_directories(&self) -> miette::Result<()> {
        let dir = self.root.join(&self.store_filename);
        let parent = dir.parent().unwrap();

        std::fs::create_dir_all(parent)
            .into_diagnostic()
            .wrap_err(format!(
                "Failed to create directories for new config in `{}`",
                parent.display()
            ))
    }

    /// Checks if the store location exists on disk and is a file.
    pub(crate) fn exists(&self) -> bool {
        self.filename().is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_store_init_and_open() {
        crate::utils::test::set_git_config();

        let dir = tempdir().unwrap();
        let root = dir.path();

        // Should initialize successfully
        let store = Store::init(root).unwrap();
        assert_eq!(store.root(), root);

        // SOPS config file should exist
        let config_path = root.join(SOPS_CONFIG_FILENAME);
        assert!(config_path.exists());

        // Should open successfully
        let opened = Store::open(root).unwrap();
        assert_eq!(opened.root(), root);
    }

    #[test]
    fn test_store_open_invalid() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Directory exists but not initialized as a store
        let err = Store::open(root).unwrap_err();
        assert!(err.to_string().contains("invalid"));
    }

    #[test]
    fn test_store_location_filename_and_exists() {
        crate::utils::test::set_git_config();

        let dir = tempdir().unwrap();
        let root = dir.path();
        let _ = Store::init(root).unwrap();

        let file_path = Path::new("foo/bar.txt");
        let loc = StoreLocation::from_path(root, file_path);

        // File should not exist yet
        assert!(!loc.exists());

        // Create directories and file
        loc.create_directories().unwrap();
        let full_path = loc.filename();
        fs::write(&full_path, "demo").unwrap();

        assert!(loc.exists());
        assert_eq!(loc.store_filename(), file_path);
    }
}
