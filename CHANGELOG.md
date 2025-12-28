# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-12-28

### Added

- Explicit compression control with `CompressionMode` enum (Auto, Always, Never)
- Header validation with `ValidationMode` enum (Strict, Lenient)
- Builder pattern support using `bon` crate for easier API usage
- `KdbCodec::builder()` for fluent codec construction
- `QStream::builder()` for fluent connection building
- `KdbCodec::with_options()` method for explicit configuration
- `QStream::connect_with_options()` and `QStream::accept_with_options()` methods
- Examples: `compression_validation_demo.rs` and `builder_pattern_demo.rs`

### Changed

- Decoder now validates message headers in Strict mode by default
- Updated documentation with new API examples

### Dependencies

- Added `bon = "3"` for builder pattern support

## [0.1.0] - 2025-12-26

Initial release with core kdb+ IPC codec functionality.

[Unreleased]: https://github.com/yshing/kdb_codec/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/yshing/kdb_codec/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yshing/kdb_codec/releases/tag/v0.1.0
