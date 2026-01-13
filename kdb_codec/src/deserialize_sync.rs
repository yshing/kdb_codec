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
        let element_bytes: [u8; 2] = $bytes[$cursor..$cursor + 2]
            .try_into()
            .map_err(|_| Error::DeserializationError("invalid i16 bytes".to_string()))?;
        let element = match $encode {
            0 => i16::from_be_bytes(element_bytes),
            _ => i16::from_le_bytes(element_bytes),
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
        let element_bytes: [u8; 4] = $bytes[$cursor..$cursor + 4]
            .try_into()
            .map_err(|_| Error::DeserializationError("invalid i32 bytes".to_string()))?;
        let element = match $encode {
            0 => i32::from_be_bytes(element_bytes),
            _ => i32::from_le_bytes(element_bytes),
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
        let element_bytes: [u8; 8] = $bytes[$cursor..$cursor + 8]
            .try_into()
            .map_err(|_| Error::DeserializationError("invalid i64 bytes".to_string()))?;
        let element = match $encode {
            0 => i64::from_be_bytes(element_bytes),
            _ => i64::from_le_bytes(element_bytes),
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
        let element_bytes: [u8; 4] = $bytes[$cursor..$cursor + 4]
            .try_into()
            .map_err(|_| Error::DeserializationError("invalid f32 bytes".to_string()))?;
        let element = match $encode {
            0 => f32::from_be_bytes(element_bytes),
            _ => f32::from_le_bytes(element_bytes),
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
        let element_bytes: [u8; 8] = $bytes[$cursor..$cursor + 8]
            .try_into()
            .map_err(|_| Error::DeserializationError("invalid f64 bytes".to_string()))?;
        let element = match $encode {
            0 => f64::from_be_bytes(element_bytes),
            _ => f64::from_le_bytes(element_bytes),
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
        let slice = &$bytes[cursor..cursor + byte_count];
        let mut list: Vec<H> = Vec::with_capacity(size);
        match $encode {
            0 => {
                let mut iter = slice.chunks_exact(2);
                for element in &mut iter {
                    let element_bytes: [u8; 2] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid i16 list bytes".to_string())
                    })?;
                    list.push(i16::from_be_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid i16 list alignment".to_string(),
                    ));
                }
            }
            _ => {
                let mut iter = slice.chunks_exact(2);
                for element in &mut iter {
                    let element_bytes: [u8; 2] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid i16 list bytes".to_string())
                    })?;
                    list.push(i16::from_le_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid i16 list alignment".to_string(),
                    ));
                }
            }
        }
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
        let slice = &$bytes[cursor..cursor + byte_count];
        let mut list: Vec<I> = Vec::with_capacity(size);
        match $encode {
            0 => {
                let mut iter = slice.chunks_exact(4);
                for element in &mut iter {
                    let element_bytes: [u8; 4] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid i32 list bytes".to_string())
                    })?;
                    list.push(i32::from_be_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid i32 list alignment".to_string(),
                    ));
                }
            }
            _ => {
                let mut iter = slice.chunks_exact(4);
                for element in &mut iter {
                    let element_bytes: [u8; 4] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid i32 list bytes".to_string())
                    })?;
                    list.push(i32::from_le_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid i32 list alignment".to_string(),
                    ));
                }
            }
        }
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
        let slice = &$bytes[cursor..cursor + byte_count];
        let mut list: Vec<J> = Vec::with_capacity(size);
        match $encode {
            0 => {
                let mut iter = slice.chunks_exact(8);
                for element in &mut iter {
                    let element_bytes: [u8; 8] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid i64 list bytes".to_string())
                    })?;
                    list.push(i64::from_be_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid i64 list alignment".to_string(),
                    ));
                }
            }
            _ => {
                let mut iter = slice.chunks_exact(8);
                for element in &mut iter {
                    let element_bytes: [u8; 8] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid i64 list bytes".to_string())
                    })?;
                    list.push(i64::from_le_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid i64 list alignment".to_string(),
                    ));
                }
            }
        }
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
        let slice = &$bytes[cursor..cursor + byte_count];
        let mut list: Vec<E> = Vec::with_capacity(size);
        match $encode {
            0 => {
                let mut iter = slice.chunks_exact(4);
                for element in &mut iter {
                    let element_bytes: [u8; 4] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid f32 list bytes".to_string())
                    })?;
                    list.push(f32::from_be_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid f32 list alignment".to_string(),
                    ));
                }
            }
            _ => {
                let mut iter = slice.chunks_exact(4);
                for element in &mut iter {
                    let element_bytes: [u8; 4] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid f32 list bytes".to_string())
                    })?;
                    list.push(f32::from_le_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid f32 list alignment".to_string(),
                    ));
                }
            }
        }
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
        let slice = &$bytes[cursor..cursor + byte_count];
        let mut list: Vec<F> = Vec::with_capacity(size);
        match $encode {
            0 => {
                let mut iter = slice.chunks_exact(8);
                for element in &mut iter {
                    let element_bytes: [u8; 8] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid f64 list bytes".to_string())
                    })?;
                    list.push(f64::from_be_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid f64 list alignment".to_string(),
                    ));
                }
            }
            _ => {
                let mut iter = slice.chunks_exact(8);
                for element in &mut iter {
                    let element_bytes: [u8; 8] = element.try_into().map_err(|_| {
                        Error::DeserializationError("invalid f64 list bytes".to_string())
                    })?;
                    list.push(f64::from_le_bytes(element_bytes));
                }
                if !iter.remainder().is_empty() {
                    return Err(Error::DeserializationError(
                        "invalid f64 list alignment".to_string(),
                    ));
                }
            }
        }
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

    /// Decode a complete IPC message including the 8-byte header.
    ///
    /// This method is the counterpart to `ipc_msg_encode()`, handling:
    /// - Parsing the 8-byte IPC message header
    /// - Automatic decompression if the compressed flag is set
    /// - Decoding the payload into a K object
    ///
    /// # Arguments
    /// * `bytes` - Complete IPC message bytes including the 8-byte header
    ///
    /// # Returns
    /// A tuple of `(MessageHeader, K)` containing the parsed header and decoded K object
    ///
    /// # Errors
    /// Returns an error if:
    /// - The message is shorter than 8 bytes
    /// - The header is malformed
    /// - Decompression fails (if compressed)
    /// - Deserialization of the payload fails
    ///
    /// # Example
    /// ```
    /// use kdb_codec::qtype;
    /// use kdb_codec::K;
    /// use kdb_codec::qmsg_type;
    ///
    /// let original = K::new_long(42);
    /// let msg = original.ipc_msg_encode(qmsg_type::synchronous, false);
    ///
    /// let (header, decoded) = K::ipc_msg_decode(&msg).unwrap();
    /// assert_eq!(header.message_type, qmsg_type::synchronous);
    /// assert_eq!(header.compressed, 0);
    /// ```
    pub fn ipc_msg_decode(bytes: &[u8]) -> Result<(crate::codec::MessageHeader, K)> {
        use crate::codec::{decompress_sync, MessageHeader};

        // Parse the 8-byte header
        let header = MessageHeader::from_bytes(bytes)?;

        // Extract payload starting from byte 8
        if bytes.len() < MessageHeader::size() {
            return Err(Error::InvalidMessageSize);
        }

        let payload_bytes = &bytes[MessageHeader::size()..];

        // Handle compression
        let decoded_payload = if header.compressed == 1 {
            // Decompress: payload_bytes contains [uncompressed_size: 4 bytes][compressed_data]
            decompress_sync(payload_bytes.to_vec(), header.encoding, None)?
        } else {
            // Uncompressed: payload_bytes is the raw serialized K object
            payload_bytes.to_vec()
        };

        // Decode the K object from the payload
        let k = K::q_ipc_decode(&decoded_payload, header.encoding)?;

        Ok((header, k))
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
        qtype::LAMBDA => deserialize_lambda_sync(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::UNARY_PRIMITIVE => deserialize_unary_primitive_or_null(bytes, cursor + 1, encode),
        qtype::BINARY_PRIMITIVE => deserialize_fixed_payload_opaque(
            bytes,
            cursor + 1,
            encode,
            qtype::BINARY_PRIMITIVE,
            1,
        ),
        qtype::PROJECTION => deserialize_projection_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::COMPOSITION => deserialize_counted_or_fixed_arity_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
            qtype::COMPOSITION,
            2,
        ),
        qtype::EACH => deserialize_single_inner_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
            qtype::EACH,
        ),
        qtype::OVER => deserialize_over_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::SCAN => deserialize_scan_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::EACH_PRIOR => deserialize_single_inner_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
            qtype::EACH_PRIOR,
        ),
        qtype::EACH_LEFT => deserialize_single_inner_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
            qtype::EACH_LEFT,
        ),
        qtype::EACH_RIGHT => deserialize_each_right_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
        ),
        qtype::FOREIGN => deserialize_counted_or_fixed_arity_opaque(
            bytes,
            cursor + 1,
            encode,
            depth,
            max_list_size,
            max_recursion_depth,
            qtype::FOREIGN,
            3,
        ),
        qtype::ERROR => deserialize_error(bytes, cursor + 1, encode),
        _ => Err(Error::InvalidType(qtype)),
    }
}

