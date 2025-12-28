# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2025-12-28

This release significantly improves the usability and flexibility of the kdb_codec library by adding explicit compression control, header validation, and a modern builder pattern API.

### Added

#### Compression Control
- **`CompressionMode` enum** for explicit compression control with three options:
  - `Auto` (default): Automatically compress messages larger than 2000 bytes on remote connections only
  - `Always`: Always attempt to compress messages larger than 2000 bytes, even on local connections
  - `Never`: Disable compression entirely for all messages
- `KdbCodec::set_compression_mode()` - Change compression mode dynamically at runtime
- `KdbCodec::compression_mode()` - Query the current compression mode

#### Header Validation
- **`ValidationMode` enum** for configurable validation strictness:
  - `Strict` (default): Reject messages with invalid compressed flags (not 0 or 1) and message types (not 0, 1, or 2)
  - `Lenient`: Accept any header values for debugging or handling non-standard implementations
- Clear, descriptive error messages when validation fails in Strict mode
- `KdbCodec::set_validation_mode()` - Change validation mode dynamically at runtime
- `KdbCodec::validation_mode()` - Query the current validation mode

#### Builder Pattern Support
- **Builder pattern API** using the `bon` crate for both `KdbCodec` and `QStream`
- `KdbCodec::builder()` - Fluent API for building codecs with optional parameters
- `QStream::builder()` - Fluent API for building QStream connections with optional parameters
- All builder parameters have sensible defaults, allowing users to specify only what they need
- Compile-time type safety for builder parameters

#### New API Methods
- `KdbCodec::with_options(is_local, compression_mode, validation_mode)` - Create codec with explicit options
- `QStream::connect_with_options(method, host, port, credential, compression_mode, validation_mode)` - Connect with explicit compression and validation control
- `QStream::accept_with_options(method, host, port, compression_mode, validation_mode)` - Accept connections with explicit compression and validation control

#### Examples
- `compression_validation_demo.rs` - Comprehensive example demonstrating compression modes and validation
- `builder_pattern_demo.rs` - Example showcasing the builder pattern API

### Changed

- **Decoder validation**: The decoder now validates message headers in `Strict` mode (default) and returns clear error messages for protocol violations
- **Documentation**: Updated README, lib.rs, and inline documentation with comprehensive examples for all new features
- **Test coverage**: Added 12 new tests covering compression modes, validation modes, builder pattern, and getter/setter methods

### Dependencies

- Added `bon = "3"` for builder pattern support

### Migration Guide

This release is fully backward compatible. Existing code continues to work without any changes.

To use the new features:

```rust
// Compression control
let codec = KdbCodec::builder()
    .compression_mode(CompressionMode::Always)
    .build();

// Header validation
let codec = KdbCodec::builder()
    .validation_mode(ValidationMode::Lenient)
    .build();

// QStream with builder
let stream = QStream::builder()
    .method(ConnectionMethod::TCP)
    .host("localhost")
    .port(5000)
    .credential("user:pass")
    .compression_mode(CompressionMode::Never)
    .validation_mode(ValidationMode::Strict)
    .connect()
    .await?;
```

## [0.4.0] - 2025-12-26

Initial release with core kdb+ IPC codec functionality.

### Added
- Tokio-based codec implementation for kdb+ IPC protocol
- `KdbCodec` for encoding and decoding kdb+ messages
- `QStream` high-level client for connecting to q/kdb+ processes
- Full compression/decompression support compatible with kdb+ (-18!/-19!)
- Multiple connection methods: TCP, TLS, and Unix Domain Socket
- Cancellation-safe message handling with `tokio-util::codec::Framed`
- Type-safe K struct for all kdb+ data types
- Synchronous deserialization without async recursion

[Unreleased]: https://github.com/yshing/kdb_codec/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/yshing/kdb_codec/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/yshing/kdb_codec/releases/tag/v0.4.0
