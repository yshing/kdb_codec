# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2025-12-27

### Added

- **Explicit Compression Control**: New `CompressionMode` enum with three options:
  - `Auto` (default): Compress messages >2000 bytes on remote connections only
  - `Always`: Compress messages >2000 bytes regardless of connection type
  - `Never`: Disable compression entirely
  
- **Header Validation**: New `ValidationMode` enum for configurable validation strictness:
  - `Strict` (default): Reject invalid compressed flags (not 0/1) and message types (not 0/1/2)
  - `Lenient`: Accept any header values for debugging/non-standard implementations
  
- **Builder Pattern Support**: Added `bon` crate and implemented fluent builder APIs:
  - `KdbCodec::builder()` - Build codecs with a fluent API
  - `QStream::builder()` - Build QStream connections with a fluent API
  - All parameters have sensible defaults
  
- **New API Methods**:
  - `KdbCodec::with_options()` - Create codec with explicit compression and validation modes
  - `KdbCodec::set_compression_mode()` - Change compression mode at runtime
  - `KdbCodec::set_validation_mode()` - Change validation mode at runtime
  - `KdbCodec::compression_mode()` - Get current compression mode
  - `KdbCodec::validation_mode()` - Get current validation mode
  - `QStream::connect_with_options()` - Connect with explicit compression/validation modes
  - `QStream::accept_with_options()` - Accept connections with explicit compression/validation modes

### Changed

- Decoder now validates message headers in `Strict` mode (default) and provides clear error messages for invalid headers
- Documentation updated with comprehensive examples for all new features

### Dependencies

- Added `bon = "3"` for builder pattern support

## [0.4.0] - Previous Release

Initial release with core kdb+ IPC codec functionality.

[Unreleased]: https://github.com/yshing/kdb_codec/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/yshing/kdb_codec/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/yshing/kdb_codec/releases/tag/v0.4.0