fn deserialize_fixed_payload_opaque(
    bytes: &[u8],
    cursor: usize,
    _: u8,
    qtype: i8,
    payload_len: usize,
) -> Result<(K, usize)> {
    if cursor + payload_len > bytes.len() {
        return Err(Error::InsufficientData {
            needed: payload_len,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    let payload = bytes[cursor..cursor + payload_len].to_vec();
    Ok((K::new(qtype, qattribute::NONE, k0_inner::opaque(payload)), cursor + payload_len))
}

fn deserialize_unary_primitive_or_null(bytes: &[u8], cursor: usize, _: u8) -> Result<(K, usize)> {
    if cursor + 1 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: bytes.len().saturating_sub(cursor),
        });
    }
    let id = bytes[cursor];

    // (::) is encoded as unary primitive id 0.
    if id == 0x00 {
        return Ok((
            K::new(qtype::NULL, qattribute::NONE, k0_inner::null(())),
            cursor + 1,
        ));
    }

    Ok((
        K::new(
            qtype::UNARY_PRIMITIVE,
            qattribute::NONE,
            k0_inner::opaque(vec![id]),
        ),
        cursor + 1,
    ))
}

fn deserialize_projection_opaque(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    // Projection format (as observed from q -8!):
    //   byte 104, then i32 count N, then N serialized q objects.
    if cursor + 4 > bytes.len() {
        return Err(Error::InsufficientData {
            needed: 4,
            available: bytes.len().saturating_sub(cursor),
        });
    }

    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    let n_bytes: [u8; 4] = bytes[cursor..cursor + 4]
        .try_into()
        .map_err(|_| Error::DeserializationError("invalid projection count bytes".to_string()))?;
    let n = match encode {
        0 => i32::from_be_bytes(n_bytes),
        _ => i32::from_le_bytes(n_bytes),
    };
    if n < 0 {
        return Err(Error::DeserializationError(
            "invalid projection count (negative)".to_string(),
        ));
    }
    let n = n as usize;

    let start_payload = cursor;
    let mut next = cursor + 4;
    for _ in 0..n {
        let (_k, new_cursor) = deserialize_bytes_sync(
            bytes,
            next,
            encode,
            depth + 1,
            max_list_size,
            max_recursion_depth,
        )?;
        next = new_cursor;
    }

    let payload = bytes[start_payload..next].to_vec();
    Ok((
        K::new(qtype::PROJECTION, qattribute::NONE, k0_inner::opaque(payload)),
        next,
    ))
}

