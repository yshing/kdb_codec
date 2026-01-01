//! Synchronous deserialization for codec
//!
//! This module provides synchronous deserialization functions for use with the codec pattern.
//! It's based on the async deserialization but removes unnecessary async/await.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::*;
use std::convert::TryInto;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Macros
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Read given bytes with a given cursor and build a basic type element of the specified type.
macro_rules! build_element {
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, i16) => {{
        if $cursor + 2 > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: 2,
                available: $bytes.len().saturating_sub($cursor),
            });
        }
        let element = match $encode {
            0 => i16::from_be_bytes($bytes[$cursor..$cursor + 2].try_into().unwrap()),
            _ => i16::from_le_bytes($bytes[$cursor..$cursor + 2].try_into().unwrap()),
        };
        Ok((
            K::new($qtype, qattribute::NONE, k0_inner::short(element)),
            $cursor + 2,
        ))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, i32) => {{
        if $cursor + 4 > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: 4,
                available: $bytes.len().saturating_sub($cursor),
            });
        }
        let element = match $encode {
            0 => i32::from_be_bytes($bytes[$cursor..$cursor + 4].try_into().unwrap()),
            _ => i32::from_le_bytes($bytes[$cursor..$cursor + 4].try_into().unwrap()),
        };
        Ok((
            K::new($qtype, qattribute::NONE, k0_inner::int(element)),
            $cursor + 4,
        ))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, i64) => {{
        if $cursor + 8 > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: 8,
                available: $bytes.len().saturating_sub($cursor),
            });
        }
        let element = match $encode {
            0 => i64::from_be_bytes($bytes[$cursor..$cursor + 8].try_into().unwrap()),
            _ => i64::from_le_bytes($bytes[$cursor..$cursor + 8].try_into().unwrap()),
        };
        Ok((
            K::new($qtype, qattribute::NONE, k0_inner::long(element)),
            $cursor + 8,
        ))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, f32) => {{
        if $cursor + 4 > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: 4,
                available: $bytes.len().saturating_sub($cursor),
            });
        }
        let element = match $encode {
            0 => f32::from_be_bytes($bytes[$cursor..$cursor + 4].try_into().unwrap()),
            _ => f32::from_le_bytes($bytes[$cursor..$cursor + 4].try_into().unwrap()),
        };
        Ok((
            K::new($qtype, qattribute::NONE, k0_inner::real(element)),
            $cursor + 4,
        ))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, f64) => {{
        if $cursor + 8 > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: 8,
                available: $bytes.len().saturating_sub($cursor),
            });
        }
        let element = match $encode {
            0 => f64::from_be_bytes($bytes[$cursor..$cursor + 8].try_into().unwrap()),
            _ => f64::from_le_bytes($bytes[$cursor..$cursor + 8].try_into().unwrap()),
        };
        Ok((
            K::new($qtype, qattribute::NONE, k0_inner::float(element)),
            $cursor + 8,
        ))
    }};
}

