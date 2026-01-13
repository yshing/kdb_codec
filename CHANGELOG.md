# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-01-14

### Added

- **K::ipc_msg_decode()** - New public API for decoding complete IPC messages with automatic header parsing and decompression
  - Parses 8-byte IPC message headers using `MessageHeader::from_bytes()`
  - Automatically decompresses messages when the compressed flag is set
  - Returns tuple `(MessageHeader, K)` with both header metadata and decoded payload
  - Symmetric counterpart to `K::ipc_msg_encode()` for complete encode/decode workflow
  - Comprehensive unit tests covering uncompressed, compressed, and error cases

## [1.0.0] - 2026-01-14

### 🎉 Major Milestone: Complete IPC Message Type Support

This release marks a significant milestone with **complete roundtrip support for all kdb+ IPC message types**, making the library production-ready for comprehensive q/kdb+ integration.

### Added

- **Complete IPC Type Coverage** - Added support for all remaining q function/adverb types:
  - Type 100: Lambda functions (`{x+y}`)
  - Type 101: Unary primitives and null (`::`)
  - Type 102: Binary operators
  - Type 104: Projection
  - Type 105: Composition
  - Type 106-111: Each adverbs (`'`, `/:`, `\:`, `':`) 
  - Type 112: Over/Scan adverbs
- **K::ipc_msg_encode()** - New public API for serializing K objects to complete IPC messages
  - Generates proper 8-byte IPC message headers
  - Optional compression support (equivalent to q `-18!`)
  - Automatic fallback to uncompressed when compression isn't beneficial
  - Comprehensive unit tests for all encoding scenarios
- **Enhanced Serialization** - Full support for table attributes (`s#` for sorted tables)
- **E2E Testing** - Added end-to-end acceptor decoding tests for real-world validation

### Changed

- **Opaque Roundtrip Pattern** - Function types now use opaque payload storage for safe roundtrip
  - Preserves exact wire format without requiring full semantic understanding
  - Ensures compatibility with all q versions and edge cases
- Improved null/unary primitive handling with proper type byte encoding

### Fixed

- Table attribute serialization now correctly preserves sorted/unique/parted/grouped flags
- Documentation test fixes for error handling examples
- Decompression bomb test edge case handling

### Documentation

- Added comprehensive examples for lambda and function type handling
- Updated type mapping documentation to reflect 100% IPC type coverage
- New example: `echo_acceptor.rs` demonstrating bidirectional IPC communication

### Security

- All new type handlers follow the same strict validation patterns
- Decompression bomb protection applies to all message types
- Safe handling of untrusted function payloads via opaque storage

### Notes

**Production Readiness**: With complete IPC type support, comprehensive security hardening, and extensive test coverage, this library is now suitable for production use in demanding environments. All kdb+ IPC message types can be safely encoded, decoded, and round-tripped.

## [0.3.1] - 2026-01-02

### Security

- Hardened IPC deserialization against malformed/untrusted input (panic-free decoding)
- Added additional bounds/resource checks to reduce risk of memory exhaustion during decode

### Changed

- Improved error messaging for unsupported/invalid q type bytes

### Documentation

- Simplified README and linked to the hosted documentation site
- Added a docs page describing supported type mapping and coverage

## [0.3.0] - 2025-12-30

### Added

- **Index Trait Implementation** for intuitive K object data access
  - `Index<usize>` for dictionary access: `dict[0]` (keys), `dict[1]` (values)
  - `Index<&str>` for table column access: `table["column_name"]`
  - `IndexMut<usize>` and `IndexMut<&str>` for mutable access
  - Safe access methods: `try_index()`, `try_index_mut()`, `try_column()`, `try_column_mut()`
  - Support for compound list indexing
- New example: `index_trait_demo.rs` demonstrating Index trait usage
- Comprehensive documentation for all Index trait implementations

### Changed

- **Enhanced `k!` macro** now supports `vec![value; count]` repetition syntax
  - Allows concise creation of large lists: `k!(long: vec![42; 3000])`
  - Supported for all list types (byte, short, int, long, real, float, symbol, temporal types)
  - Works with attributes: `k!(long: vec![1; 2500]; @sorted)`
- Simplified test code using `k!` macro throughout `index.rs` and `codec.rs`
- Improved code readability with 50-67% reduction in boilerplate

### Documentation

- Added `INDEX_TRAIT.md` with complete design rationale
- Updated `k!` macro documentation with repetition syntax examples
- Updated README with Index trait usage examples

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

[Unreleased]: https://github.com/yshing/kdb_codec/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/yshing/kdb_codec/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/yshing/kdb_codec/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/yshing/kdb_codec/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/yshing/kdb_codec/releases/tag/v0.1.0
