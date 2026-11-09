# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-11-07

### Added

- First crates.io release as [`kense-kcl`](https://crates.io/crates/kense-kcl)
  (Rust library name remains `kcl`).
- Crate-level documentation, example, LICENSE, and CI workflow.

### Changed

- Diagnostic and canonical wire codes use the `kcl.*` prefix
  (`kcl.parse.*`, `kcl.limit.*`, `kcl.schema.*`, `kcl.canonical.*`).
- Package metadata and documentation target the Kense Configuration Language
  brand.

[0.3.0]: https://github.com/kense-labs/kcl/releases/tag/v0.3.0