/// Read given bytes with a given cursor and build a basic type list of the specified type.
macro_rules! build_list {
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, i16, $max_list_size:expr) => {{
        let (attribute, size, cursor) =
            get_attribute_and_size($bytes, $cursor, $encode, $max_list_size)?;
        let byte_count = size.checked_mul(2).ok_or(Error::SizeOverflow)?;
        if cursor + byte_count > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: byte_count,
                available: $bytes.len().saturating_sub(cursor),
            });
        }
        let list = match $encode {
            0 => $bytes[cursor..cursor + byte_count]
                .chunks(2)
                .map(|element| i16::from_be_bytes(element.try_into().unwrap()))
                .collect::<Vec<H>>(),
            _ => $bytes[cursor..cursor + byte_count]
                .chunks(2)
                .map(|element| i16::from_le_bytes(element.try_into().unwrap()))
                .collect::<Vec<H>>(),
        };
        let k = K::new($qtype, attribute, k0_inner::list(k0_list::new(list)));
        Ok((k, cursor + byte_count))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, i32, $max_list_size:expr) => {{
        let (attribute, size, cursor) =
            get_attribute_and_size($bytes, $cursor, $encode, $max_list_size)?;
        let byte_count = size.checked_mul(4).ok_or(Error::SizeOverflow)?;
        if cursor + byte_count > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: byte_count,
                available: $bytes.len().saturating_sub(cursor),
            });
        }
        let list = match $encode {
            0 => $bytes[cursor..cursor + byte_count]
                .chunks(4)
                .map(|element| i32::from_be_bytes(element.try_into().unwrap()))
                .collect::<Vec<I>>(),
            _ => $bytes[cursor..cursor + byte_count]
                .chunks(4)
                .map(|element| i32::from_le_bytes(element.try_into().unwrap()))
                .collect::<Vec<I>>(),
        };
        let k = K::new($qtype, attribute, k0_inner::list(k0_list::new(list)));
        Ok((k, cursor + byte_count))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, i64, $max_list_size:expr) => {{
        let (attribute, size, cursor) =
            get_attribute_and_size($bytes, $cursor, $encode, $max_list_size)?;
        let byte_count = size.checked_mul(8).ok_or(Error::SizeOverflow)?;
        if cursor + byte_count > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: byte_count,
                available: $bytes.len().saturating_sub(cursor),
            });
        }
        let list = match $encode {
            0 => $bytes[cursor..cursor + byte_count]
                .chunks(8)
                .map(|element| i64::from_be_bytes(element.try_into().unwrap()))
                .collect::<Vec<J>>(),
            _ => $bytes[cursor..cursor + byte_count]
                .chunks(8)
                .map(|element| i64::from_le_bytes(element.try_into().unwrap()))
                .collect::<Vec<J>>(),
        };
        let k = K::new($qtype, attribute, k0_inner::list(k0_list::new(list)));
        Ok((k, cursor + byte_count))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, f32, $max_list_size:expr) => {{
        let (attribute, size, cursor) =
            get_attribute_and_size($bytes, $cursor, $encode, $max_list_size)?;
        let byte_count = size.checked_mul(4).ok_or(Error::SizeOverflow)?;
        if cursor + byte_count > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: byte_count,
                available: $bytes.len().saturating_sub(cursor),
            });
        }
        let list = match $encode {
            0 => $bytes[cursor..cursor + byte_count]
                .chunks(4)
                .map(|element| f32::from_be_bytes(element.try_into().unwrap()))
                .collect::<Vec<E>>(),
            _ => $bytes[cursor..cursor + byte_count]
                .chunks(4)
                .map(|element| f32::from_le_bytes(element.try_into().unwrap()))
                .collect::<Vec<E>>(),
        };
        let k = K::new($qtype, attribute, k0_inner::list(k0_list::new(list)));
        Ok((k, cursor + byte_count))
    }};
    ($bytes:expr, $cursor:expr, $encode:expr, $qtype:expr, f64, $max_list_size:expr) => {{
        let (attribute, size, cursor) =
            get_attribute_and_size($bytes, $cursor, $encode, $max_list_size)?;
        let byte_count = size.checked_mul(8).ok_or(Error::SizeOverflow)?;
        if cursor + byte_count > $bytes.len() {
            return Err(Error::InsufficientData {
                needed: byte_count,
                available: $bytes.len().saturating_sub(cursor),
            });
        }
        let list = match $encode {
            0 => $bytes[cursor..cursor + byte_count]
                .chunks(8)
                .map(|element| f64::from_be_bytes(element.try_into().unwrap()))
                .collect::<Vec<F>>(),
            _ => $bytes[cursor..cursor + byte_count]
                .chunks(8)
                .map(|element| f64::from_le_bytes(element.try_into().unwrap()))
                .collect::<Vec<F>>(),
        };
        let k = K::new($qtype, attribute, k0_inner::list(k0_list::new(list)));
        Ok((k, cursor + byte_count))
    }};
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

impl K {
    /// Synchronously decode q object from bytes in a manner of q function `-8!`.
    /// Returns Result to handle errors gracefully instead of panicking.
    /// Uses default security limits for list size and recursion depth.
    pub fn q_ipc_decode(bytes: &[u8], encode: u8) -> Result<K> {
        q_ipc_decode_sync(
            bytes,
            encode,
            crate::MAX_LIST_SIZE,
            crate::MAX_RECURSION_DEPTH,
        )
    }
}

