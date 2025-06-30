use crate::utils::git::GitOperationResult;

use super::{Store, StoreLocation};
use miette::{Context, IntoDiagnostic, miette};
use saphyr::LoadableYamlNode;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};
use zeroize::Zeroizing;

impl Store {
    /// Create a new record in the store.
    ///
    /// Performs validation that it can exist and creates the required directories, but otherwise
    /// does not write anything to disk.
    /// Writing must be performed using the returned `Record`.
    pub(crate) fn create_record(&self, path: &Path) -> miette::Result<Record> {
        let location = StoreLocation::from_path(&self.root, path);

        if location.exists() {
            return Err(miette!(
                "Secret already exists at `{}`",
                location.filename().display()
            ));
        }

        location.create_directories()?;

        Ok(Record { location })
    }

    /// Lists all secrets under a given store path (or in the entire store if no path is provided).
    ///
    /// Assumes that any file that is not a SOPS config (`.sops.yml`) or a hidden/dotfile
    /// (`.something`) is a secret file.
    /// No actual validation happens here.
    pub(crate) fn list_records(&self, store_path: Option<&Path>) -> miette::Result<Vec<PathBuf>> {
        let path = match store_path {
            Some(store_path) => self.root.join(store_path),
            None => self.root.to_owned(),
        };

        fn is_hidden(entry: &DirEntry) -> bool {
            entry
                .file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        }

        fn sort_by_name_files_before_dirs(a: &DirEntry, b: &DirEntry) -> Ordering {
            if a.path().is_dir() && b.path().is_file() {
                Ordering::Greater
            } else if a.path().is_file() && b.path().is_dir() {
                Ordering::Less
            } else {
                a.file_name().cmp(b.file_name())
            }
        }

        Ok(WalkDir::new(path)
            .sort_by(sort_by_name_files_before_dirs)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .flat_map(|e| e.ok())
            .flat_map(|e| {
                let e = e.path();

                if e.is_file() {
                    Some(e.strip_prefix(&self.root).unwrap().to_owned())
                } else {
                    None
                }
            })
            .collect())
    }

    /// Get a record given a path in the store.
    pub(crate) fn get_record(&self, path: &Path) -> miette::Result<Record> {
        let exact_location = StoreLocation::from_path(&self.root, path);

        if exact_location.exists() {
            Ok(Record {
                location: exact_location,
            })
        } else {
            Err(miette!(
                "No secret found at `{}`",
                exact_location.store_filename().display()
            ))
        }
    }

