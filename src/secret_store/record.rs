use crate::utils::git::GitOperationResult;

use super::{Store, StoreLocation};
use miette::{IntoDiagnostic, miette};
use saphyr::LoadableYamlNode;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zeroize::Zeroizing;

impl Store {
    /// Create a new record in the store.
    ///
    /// Performs validation that it can exist and creates the required directories, but otherwise
    /// does not write anything to disk.
    /// Writing must be performed using the returned `Record`.
    pub(crate) fn create_record(&self, path: &Path) -> miette::Result<Record> {
        let location = self.location(path);

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

        Ok(WalkDir::new(path)
            .sort_by(crate::utils::file::sort_by_name_files_before_dirs)
            .into_iter()
            .filter_entry(|e| !crate::utils::file::is_hidden(e))
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

    /// Lists all valid locations in the store.
    ///
    /// Locations may be either directories or files that represent a record.
    pub(crate) fn list_locations(&self) -> miette::Result<Vec<PathBuf>> {
        Ok(WalkDir::new(&self.root)
            .sort_by(crate::utils::file::sort_by_name_files_before_dirs)
            .into_iter()
            .filter_entry(|e| !crate::utils::file::is_hidden(e))
            .flat_map(|e| e.ok())
            .flat_map(|e| {
                let e = e.path();

                if e.is_file() || e.is_dir() {
                    Some(e.strip_prefix(&self.root).unwrap().to_owned())
                } else {
                    None
                }
            })
            .collect())
    }

    /// Get a record given a path in the store.
    pub(crate) fn get_record(&self, path: &Path) -> miette::Result<Record> {
        let exact_location = self.location(path);

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
        let exact_location = self.location(path);

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

    /// Return a list of all paths that lead to values.
    ///
    /// Will only give results for files that are valid YAML or JSON encoded SOPS files.
    pub(crate) fn list_attributes(&self) -> miette::Result<Vec<String>> {
        let content = std::fs::read_to_string(self.location.filename()).into_diagnostic()?;

        fn handle_json_object(object: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
            object
                .iter()
                .flat_map(|(k, v)| {
                    // Ignore the SOPS internal data
                    if k == "sops" {
                        vec![]
                    } else if v.is_string() {
                        vec![k.to_owned()]
                    } else if v.is_object() {
                        handle_json_object(v.as_object().unwrap())
                            .iter()
                            .map(|i| format!("{k}/{i}"))
                            .collect()
                    } else {
                        vec![]
                    }
                })
                .collect()
        }

        // Try to parse the encrypted SOPS file as JSON
        if let Ok(serde_json::Value::Object(map)) =
            serde_json::from_str::<serde_json::Value>(&content)
        {
            return Ok(handle_json_object(&map));
        }

        fn handle_yaml_mapping(mapping: &saphyr::Mapping<'_>) -> Vec<String> {
            mapping
                .iter()
                .flat_map(|(k, v)| {
                    let k = k.as_str().unwrap();

                    // Ignore the SOPS internal data
                    if k == "sops" {
                        vec![]
                    } else if v.is_string() {
                        vec![k.to_owned()]
                    } else if v.is_mapping() {
                        handle_yaml_mapping(v.as_mapping().unwrap())
                            .iter()
                            .map(|i| format!("{k}/{i}"))
                            .collect()
                    } else {
                        vec![]
                    }
                })
                .collect()
        }

        // Try to parse the encrypted SOPS file as YAML
        if let Ok(yaml) = saphyr::Yaml::load_from_str(&content) {
            // Assuming that the file has a single YAML document
            if let Some(yaml) = yaml.first() {
                if let Some(map) = yaml.as_mapping() {
                    return Ok(handle_yaml_mapping(map));
                }
            }
        }

        Err(miette::miette!("File is not valid JSON or YAML"))
    }
}

fn format_selector(selector: Option<&str>) -> Option<String> {
    selector.map(|s| {
        if s.contains('[') || s.contains(']') {
            // If the string contains brackets, assume it's an already formatted selector
            s.to_string()
        } else if s.contains('/') {
            // With slashes, create a string path selector
            s.split('/')
                .flat_map(|part| {
                    if part.is_empty() {
                        None
                    } else {
                        Some(format!("[\"{}\"]", part))
                    }
                })
                .collect::<Vec<_>>()
                .join("")
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
    fn create_and_get_record() {
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
    fn list_records() {
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
    fn get_record_unchecked() {
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
    fn format_selector_none() {
        assert_eq!(format_selector(None), None);
    }

    #[test]
    fn format_selector_basic() {
        assert_eq!(format_selector(Some("foo")), Some("[\"foo\"]".to_string()));
    }

    #[test]
    fn format_selector_selector() {
        assert_eq!(
            format_selector(Some("[\"foo\"][\"bar\"]")),
            Some("[\"foo\"][\"bar\"]".to_string())
        );
    }

    #[test]
    fn format_selector_slashes_1() {
        assert_eq!(
            format_selector(Some("foo/bar")),
            Some("[\"foo\"][\"bar\"]".to_string())
        );
    }

    #[test]
    fn format_selector_slashes_2() {
        assert_eq!(
            format_selector(Some("/foo//bar/")),
            Some("[\"foo\"][\"bar\"]".to_string())
        );
    }
}