/// Synchronously decode K object from bytes (for codec)
pub(crate) fn q_ipc_decode_sync(
    bytes: &[u8],
    encode: u8,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<K> {
    deserialize_bytes_sync(bytes, 0, encode, 0, max_list_size, max_recursion_depth).map(|(k, _)| k)
}

fn deserialize_bytes_sync(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    // Check recursion depth
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    // Type of q object is stored in a byte
    if cursor >= bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: 0,
        });
    }

    let qtype = bytes[cursor] as i8;

    match qtype {
        qtype::BOOL_ATOM => deserialize_bool(bytes, cursor + 1, encode),
        qtype::GUID_ATOM => deserialize_guid(bytes, cursor + 1, encode),
        qtype::BYTE_ATOM => deserialize_byte(bytes, cursor + 1, encode),
        qtype::SHORT_ATOM => build_element!(bytes, cursor + 1, encode, qtype::SHORT_ATOM, i16),
        qtype::INT_ATOM => build_element!(bytes, cursor + 1, encode, qtype::INT_ATOM, i32),
        qtype::LONG_ATOM => build_element!(bytes, cursor + 1, encode, qtype::LONG_ATOM, i64),
        qtype::REAL_ATOM => build_element!(bytes, cursor + 1, encode, qtype::REAL_ATOM, f32),
        qtype::FLOAT_ATOM => build_element!(bytes, cursor + 1, encode, qtype::FLOAT_ATOM, f64),
        qtype::CHAR => deserialize_char(bytes, cursor + 1, encode),
        qtype::SYMBOL_ATOM => deserialize_symbol(bytes, cursor + 1, encode),
        qtype::TIMESTAMP_ATOM => {
            build_element!(bytes, cursor + 1, encode, qtype::TIMESTAMP_ATOM, i64)
        }
        qtype::MONTH_ATOM => build_element!(bytes, cursor + 1, encode, qtype::MONTH_ATOM, i32),
        qtype::DATE_ATOM => build_element!(bytes, cursor + 1, encode, qtype::DATE_ATOM, i32),
        qtype::DATETIME_ATOM => {
            build_element!(bytes, cursor + 1, encode, qtype::DATETIME_ATOM, f64)
        }
        qtype::TIMESPAN_ATOM => {
            build_element!(bytes, cursor + 1, encode, qtype::TIMESPAN_ATOM, i64)
        }
        qtype::MINUTE_ATOM => build_element!(bytes, cursor + 1, encode, qtype::MINUTE_ATOM, i32),
        qtype::SECOND_ATOM => build_element!(bytes, cursor + 1, encode, qtype::SECOND_ATOM, i32),
        qtype::TIME_ATOM => build_element!(bytes, cursor + 1, encode, qtype::TIME_ATOM, i32),
        qtype::COMPOUND_LIST => deserialize_compound_list_sync(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::BOOL_LIST => deserialize_bool_list(bytes, cursor + 1, encode, max_list_size),
        qtype::GUID_LIST => deserialize_guid_list_sync(bytes, cursor + 1, encode, max_list_size),
        qtype::BYTE_LIST => deserialize_byte_list(bytes, cursor + 1, encode, max_list_size),
        qtype::SHORT_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::SHORT_LIST,
            i16,
            max_list_size
        ),
        qtype::INT_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::INT_LIST,
            i32,
            max_list_size
        ),
        qtype::LONG_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::LONG_LIST,
            i64,
            max_list_size
        ),
        qtype::REAL_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::REAL_LIST,
            f32,
            max_list_size
        ),
        qtype::FLOAT_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::FLOAT_LIST,
            f64,
            max_list_size
        ),
        qtype::STRING => deserialize_string(bytes, cursor + 1, encode, max_list_size),
        qtype::SYMBOL_LIST => {
            deserialize_symbol_list_sync(bytes, cursor + 1, encode, max_list_size)
        }
        qtype::TIMESTAMP_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::TIMESTAMP_LIST,
            i64,
            max_list_size
        ),
        qtype::MONTH_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::MONTH_LIST,
            i32,
            max_list_size
        ),
        qtype::DATE_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::DATE_LIST,
            i32,
            max_list_size
        ),
        qtype::DATETIME_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::DATETIME_LIST,
            f64,
            max_list_size
        ),
        qtype::TIMESPAN_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::TIMESPAN_LIST,
            i64,
            max_list_size
        ),
        qtype::MINUTE_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::MINUTE_LIST,
            i32,
            max_list_size
        ),
        qtype::SECOND_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::SECOND_LIST,
            i32,
            max_list_size
        ),
        qtype::TIME_LIST => build_list!(
            bytes,
            cursor + 1,
            encode,
            qtype::TIME_LIST,
            i32,
            max_list_size
        ),
        qtype::TABLE => deserialize_table_sync(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::DICTIONARY | qtype::SORTED_DICTIONARY => deserialize_dictionary_sync(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::NULL => deserialize_null(bytes, cursor + 1, encode),
        qtype::ERROR => deserialize_error(bytes, cursor + 1, encode),
        _ => Err(Error::InvalidType(qtype)),
    }
}