    /// Get a record given a path in the store.
    ///
    /// Will not fail if the path in the store does not exist.
    /// Instead a `Record` will be returned that simply does not exist.
    pub(crate) fn get_record_unchecked(&self, path: &Path) -> miette::Result<Record> {
        let exact_location = StoreLocation::from_path(&self.root, path);

        Ok(Record {
            location: exact_location,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Record<'a> {
    location: StoreLocation<'a>,
}

impl<'a> Record<'a> {
    pub(crate) fn filename(&self) -> PathBuf {
        self.location.filename()
    }

    pub(crate) fn edit_interactive(&self) -> miette::Result<bool> {
        // Ensure the directory for this record exists before trying to edit interactively with SOPS
        self.location.create_directories()?;

        Ok(crate::utils::git::git_operation(
            self.location.root,
            &format!("Edit record `{}`", self.location.store_filename().display()),
            || crate::utils::sops::edit(self.location.root, &self.location.filename()),
        )? == GitOperationResult::Commit)
    }

    /// Moves/renames this record.
    pub(crate) fn move_to(&mut self, destination: StoreLocation<'a>) -> miette::Result<()> {
        // Ensure that the destination is in the same store
        if self.location.root != destination.root {
            return Err(miette!(
                "Cannot move record to a different store: {}",
                destination.root.display()
            ));
        }

        // Ensure that the destination does not already exist
        if destination.exists() {
            return Err(miette!(
                "Destination `{}` already exists",
                destination.store_filename().display()
            ));
        }

        // Ensure directories for destination exist
        destination.create_directories()?;

        // Move the record file
        let _ = crate::utils::git::git_operation(
            self.location.root,
            &format!(
                "Move record `{}` => `{}`",
                self.location.store_filename().display(),
                destination.store_filename().display()
            ),
            || {
                std::fs::rename(self.location.filename(), destination.filename())
                    .into_diagnostic()
                    .wrap_err_with(|| {
                        format!(
                            "Failed to move record `{}` => `{}`",
                            self.location.store_filename().display(),
                            destination.store_filename().display()
                        )
                    })
            },
        )?;

        self.location = destination;

        Ok(())
    }

    /// Deletes this record from the store, committing changes.
    pub(crate) fn delete(&self) -> miette::Result<()> {
        let _ = crate::utils::git::git_operation(
            self.location.root,
            &format!(
                "Delete record `{}`",
                self.location.store_filename().display()
            ),
            || {
                std::fs::remove_file(self.location.filename())
                    .into_diagnostic()
                    .wrap_err_with(|| {
                        format!(
                            "Failed to delete record `{}`",
                            self.location.store_filename().display()
                        )
                    })
            },
        )?;

        Ok(())
    }

    pub(crate) fn encrypt_entire_file(&self, contents: Zeroizing<Vec<u8>>) -> miette::Result<()> {
        let _ = crate::utils::git::git_operation(
            self.location.root,
            &format!(
                "Update contents of record `{}`",
                self.location.store_filename().display()
            ),
            || {
                crate::utils::sops::encrypt(
                    self.location.root,
                    &self.location.filename(),
                    contents.clone(),
                )
            },
        )?;

        Ok(())
    }

    pub(crate) fn encrypt_set(
        &self,
        selector: &str,
        contents: Zeroizing<Vec<u8>>,
    ) -> miette::Result<()> {
        let _ = crate::utils::git::git_operation(
            self.location.root,
            &format!(
                "Update `{selector}` in record `{}`",
                self.location.store_filename().display()
            ),
            || {
                let selector = format_selector(Some(selector));
                let contents = crate::utils::bytes_to_string(contents.clone())?;

                crate::utils::sops::set(
                    self.location.root,
                    &self.location.filename(),
                    selector.unwrap().as_str(),
                    contents,
                )
            },
        )?;

        Ok(())
    }

    pub(crate) fn decrypt_and_extract(
        &self,
        selector: Option<&str>,
    ) -> miette::Result<Zeroizing<Vec<u8>>> {
        let selector = format_selector(selector);

        crate::utils::sops::decrypt(
            self.location.root,
            self.location.store_filename(),
            selector.as_deref(),
        )
    }

    /// Return a list of top level keys from this record.
    ///
    /// Only returns keys that have string values.
    /// Will only give results for files that are valid YAML or JSON encoded SOPS files.
    pub(crate) fn list_top_level_attributes(&self) -> miette::Result<Vec<String>> {
        let content = std::fs::read_to_string(self.location.filename()).into_diagnostic()?;

        // Try to parse the encrypted SOPS file as JSON
        if let Ok(serde_json::Value::Object(map)) =
            serde_json::from_str::<serde_json::Value>(&content)
        {
            return Ok(map
                .iter()
                .flat_map(|(k, v)| {
                    if v.is_string() {
                        Some(k.to_owned())
                    } else {
                        None
                    }
                })
                .collect());
        }

        // Try to parse the encrypted SOPS file as YAML
        if let Ok(yaml) = saphyr::Yaml::load_from_str(&content) {
            if let Some(yaml) = yaml.first() {
                if let Some(mapping) = yaml.as_mapping() {
                    return Ok(mapping
                        .iter()
                        .flat_map(|(k, v)| {
                            if v.is_string() {
                                Some(k.as_str().unwrap().to_owned())
                            } else {
                                None
                            }
                        })
                        .collect());
                }
            }
        }

        Err(miette::miette!("File is not valid JSON or YAML"))
    }
}

fn format_selector(selector: Option<&str>) -> Option<String> {
    selector.map(|s| {
        if s.contains('[') || s.contains(']') {
            // If the string contains brackets, assume it's a JSON path or similar
            s.to_string()
        } else {
            // Otherwise, treat it as a simple key
            format!("[\"{s}\"]")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_get_record() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("store");

        crate::utils::test::set_git_config();
        crate::utils::test::init_git_repo(dir.path());
        let store = Store {
            root: store_path.clone(),
        };

        let record_path = Path::new("foo/bar.txt");
        let record = store.create_record(record_path).unwrap();
        assert_eq!(record.filename(), store_path.join(record_path));

        // Now the record should not exist yet
        assert!(std::fs::metadata(record.filename()).is_err());

        // Write the file to simulate existence
        std::fs::write(record.filename(), "dummy").unwrap();

        let fetched = store.get_record(record_path).unwrap();
        assert_eq!(fetched.filename(), record.filename());
    }

    #[test]
    fn test_list_records() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("store");

        crate::utils::test::set_git_config();
        crate::utils::test::init_git_repo(&store_path);
        let store = Store { root: store_path };

        let paths = [
            Path::new("foo/bar.txt"),
            Path::new("foo/baz.txt"),
            Path::new("qux.txt"),
        ];
        for p in &paths {
            let rec = store.create_record(p).unwrap();
            println!("{}", rec.filename().display());
            std::fs::write(rec.filename(), "dummy").unwrap();
        }
        // Add a hidden file and a .sops.yml
        std::fs::write(dir.path().join(".hidden"), "x").unwrap();
        std::fs::write(dir.path().join(".sops.yml"), "x").unwrap();

        let listed = store.list_records(None).unwrap();
        assert_eq!(listed.len(), 3);
        for p in &paths {
            assert!(listed.contains(&Path::new(p).to_path_buf()));
        }
    }

    #[test]
    fn test_get_record_unchecked() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("store");

        crate::utils::test::set_git_config();
        crate::utils::test::init_git_repo(dir.path());
        let store = Store {
            root: store_path.clone(),
        };

        let record_path = Path::new("does/not/exist.txt");
        let record = store.get_record_unchecked(record_path).unwrap();
        assert_eq!(record.filename(), store_path.join(record_path));
    }

    #[test]
    fn test_format_selector() {
        assert_eq!(format_selector(Some("foo")), Some("[\"foo\"]".to_string()));
        assert_eq!(format_selector(Some("foo[0]")), Some("foo[0]".to_string()));
        assert_eq!(format_selector(None), None);
    }
}
