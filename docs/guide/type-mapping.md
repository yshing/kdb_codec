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

Compared to the full datatype taxonomy described in KX documentation, the following categories are currently **not** supported by the IPC encoder/decoder:

- Enums (KX 20–76)
- Nested/other types (KX 77+)
- Function/derived/iterator types (KX 100–112)
- Foreign objects (type `112`)

For implementation notes and deeper coverage discussions, please refer to the documentation site and repository history.

## Extending coverage

If you want to add support for additional q types, the typical work involves:

- Extending the internal `K` representation to carry the new payloads
- Adding safe deserialization in the IPC decoder (bounds checks + resource limits)
- Mirroring serialization support if round-trip is required
- Adding tests + fuzz coverage