fn deserialize_bool(bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    if cursor + 1 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    Ok((K::new_bool(bytes[cursor] != 0), cursor + 1))
}

fn deserialize_guid(bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    if cursor + 16 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 16,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    Ok((
        K::new_guid(bytes[cursor..cursor + 16].try_into().unwrap()),
        cursor + 16,
    ))
}

fn deserialize_byte(bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    if cursor + 1 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    Ok((K::new_byte(bytes[cursor]), cursor + 1))
}

fn deserialize_char(bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    if cursor + 1 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    Ok((
        K::new(qtype::CHAR, qattribute::NONE, k0_inner::byte(bytes[cursor])),
        cursor + 1,
    ))
}

fn deserialize_symbol(bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    if cursor >= bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: 0,
        });
    }

    let null_location = bytes
        .split_at(cursor)
        .1
        .iter()
        .position(|b| *b == 0x00)
        .ok_or(Error::MissingNullTerminator)?;

    let symbol_str = String::from_utf8(bytes[cursor..cursor + null_location].to_vec())
        .map_err(|_| Error::InvalidUtf8)?;
    let k = K::new_symbol(symbol_str);
    Ok((k, cursor + null_location + 1))
}

/// Extract attribute and list length and then proceed the cursor.
fn get_attribute_and_size(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    max_list_size: usize,
) -> Result<(i8, usize, usize)> {
    // Ensure we have enough bytes for attribute (1) + size (4)
    if cursor + 5 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 5,
            available: bytes.len().saturating_sub(cursor),
        });
    }

    let size_u32 = match encode {
        0 => u32::from_be_bytes(bytes[cursor + 1..cursor + 5].try_into().unwrap()),
        _ => u32::from_le_bytes(bytes[cursor + 1..cursor + 5].try_into().unwrap()),
    };

    let size = size_u32 as usize;

    // Validate size is reasonable
    if size > max_list_size {
        return Err(Error::ListTooLarge {
            size,
            max: max_list_size,
        });
    }

    Ok((bytes[cursor] as i8, size, cursor + 5))
}

