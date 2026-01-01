# Type Mapping & Coverage

This page documents what q/kdb+ datatypes `kdb_codec` currently supports for IPC **encoding/decoding**, how those types map to Rust, and which q types are not supported yet.

## Goals

- Safe decoding of untrusted IPC bytes (no panics, no OOM)
- Clear, explicit behavior for unsupported q types (return an error)

## Supported q types (IPC)

The IPC encoder/decoder currently supports:

- Basic atoms/lists (KX 0–19, and their corresponding atom type ids)
  - bool, guid, byte, short, int, long, real, float, char/string, symbol
  - timestamp, month, date, datetime, timespan, minute, second, time
- Mixed list (compound list): type `0`
- Table: type `98`
- Dictionary / sorted dictionary: type `99` / `127`
- General null: type `101`
- Error: type `-128`

If the decoder encounters a q type that is not supported, it returns an error (typically `Error::InvalidType`).

## Type mapping (q → Rust)

All q values are represented as the `K` struct. The following table summarizes the primary Rust types used by constructors/accessors.

| q | Rust |
|---|---|
| `bool` | `bool` |
| `GUID` | `[u8; 16]` |
| `byte` | `u8` |
| `short` | `i16` |
| `int` | `i32` |
| `long` | `i64` |
| `real` | `f32` |
| `float` | `f64` |
| `char` | `char` |
| `symbol` | `String` |
| `timestamp` | `chrono::DateTime<chrono::Utc>` |
| `month` | `chrono::NaiveDate` |
| `date` | `chrono::NaiveDate` |
| `datetime` | `chrono::DateTime<chrono::Utc>` |
| `timespan` | `chrono::Duration` |
| `minute` | `chrono::Duration` |
| `second` | `chrono::Duration` |
| `time` | `chrono::Duration` |
| `list` | `Vec<Item>` |
| `compound list` | `Vec<K>` |
| `table` | `Vec<K>` |
| `dictionary` | `Vec<K>` |
| `null` | `()` |

## Missing / not supported yet

Compared to the full datatype taxonomy described in KX documentation, the following categories have **partial** or **no** support by the IPC encoder/decoder:

### Partial support (decode-only)

- **Enums (KX type -20 atom, 20 list)**: The decoder can safely deserialize enum atoms and enum lists as integer values (i32). The underlying symbol mapping is not reconstructed; only the numeric indices are preserved. Encoding of enums is not yet supported. This conservative approach avoids unbounded allocations and maintains compatibility with the existing K API.
  - Security: Enum deserialization performs bounds checking and respects MAX_LIST_SIZE limits to prevent OOM attacks.
  - Implementation: Enum atoms are stored as INT_ATOM-compatible values, and enum lists are stored as INT_LIST-compatible vectors.

- **Foreign objects (type 112)**: The decoder can safely deserialize foreign objects as opaque byte payloads (Vec<u8>). No interpretation of the payload structure is performed. Encoding of foreign objects is not yet supported.
  - Security: Foreign object deserialization validates payload length against MAX_LIST_SIZE before allocation, preventing excessive memory consumption.
  - Implementation: Foreign objects are stored internally as BYTE_LIST-compatible vectors with the FOREIGN type marker.

### Not supported

- Nested/other types (KX 77+) beyond foreign objects
- Function/derived/iterator types (KX 100–111, 113+)

For implementation notes and deeper coverage discussions, please refer to the documentation site and repository history.

## Extending coverage

If you want to add support for additional q types, the typical work involves:

- Extending the internal `K` representation to carry the new payloads
- Adding safe deserialization in the IPC decoder (bounds checks + resource limits)
- Mirroring serialization support if round-trip is required
- Adding tests + fuzz coverage
