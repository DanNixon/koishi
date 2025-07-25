# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/DanNixon/koishi/compare/v0.1.1...v0.1.2) - 2025-07-16

### Added

- cli completion for store locations
- allow `rm` command to work on both records and directories
- allow `mv` command to work on both records and directories
- choose characters from a secret

### Fixed

- fix failing style/formtting check

### Other

- remove old notes from `set.rs`
- `StoreLocation` generation from a `Store`

## [0.1.1](https://github.com/DanNixon/koishi/compare/v0.1.0...v0.1.1) - 2025-07-07

### Added

- allow selecting a nested attribute from a secret in interactive mode
- add cli completion
- string path selectors to `koishi get`

### Other

- *(deps)* disable unused feature flags
- add readme badges

## [0.1.0](https://github.com/DanNixon/koishi/releases/tag/v0.1.0) - 2025-07-01

### Added

- Initial commit

### Other

- add release-plz
