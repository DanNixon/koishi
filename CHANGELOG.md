# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1](https://github.com/DanNixon/koishi/compare/v0.2.0...v0.2.1) - 2026-01-10

### Added

- add `--yes` flag to `updatekeys`

### Fixed

- be consistent with clap derive syntax

### Other

- set Dependabot to quarterly
- replace devenv.sh with Nix flake
- *(deps)* bump the cargo-dependencies group with 5 updates ([#32](https://github.com/DanNixon/koishi/pull/32))
- *(deps)* bump the cargo-dependencies group with 4 updates ([#29](https://github.com/DanNixon/koishi/pull/29))
- disable persist-credentials on release jobs
- update devenv inputs
- *(deps)* bump actions/checkout from 5 to 6 ([#28](https://github.com/DanNixon/koishi/pull/28))
- *(deps)* bump the cargo-dependencies group with 3 updates ([#26](https://github.com/DanNixon/koishi/pull/26))

## [0.2.0](https://github.com/DanNixon/koishi/compare/v0.1.2...v0.2.0) - 2025-11-01

### Added

- [**breaking**] auto transforms for `get` command
- [**breaking**] only allow setting store location via env var ([#24](https://github.com/DanNixon/koishi/pull/24))

### Other

- *(deps)* bump the cargo-dependencies group across 1 directory with 6 updates ([#23](https://github.com/DanNixon/koishi/pull/23))
- *(deps)* update cargo dependencies
- *(deps)* bump actions/checkout from 4 to 5
- *(deps)* bump the cargo-dependencies group with 7 updates

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