fn deserialize_bool_list(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    max_list_size: usize,
) -> Result<(K, usize)> {
    let (attribute, size, cursor) = get_attribute_and_size(bytes, cursor, encode, max_list_size)?;
    if cursor + size > bytes.len() {
        return Err(Error::InsufficientData {
            needed: size,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    let list = bytes[cursor..cursor + size].to_vec();
    Ok((
        K::new(
            qtype::BOOL_LIST,
            attribute,
            k0_inner::list(k0_list::new(list)),
        ),
        cursor + size,
    ))
}

fn deserialize_guid_list_sync(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    max_list_size: usize,
) -> Result<(K, usize)> {
    let (attribute, size, cursor) = get_attribute_and_size(bytes, cursor, encode, max_list_size)?;
    let byte_count = size.checked_mul(16).ok_or(Error::SizeOverflow)?;
    if cursor + byte_count > bytes.len() {
        return Err(Error::InsufficientData {
            needed: byte_count,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    let list = bytes[cursor..cursor + byte_count]
        .chunks(16)
        .map(|guid| guid.try_into().unwrap())
        .collect::<Vec<U>>();
    Ok((K::new_guid_list(list, attribute), cursor + byte_count))
}

fn deserialize_byte_list(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    max_list_size: usize,
) -> Result<(K, usize)> {
    let (attribute, size, cursor) = get_attribute_and_size(bytes, cursor, encode, max_list_size)?;
    if cursor + size > bytes.len() {
        return Err(Error::InsufficientData {
            needed: size,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    let list = bytes[cursor..cursor + size].to_vec();
    Ok((
        K::new(
            qtype::BYTE_LIST,
            attribute,
            k0_inner::list(k0_list::new(list)),
        ),
        cursor + size,
    ))
}

fn deserialize_string(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    max_list_size: usize,
) -> Result<(K, usize)> {
    let (attribute, size, cursor) = get_attribute_and_size(bytes, cursor, encode, max_list_size)?;
    if cursor + size > bytes.len() {
        return Err(Error::InsufficientData {
            needed: size,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    let string =
        String::from_utf8(bytes[cursor..cursor + size].to_vec()).map_err(|_| Error::InvalidUtf8)?;
    Ok((
        K::new(qtype::STRING, attribute, k0_inner::symbol(string)),
        cursor + size,
    ))
}

fn deserialize_symbol_list_sync(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    max_list_size: usize,
) -> Result<(K, usize)> {
    let (attribute, size, mut cursor) =
        get_attribute_and_size(bytes, cursor, encode, max_list_size)?;
    let mut list = Vec::with_capacity(size);
    for _ in 0..size {
        if cursor >= bytes.len() {
            return Err(Error::InsufficientData {
                needed: 1,
                available: 0,
            });
        }
        let null_location = bytes
            .split_at(cursor)
            .1
            .iter()
            .position(|b| *b == 0x00)
            .ok_or(Error::MissingNullTerminator)?;
        let symbol = String::from_utf8(bytes[cursor..cursor + null_location].to_vec())
            .map_err(|_| Error::InvalidUtf8)?;
        list.push(symbol);
        cursor += null_location + 1;
    }
    Ok((K::new_symbol_list(list, attribute), cursor))
}

fn deserialize_compound_list_sync(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    let (attribute, size, mut cursor) =
        get_attribute_and_size(bytes, cursor, encode, max_list_size)?;
    let mut list = Vec::with_capacity(size);
    for _ in 0..size {
        let (k, new_cursor) = deserialize_bytes_sync(
            bytes,
            cursor,
            encode,
            depth + 1,
            max_list_size,
            max_recursion_depth,
        )?;
        list.push(k);
        cursor = new_cursor;
    }
    let mut k = K::new_compound_list(list);
    k.0.attribute = attribute;
    Ok((k, cursor))
}

fn deserialize_table_sync(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    // Table format: [attribute (1 byte)] [dictionary_qtype (1 byte)] [dictionary_data]
    // Ensure we have at least 2 bytes
    if cursor + 2 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 2,
            available: bytes.len().saturating_sub(cursor),
        });
    }

    // Skip attribute byte
    let _attribute = bytes[cursor] as i8;
    // Skip dictionary qtype byte (should be 99 or 127)
    let _dict_qtype = bytes[cursor + 1] as i8;
    let cursor = cursor + 2;

    // Deserialize the dictionary (keys and values)
    let (dictionary, cursor) = deserialize_dictionary_sync(
        bytes,
        cursor,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;
    Ok((
        K::new(qtype::TABLE, qattribute::NONE, k0_inner::table(dictionary)),
        cursor,
    ))
}

fn deserialize_dictionary_sync(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    // Deserialize keys
    let (keys, cursor) = deserialize_bytes_sync(
        bytes,
        cursor,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;
    // Deserialize values
    let (values, cursor) = deserialize_bytes_sync(
        bytes,
        cursor,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;
    // Build dictionary - new_dictionary handles sorted and keyed tables internally
    let dictionary = K::new_dictionary(keys, values)
        .map_err(|e| Error::DeserializationError(format!("Failed to build dictionary: {}", e)))?;
    Ok((dictionary, cursor))
}

fn deserialize_null(_bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    Ok((
        K::new(qtype::NULL, qattribute::NONE, k0_inner::null(())),
        cursor,
    ))
}

fn deserialize_error(bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    if cursor >= bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: 0,
        });
    }

    let null_location = bytes
        .split_at(cursor)
        .1
        .iter()
        .position(|b| *b == 0x00)
        .ok_or(Error::MissingNullTerminator)?;

    let error_msg = String::from_utf8(bytes[cursor..cursor + null_location].to_vec())
        .map_err(|_| Error::InvalidUtf8)?;

    let k = K::new(qtype::ERROR, qattribute::NONE, k0_inner::symbol(error_msg));
    Ok((k, cursor + null_location + 1))
}