fn deserialize_counted_or_fixed_arity_opaque(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
    outer_qtype: i8,
    fallback_arity: usize,
) -> Result<(K, usize)> {
    // Heuristic decoder for opaque function-ish types.
    //
    // Some q objects (e.g. projections) encode as:
    //   <type byte> <i32 count N> <N serialized q objects>
    //
    // Other objects appear to encode as a fixed number of serialized q objects without the count.
    // We attempt the counted form first (if the count looks plausible), otherwise fall back to
    // reading `fallback_arity` serialized q objects.
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    // Attempt counted form.
    if cursor + 4 <= bytes.len() {
        let n_bytes: [u8; 4] = bytes[cursor..cursor + 4]
            .try_into()
            .map_err(|_| Error::DeserializationError("invalid count bytes".to_string()))?;
        let n = match encode {
            0 => i32::from_be_bytes(n_bytes),
            _ => i32::from_le_bytes(n_bytes),
        };

        if n >= 0 {
            let n_usize = n as usize;
            if n_usize <= max_list_size {
                let start_payload = cursor;
                let mut next = cursor + 4;
                let mut ok = true;
                for _ in 0..n_usize {
                    match deserialize_bytes_sync(
                        bytes,
                        next,
                        encode,
                        depth + 1,
                        max_list_size,
                        max_recursion_depth,
                    ) {
                        Ok((_k, new_cursor)) => next = new_cursor,
                        Err(_) => {
                            ok = false;
                            break;
                        }
                    }
                }

                if ok {
                    let payload = bytes[start_payload..next].to_vec();
                    return Ok((
                        K::new(outer_qtype, qattribute::NONE, k0_inner::opaque(payload)),
                        next,
                    ));
                }
            }
        }
    }

    // Fall back to fixed-arity form.
    let start_payload = cursor;
    let mut next = cursor;
    for _ in 0..fallback_arity {
        let (_k, new_cursor) = deserialize_bytes_sync(
            bytes,
            next,
            encode,
            depth + 1,
            max_list_size,
            max_recursion_depth,
        )?;
        next = new_cursor;
    }
    let payload = bytes[start_payload..next].to_vec();
    Ok((
        K::new(outer_qtype, qattribute::NONE, k0_inner::opaque(payload)),
        next,
    ))
}

fn deserialize_over_opaque(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    // Observed from q `-8!`:
    //   107 (0x6b) followed by exactly one serialized q object (typically a function).
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    let start_payload = cursor;
    let (_inner, next) = deserialize_bytes_sync(
        bytes,
        cursor,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;

    let payload = bytes[start_payload..next].to_vec();
    Ok((K::new(qtype::OVER, qattribute::NONE, k0_inner::opaque(payload)), next))
}

fn deserialize_scan_opaque(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    // Observed from q `-8!`:
    //   108 (0x6c) then a 1-byte adverb indicator (often also 0x6c), then one serialized q object.
    if cursor >= bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: 0,
        });
    }
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    let start_payload = cursor;
    let after_adverb = cursor + 1;
    let (_inner, next) = deserialize_bytes_sync(
        bytes,
        after_adverb,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;

    let payload = bytes[start_payload..next].to_vec();
    Ok((K::new(qtype::SCAN, qattribute::NONE, k0_inner::opaque(payload)), next))
}

fn deserialize_single_inner_opaque(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
    outer_qtype: i8,
) -> Result<(K, usize)> {
    // Derived-function format (as observed from q `-8!` for EACH/EACH_PRIOR/EACH_LEFT/OVER):
    //   outer type byte, then exactly one serialized q object (typically a function).
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    let start_payload = cursor;
    let (_inner, next) = deserialize_bytes_sync(
        bytes,
        cursor,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;

    let payload = bytes[start_payload..next].to_vec();
    Ok((
        K::new(outer_qtype, qattribute::NONE, k0_inner::opaque(payload)),
        next,
    ))
}

fn deserialize_each_right_opaque(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    // Observed from q `-8!` for `+\\:`:
    //   111 (0x6f) then a 1-byte marker (observed 0x6c), then one serialized q object.
    if cursor >= bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: 0,
        });
    }
    if depth > max_recursion_depth {
        return Err(Error::MaxDepthExceeded {
            depth,
            max: max_recursion_depth,
        });
    }

    let start_payload = cursor;
    let after_marker = cursor + 1;
    let (_inner, next) = deserialize_bytes_sync(
        bytes,
        after_marker,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;

    let payload = bytes[start_payload..next].to_vec();
    Ok((
        K::new(qtype::EACH_RIGHT, qattribute::NONE, k0_inner::opaque(payload)),
        next,
    ))
}

fn deserialize_lambda_sync(
    bytes: &[u8],
    cursor: usize,
    encode: u8,
    depth: usize,
    max_list_size: usize,
    max_recursion_depth: usize,
) -> Result<(K, usize)> {
    // Context: null-terminated string
    if cursor >= bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: 0,
        });
    }

    let mut idx = cursor;
    while idx < bytes.len() && bytes[idx] != 0x00 {
        idx += 1;
    }
    if idx >= bytes.len() {
        return Err(Error::InsufficientData {
            needed: 1,
            available: 0,
        });
    }

    let context = String::from_utf8_lossy(&bytes[cursor..idx]).to_string();
    idx += 1; // skip null terminator

    // Body: a char vector (type 10)
    let (body_k, next_cursor) = deserialize_bytes_sync(
        bytes,
        idx,
        encode,
        depth + 1,
        max_list_size,
        max_recursion_depth,
    )?;

    if body_k.get_type() != qtype::STRING {
        return Err(Error::DeserializationError(
            "invalid lambda body (expected char vector)".to_string(),
        ));
    }
    let body = body_k.as_string()?.to_string();

    Ok((K::new_lambda(context, body), next_cursor))
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
    let guid: [u8; 16] = bytes[cursor..cursor + 16]
        .try_into()
        .map_err(|_| Error::DeserializationError("invalid guid bytes".to_string()))?;
    Ok((K::new_guid(guid), cursor + 16))
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

    let size_bytes: [u8; 4] = bytes[cursor + 1..cursor + 5]
        .try_into()
        .map_err(|_| Error::DeserializationError("invalid list size bytes".to_string()))?;
    let size_u32 = match encode {
        0 => u32::from_be_bytes(size_bytes),
        _ => u32::from_le_bytes(size_bytes),
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
    let slice = &bytes[cursor..cursor + byte_count];
    let mut list: Vec<U> = Vec::with_capacity(size);
    let mut iter = slice.chunks_exact(16);
    for guid_bytes in &mut iter {
        let guid: [u8; 16] = guid_bytes
            .try_into()
            .map_err(|_| Error::DeserializationError("invalid guid list bytes".to_string()))?;
        list.push(guid);
    }
    if !iter.remainder().is_empty() {
        return Err(Error::DeserializationError(
            "invalid guid list alignment".to_string(),
        ));
    }
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
    // Each symbol requires at least 1 byte (null terminator). If the input can't possibly
    // contain `size` symbols, fail early before attempting large allocations.
    let remaining = bytes.len().saturating_sub(cursor);
    if size > remaining {
        return Err(Error::InsufficientData {
            needed: size,
            available: remaining,
        });
    }
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
    // Each nested element requires at least 1 byte (its qtype). If the input can't possibly
    // contain `size` elements, fail early before attempting large allocations.
    let remaining = bytes.len().saturating_sub(cursor);
    if size > remaining {
        return Err(Error::InsufficientData {
            needed: size,
            available: remaining,
        });
    }
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

    // Read table attribute byte (e.g. `s#`)
    let attribute = bytes[cursor] as i8;
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
        K::new(qtype::TABLE, attribute, k0_inner::table(dictionary)),
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

fn deserialize_null(bytes: &[u8], cursor: usize, encode: u8) -> Result<(K, usize)> {
    // Kept for backwards compatibility: historically qtype::NULL(101) was treated as a dedicated
    // type, but on the wire it is actually a unary primitive with id 0.
    deserialize_unary_primitive_or_null(bytes, cursor, encode)
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
