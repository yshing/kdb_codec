//! # KDB+ Codec
//!
//! This module provides a tokio-util codec implementation for the kdb+ IPC protocol.
//! It implements the `Encoder` and `Decoder` traits to work with `Framed` streams,
//! providing a cleaner and more idiomatic async Rust interface.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::deserialize_sync::q_ipc_decode_sync;
use super::serialize::ENCODING;
use super::{Error, Result, K};
use bytes::{BufMut, BytesMut};
use std::convert::TryInto;
use std::io;
use tokio_util::codec::{Decoder, Encoder};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Constants
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Size of the kdb+ IPC message header in bytes
const HEADER_SIZE: usize = 8;

/// Compression threshold - messages larger than this may be compressed
const COMPRESSION_THRESHOLD: usize = 2000;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Enums
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Compression behavior for encoding messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMode {
    /// Automatically compress based on message size and connection type (default behavior)
    /// - Local connections: no compression
    /// - Remote connections: compress if message > 2000 bytes
    Auto,
    /// Always attempt to compress messages larger than 2000 bytes (respects kdb+ compression algorithm)
    Always,
    /// Never compress messages
    Never,
}

impl Default for CompressionMode {
    fn default() -> Self {
        CompressionMode::Auto
    }
}

/// Validation strictness for decoding messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Strict validation - reject invalid headers
    /// - compressed flag must be 0 or 1
    /// - message type must be 0, 1, or 2
    Strict,
    /// Lenient validation - accept potentially invalid headers
    /// - allows any compressed flag value
    /// - allows any message type value
    Lenient,
}

impl Default for ValidationMode {
    fn default() -> Self {
        ValidationMode::Strict
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Structs
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Header of q IPC data frame.
#[derive(Clone, Copy, Debug)]
pub struct MessageHeader {
    /// Encoding.
    /// - 0: Big Endian
    /// - 1: Little Endian
    pub encoding: u8,
    /// Message type. One of followings:
    /// - 0: Asynchronous
    /// - 1: Synchronous
    /// - 2: Response
    pub message_type: u8,
    /// Indicator of whether the message is compressed or not.
    /// - 0: Uncompressed
    /// - 1: Compressed
    pub compressed: u8,
    /// Reserved byte.
    pub _unused: u8,
    /// Total length of the message including header.
    pub length: u32,
}

impl MessageHeader {
    /// Size of the message header in bytes
    pub const fn size() -> usize {
        HEADER_SIZE
    }

    /// Parse a message header from bytes
    pub fn from_bytes(buf: &[u8]) -> Result<Self> {
        if buf.len() < HEADER_SIZE {
            return Err(Error::InvalidMessageSize);
        }

        let encoding = buf[0];
        let message_type = buf[1];
        let compressed = buf[2];
        let _unused = buf[3];

        let length = match encoding {
            0 => u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            _ => u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
        };

        Ok(MessageHeader {
            encoding,
            message_type,
            compressed,
            _unused,
            length,
        })
    }

    /// Serialize the header to bytes
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0] = self.encoding;
        bytes[1] = self.message_type;
        bytes[2] = self.compressed;
        bytes[3] = self._unused;

        let length_bytes = match self.encoding {
            0 => self.length.to_be_bytes(),
            _ => self.length.to_le_bytes(),
        };
        bytes[4..8].copy_from_slice(&length_bytes);

        bytes
    }
}

/// Kdb+ Protocol Codec
///
/// This codec handles encoding and decoding of kdb+ IPC messages.
/// It manages the message framing, compression, and serialization/deserialization
/// of K objects.
#[derive(Debug, Clone)]
pub struct KdbCodec {
    /// Whether the connection is local (affects compression in Auto mode)
    is_local: bool,
    /// Compression mode for encoding
    compression_mode: CompressionMode,
    /// Validation mode for decoding
    validation_mode: ValidationMode,
}

#[bon::bon]
impl KdbCodec {
    /// Create a new KdbCodec with default settings (Auto compression, Strict validation)
    ///
    /// # Parameters
    /// - `is_local`: Whether the connection is within the same host (affects compression in Auto mode)
    pub fn new(is_local: bool) -> Self {
        KdbCodec {
            is_local,
            compression_mode: CompressionMode::Auto,
            validation_mode: ValidationMode::Strict,
        }
    }

    /// Create a new KdbCodec with custom compression and validation modes
    ///
    /// # Parameters
    /// - `is_local`: Whether the connection is within the same host (affects compression in Auto mode)
    /// - `compression_mode`: How to handle message compression
    /// - `validation_mode`: How strictly to validate incoming messages
    ///
    /// # Example
    /// ```
    /// use kdb_codec::{KdbCodec, CompressionMode, ValidationMode};
    ///
    /// // Always compress, lenient validation
    /// let codec = KdbCodec::with_options(false, CompressionMode::Always, ValidationMode::Lenient);
    /// ```
    pub fn with_options(
        is_local: bool,
        compression_mode: CompressionMode,
        validation_mode: ValidationMode,
    ) -> Self {
        KdbCodec {
            is_local,
            compression_mode,
            validation_mode,
        }
    }

    /// Create a builder for KdbCodec with fluent API
    ///
    /// # Example
    /// ```
    /// use kdb_codec::{KdbCodec, CompressionMode, ValidationMode};
    ///
    /// // Using builder pattern
    /// let codec = KdbCodec::builder()
    ///     .is_local(false)
    ///     .compression_mode(CompressionMode::Always)
    ///     .validation_mode(ValidationMode::Lenient)
    ///     .build();
    /// ```
    #[builder]
    pub fn builder(
        #[builder(default = false)] is_local: bool,
        #[builder(default)] compression_mode: CompressionMode,
        #[builder(default)] validation_mode: ValidationMode,
    ) -> Self {
        KdbCodec {
            is_local,
            compression_mode,
            validation_mode,
        }
    }

    /// Set the compression mode
    pub fn set_compression_mode(&mut self, mode: CompressionMode) {
        self.compression_mode = mode;
    }

    /// Get the current compression mode
    pub fn compression_mode(&self) -> CompressionMode {
        self.compression_mode
    }

    /// Set the validation mode
    pub fn set_validation_mode(&mut self, mode: ValidationMode) {
        self.validation_mode = mode;
    }

    /// Get the current validation mode
    pub fn validation_mode(&self) -> ValidationMode {
        self.validation_mode
    }
}

/// Message type for encoding
#[derive(Debug, Clone)]
pub struct KdbMessage {
    /// The message type (async, sync, or response)
    pub message_type: u8,
    /// The K object payload
    pub payload: K,
}

impl KdbMessage {
    /// Create a new KdbMessage
    pub fn new(message_type: u8, payload: K) -> Self {
        KdbMessage {
            message_type,
            payload,
        }
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Encoder Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

impl Encoder<KdbMessage> for KdbCodec {
    type Error = io::Error;

    fn encode(&mut self, item: KdbMessage, dst: &mut BytesMut) -> io::Result<()> {
        // Serialize the K object to bytes
        let payload_bytes = item.payload.q_ipc_encode();
        let message_length = payload_bytes.len();
        let total_length = (HEADER_SIZE + message_length) as u32;

        // Determine if compression should be attempted based on compression mode
        let should_compress = match self.compression_mode {
            CompressionMode::Never => false,
            CompressionMode::Always => message_length > COMPRESSION_THRESHOLD - HEADER_SIZE,
            CompressionMode::Auto => {
                // Auto mode: compress if message is large and connection is not local
                message_length > COMPRESSION_THRESHOLD - HEADER_SIZE && !self.is_local
            }
        };

        if should_compress {
            // Prepare raw message with placeholder header and payload
            let mut raw = Vec::with_capacity(HEADER_SIZE + message_length);
            raw.extend_from_slice(&[ENCODING, item.message_type, 0, 0, 0, 0, 0, 0]);
            raw.extend_from_slice(&payload_bytes);

            // Try to compress
            match compress_sync(raw) {
                (true, compressed) => {
                    // Message was compressed successfully
                    dst.reserve(compressed.len());
                    dst.put_slice(&compressed);
                }
                (false, mut uncompressed) => {
                    // Message was not compressed (compressed size >= half of original)
                    // Write original total data size
                    let total_length_bytes = match ENCODING {
                        0 => total_length.to_be_bytes(),
                        _ => total_length.to_le_bytes(),
                    };
                    uncompressed[4..8].copy_from_slice(&total_length_bytes);
                    dst.reserve(uncompressed.len());
                    dst.put_slice(&uncompressed);
                }
            }
        } else {
            // Uncompressed message
            let header = MessageHeader {
                encoding: ENCODING,
                message_type: item.message_type,
                compressed: 0,
                _unused: 0,
                length: total_length,
            };

            dst.reserve(total_length as usize);
            dst.put_slice(&header.to_bytes());
            dst.put_slice(&payload_bytes);
        }

        Ok(())
    }
}

//+++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Decoder Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

impl Decoder for KdbCodec {
    type Item = KdbMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        // Need at least header to proceed
        if src.len() < HEADER_SIZE {
            // Not enough data yet
            return Ok(None);
        }

        // Parse the header
        let header = MessageHeader::from_bytes(&src[..HEADER_SIZE]).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid header: {}", e))
        })?;

        // Validate header fields if in strict mode
        if self.validation_mode == ValidationMode::Strict {
            // Validate compressed flag (must be 0 or 1)
            if header.compressed > 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid compressed flag: {}. Expected 0 (uncompressed) or 1 (compressed)",
                        header.compressed
                    ),
                ));
            }

            // Validate message type (must be 0, 1, or 2)
            if header.message_type > 2 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Invalid message type: {}. Expected 0 (async), 1 (sync), or 2 (response)",
                        header.message_type
                    ),
                ));
            }
        }

        // Check if we have the complete message
        let total_length = header.length as usize;
        if src.len() < total_length {
            // Reserve space for the rest of the message
            src.reserve(total_length - src.len());
            return Ok(None);
        }

        // We have a complete message, extract it
        let message_data = src.split_to(total_length);

        // Skip the header, get payload
        let payload_data = &message_data[HEADER_SIZE..];

        // Handle decompression if needed
        let decoded_payload = if header.compressed == 1 {
            // Decompress the payload
            decompress_sync(payload_data.to_vec(), header.encoding)
        } else {
            payload_data.to_vec()
        };

        // Deserialize the K object
        let k_object = q_ipc_decode_sync(&decoded_payload, header.encoding);

        Ok(Some(KdbMessage {
            message_type: header.message_type,
            payload: k_object,
        }))
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Helper Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Convert IO errors to our Error type
pub fn io_error_to_kdb_error(err: io::Error) -> Error {
    Error::NetworkError(err.to_string())
}

/// Compress body synchronously. The combination of serializing the data and compressing will result in
/// the same output as shown in the q language by using the -18! function e.g.
/// serializing 2000 bools set to true, then compressing, will have the same output as `-18!2000#1b`.
///
/// # Data Format Flow
///
/// **Input (`raw`):**
/// ```text
/// [Header: 8 bytes][Payload: N bytes]
///  ├─ byte 0: encoding (0=BE, 1=LE)
///  ├─ byte 1: message_type (0=async, 1=sync, 2=response)
///  ├─ byte 2: compressed (always 0 in input)
///  ├─ byte 3: reserved
///  └─ bytes 4-7: length (placeholder, not used)
/// ```
///
/// **Output (when compressed successfully):**
/// ```text
/// [Header: 8 bytes][Uncompressed Size: 4 bytes][Compressed Payload]
///  ├─ byte 0: encoding (copied from input)
///  ├─ byte 1: message_type (copied from input)
///  ├─ byte 2: compressed (set to 1)
///  ├─ byte 3: reserved (copied from input)
///  ├─ bytes 4-7: total compressed message size (including this 8-byte header)
///  ├─ bytes 8-11: original uncompressed size (including original 8-byte header)
///  └─ bytes 12+: compressed payload data
/// ```
///
/// **Output (when compression fails - compressed size >= half of original):**
/// Returns `(false, original_raw_with_corrected_length)` where bytes 4-7 contain the actual total size.
///
/// # Usage in Encoder/Decoder
///
/// **Encoder:** Creates `raw` and calls this function. If compression succeeds, writes entire output to network.
///
/// **Decoder:** Reads from network, parses header (bytes 0-7), then passes bytes 8+ to `decompress_sync()`.
///
/// # Parameters
/// - `raw`: Serialized message (including header).
///
/// # Returns
/// - `(bool, Vec<u8>)`: Tuple of (compressed successfully, resulting bytes)
///   - If compression reduces size to less than half: `(true, compressed_data)`
///   - If compression doesn't save enough space: `(false, original_data)`
///
/// # Note
/// This function implements the kdb+ IPC compression algorithm which has been tested
/// in production and is compatible with kdb+ -18! function.
pub fn compress_sync(raw: Vec<u8>) -> (bool, Vec<u8>) {
    let mut i = 0_u8;
    let mut f = 0_u8;
    let mut h0 = 0_usize;
    let mut h = 0_usize;
    let mut g: bool;
    let mut compressed: Vec<u8> = Vec::with_capacity((raw.len()) / 2);
    // Assure that vector is filled with 0
    compressed.resize((raw.len()) / 2, 0_u8);

    // Start index of compressed body
    // 12 bytes are reserved for the header + size of raw bytes
    let mut c = 12;
    let mut d = c;
    let e = compressed.len();
    let mut p = 0_usize;
    let mut q: usize;
    let mut r: usize;
    let mut s0 = 0_usize;

    // Body starts from index 8
    let mut s = 8_usize;
    let t = raw.len();
    let mut a = [0_i32; 256];

    // Copy encode, message type, compressed and reserved
    compressed[0..4].copy_from_slice(&raw[0..4]);
    // Set compressed flag
    compressed[2] = 1;

    // Write size of raw bytes including a header
    let raw_size = match ENCODING {
        0 => (t as u32).to_be_bytes(),
        _ => (t as u32).to_le_bytes(),
    };
    compressed[8..12].copy_from_slice(&raw_size);

    while s < t {
        if i == 0 {
            if d > e - 17 {
                // Early return when compressing to less than half failed
                return (false, raw);
            }
            i = 1;
            compressed[c] = f;
            c = d;
            d += 1;
            f = 0;
        }
        g = s > t - 3;
        if !g {
            h = (raw[s] ^ raw[s + 1]) as usize;
            p = a[h] as usize;
            g = (0 == p) || (0 != (raw[s] ^ raw[p]));
        }
        if 0 < s0 {
            a[h0] = s0 as i32;
            s0 = 0;
        }
        if g {
            h0 = h;
            s0 = s;
            compressed[d] = raw[s];
            d += 1;
            s += 1;
        } else {
            a[h] = s as i32;
            f |= i;
            p += 2;
            s += 2;
            r = s;
            q = if s + 255 > t { t } else { s + 255 };
            while (s < q) && (raw[p] == raw[s]) {
                s += 1;
                if s < q {
                    p += 1;
                }
            }
            compressed[d] = h as u8;
            d += 1;
            compressed[d] = (s - r) as u8;
            d += 1;
        }
        i = i.wrapping_mul(2);
    }
    compressed[c] = f;
    // Final compressed data size
    let compressed_size = match ENCODING {
        0 => (d as u32).to_be_bytes(),
        _ => (d as u32).to_le_bytes(),
    };
    compressed[4..8].copy_from_slice(&compressed_size);
    let _ = compressed.split_off(d);
    (true, compressed)
}

/// Decompress body synchronously. The combination of decompressing and deserializing the data
/// will result in the same output as shown in the q language by using the `-19!` function.
///
/// # Data Format Flow
///
/// **Input (`compressed`):**
/// ```text
/// [Uncompressed Size: 4 bytes][Compressed Payload]
///  ├─ bytes 0-3: original uncompressed size (including the original 8-byte header that was removed)
///  └─ bytes 4+: compressed payload data
/// ```
///
/// **Output:**
/// ```text
/// [Decompressed Payload: N bytes]
/// (The original 8-byte header is NOT included in the output)
/// ```
///
/// # Usage in Decoder
///
/// The Decoder:
/// 1. Reads complete message from network: `[Header: 8 bytes][Uncompressed Size: 4 bytes][Compressed Payload]`
/// 2. Parses and validates the header (bytes 0-7)
/// 3. Extracts payload data starting from byte 8: `[Uncompressed Size: 4 bytes][Compressed Payload]`
/// 4. Passes this payload data to `decompress_sync()`
/// 5. Receives decompressed payload (without header)
/// 6. Deserializes the payload into a K object
///
/// # Parameters
/// - `compressed`: Compressed serialized message (**header already removed**, starts with uncompressed size).
/// - `encoding`:
///   - `0`: Big Endian
///   - `1`: Little Endian.
///
/// # Panics
/// This function will panic if the compressed data is malformed. This includes:
/// - Size field less than 8 bytes
/// - Invalid format that doesn't match kdb+ compression structure
///
/// # Note
/// This function implements the kdb+ IPC compression algorithm which has been tested
/// in production. Future improvements could include returning Result for better error handling.
pub fn decompress_sync(compressed: Vec<u8>, encoding: u8) -> Vec<u8> {
    let mut n = 0;
    let mut r: usize;
    let mut f = 0_usize;

    // Header has already been removed.
    // Start index of decompressed bytes is 0
    let mut s = 0_usize;
    let mut p = s;
    let mut i = 0_usize;

    // Read the uncompressed size from the compressed data
    // Subtract 8 bytes from decoded bytes size as 8 bytes have already been taken as header
    let size_with_header = match encoding {
        0 => i32::from_be_bytes(
            compressed[0..4]
                .try_into()
                .expect("Invalid compressed data: header size field must be 4 bytes"),
        ),
        _ => i32::from_le_bytes(
            compressed[0..4]
                .try_into()
                .expect("Invalid compressed data: header size field must be 4 bytes"),
        ),
    };

    // Validate size is positive and reasonable
    if size_with_header < 8 {
        panic!(
            "Invalid compressed data: size {} is less than minimum header size",
            size_with_header
        );
    }

    let size = (size_with_header - 8) as usize;
    let mut decompressed: Vec<u8> = Vec::with_capacity(size);
    // Assure that vector is filled with 0
    decompressed.resize(size, 0_u8);

    // Start index of compressed body.
    // 8 bytes have already been removed as header
    let mut d = 4;
    let mut aa = [0_i32; 256];
    while s < decompressed.len() {
        if i == 0 {
            f = (0xff & compressed[d]) as usize;
            d += 1;
            i = 1;
        }
        if (f & i) != 0 {
            r = aa[(0xff & compressed[d]) as usize] as usize;
            d += 1;
            decompressed[s] = decompressed[r];
            s += 1;
            r += 1;
            decompressed[s] = decompressed[r];
            s += 1;
            r += 1;
            n = (0xff & compressed[d]) as usize;
            d += 1;
            for m in 0..n {
                decompressed[s + m] = decompressed[r + m];
            }
        } else {
            decompressed[s] = compressed[d];
            s += 1;
            d += 1;
        }
        while p < s - 1 {
            aa[((0xff & decompressed[p]) ^ (0xff & decompressed[p + 1])) as usize] = p as i32;
            p += 1;
        }
        if (f & i) != 0 {
            s += n;
            p = s;
        }
        i *= 2;
        if i == 256 {
            i = 0;
        }
    }
    decompressed
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Tests
//++++++++++++++++++++++++++++++++++++++++++++++++++//

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{k, qmsg_type};

    #[test]
    fn test_compress_decompress_roundtrip() {
        // Create a message with a large K object that should be compressed
        let large_list = k!(long: vec![1; 3000]);
        let message = KdbMessage::new(1, large_list); // synchronous message

        // Encode the message (this should trigger compression for non-local)
        let mut codec = KdbCodec::new(false); // not local, so compression enabled
        let mut buffer = BytesMut::new();
        codec.encode(message.clone(), &mut buffer).unwrap();

        // The buffer should contain a complete message
        assert!(buffer.len() > 0);

        // Decode the message
        let decoded = codec.decode(&mut buffer).unwrap();
        assert!(decoded.is_some());

        let response = decoded.unwrap();
        assert_eq!(response.message_type, 1);

        // Verify the decoded payload matches the original
        let decoded_list = response.payload.as_vec::<i64>().unwrap();
        assert_eq!(decoded_list.len(), 3000);
        assert_eq!(decoded_list[0], 1);
    }

    #[test]
    fn test_small_message_no_compression() {
        // Create a small message that should NOT be compressed
        let small_list = k!(long: vec![1, 2, 3, 4, 5]);
        let message = KdbMessage::new(1, small_list);

        // Encode the message
        let mut codec = KdbCodec::new(false); // not local
        let mut buffer = BytesMut::new();
        codec.encode(message.clone(), &mut buffer).unwrap();

        // Check the compression flag in the header (should be 0)
        assert_eq!(buffer[2], 0); // compressed flag at byte 2

        // Decode the message
        let decoded = codec.decode(&mut buffer).unwrap();
        assert!(decoded.is_some());

        let response = decoded.unwrap();
        let decoded_list = response.payload.as_vec::<i64>().unwrap();
        assert_eq!(*decoded_list, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_local_connection_no_compression() {
        // Create a large message with local connection
        let large_list = k!(long: vec![42; 3000]);
        let message = KdbMessage::new(1, large_list);

        // Encode with local connection (compression disabled)
        let mut codec = KdbCodec::new(true); // local connection
        let mut buffer = BytesMut::new();
        codec.encode(message.clone(), &mut buffer).unwrap();

        // Check the compression flag in the header (should be 0 even for large message)
        assert_eq!(buffer[2], 0); // compressed flag at byte 2

        // Decode the message
        let decoded = codec.decode(&mut buffer).unwrap();
        assert!(decoded.is_some());

        let response = decoded.unwrap();
        let decoded_list = response.payload.as_vec::<i64>().unwrap();
        assert_eq!(decoded_list.len(), 3000);
        assert_eq!(decoded_list[0], 42);
    }

    #[test]
    fn test_string_query_encoding() {
        // Test encoding a string query
        let mut codec = KdbCodec::new(true);
        let mut buffer = BytesMut::new();

        // Create a KdbMessage with a string query
        let query = k!(string: "1+1");
        let message = KdbMessage::new(qmsg_type::synchronous, query);

        // Encode the message
        codec.encode(message, &mut buffer).unwrap();

        // Check that we have a valid message
        assert!(buffer.len() > HEADER_SIZE);

        // Check message type
        assert_eq!(buffer[1], qmsg_type::synchronous);

        // Check that it's not compressed (string queries are typically small)
        assert_eq!(buffer[2], 0);
    }

    #[test]
    fn test_message_header_roundtrip() {
        // Test message header serialization/deserialization
        let header = MessageHeader {
            encoding: ENCODING,
            message_type: 1,
            compressed: 1,
            _unused: 0,
            length: 1234,
        };

        let bytes = header.to_bytes();
        let parsed = MessageHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.encoding, header.encoding);
        assert_eq!(parsed.message_type, header.message_type);
        assert_eq!(parsed.compressed, header.compressed);
        assert_eq!(parsed.length, header.length);
    }

    #[test]
    fn test_compress_decompress_direct() {
        // Test compress_sync and decompress_sync directly to validate the data format
        // Use large enough data to trigger compression

        // Create a raw message: header + payload (large enough to compress)
        let payload = vec![42u8; 2000]; // Repetitive data compresses well
        let mut raw = Vec::new();
        // Header: encoding, message_type, compressed=0, reserved, length (placeholder)
        raw.extend_from_slice(&[ENCODING, 1, 0, 0, 0, 0, 0, 0]);
        raw.extend_from_slice(&payload);

        let original_size = raw.len();

        // Compress it
        let (was_compressed, compressed_data) = compress_sync(raw.clone());

        println!("Original size: {}", original_size);
        println!("Compressed data size: {}", compressed_data.len());
        println!("Was compressed: {}", was_compressed);

        // Should be compressed (repetitive data compresses well)
        assert!(was_compressed, "Large repetitive data should compress");

        // Verify compressed format
        // Bytes 0-3: encoding, message_type, compressed=1, reserved
        assert_eq!(compressed_data[0], ENCODING);
        assert_eq!(compressed_data[1], 1); // message type
        assert_eq!(compressed_data[2], 1); // compressed flag

        // Bytes 4-7: total compressed size (including header)
        let compressed_size = match ENCODING {
            0 => u32::from_be_bytes([
                compressed_data[4],
                compressed_data[5],
                compressed_data[6],
                compressed_data[7],
            ]),
            _ => u32::from_le_bytes([
                compressed_data[4],
                compressed_data[5],
                compressed_data[6],
                compressed_data[7],
            ]),
        };
        assert_eq!(compressed_size as usize, compressed_data.len());

        // Bytes 8-11: original uncompressed size (including header)
        let uncompressed_size = match ENCODING {
            0 => u32::from_be_bytes([
                compressed_data[8],
                compressed_data[9],
                compressed_data[10],
                compressed_data[11],
            ]),
            _ => u32::from_le_bytes([
                compressed_data[8],
                compressed_data[9],
                compressed_data[10],
                compressed_data[11],
            ]),
        };
        assert_eq!(uncompressed_size as usize, original_size);

        // Now decompress - skip header (bytes 0-7) to simulate what Decoder does
        // This is the KEY insight: Decoder removes the header before calling decompress_sync
        let payload_data = &compressed_data[HEADER_SIZE..];
        let decompressed = decompress_sync(payload_data.to_vec(), ENCODING);

        // The decompressed data should match the original payload (without header)
        assert_eq!(
            decompressed, payload,
            "Decompressed payload should match original"
        );
    }

    #[test]
    fn test_compression_with_large_data() {
        // Test with data large enough to trigger compression
        let large_payload = vec![42u8; 3000];
        let mut raw = Vec::new();
        raw.extend_from_slice(&[ENCODING, 1, 0, 0, 0, 0, 0, 0]);
        raw.extend_from_slice(&large_payload);

        let original_size = raw.len();

        // Compress
        let (was_compressed, compressed_data) = compress_sync(raw);

        // Should be compressed (large data with repetition compresses well)
        assert!(was_compressed, "Large repetitive data should compress");

        // Compressed size should be less than original
        assert!(
            compressed_data.len() < original_size,
            "Compressed size {} should be less than original size {}",
            compressed_data.len(),
            original_size
        );

        // Decompress - skip the header as Decoder does
        let payload_data = &compressed_data[HEADER_SIZE..];
        let decompressed = decompress_sync(payload_data.to_vec(), ENCODING);

        // Should match original payload
        assert_eq!(decompressed, large_payload);
    }

    #[test]
    fn test_codec_with_compression_end_to_end() {
        // Full end-to-end test through the codec
        let large_list = k!(long: vec![123; 2500]);
        let message = KdbMessage::new(qmsg_type::synchronous, large_list.clone());

        // Encode (should compress for non-local connection)
        let mut codec = KdbCodec::new(false);
        let mut buffer = BytesMut::new();
        codec.encode(message, &mut buffer).unwrap();

        // Verify compression flag is set
        let header = MessageHeader::from_bytes(&buffer[..HEADER_SIZE]).unwrap();
        assert_eq!(header.compressed, 1, "Large message should be compressed");

        // Decode
        let decoded = codec.decode(&mut buffer).unwrap();
        assert!(decoded.is_some());

        let response = decoded.unwrap();
        assert_eq!(response.message_type, qmsg_type::synchronous);

        // Verify payload matches
        let decoded_list = response.payload.as_vec::<i64>().unwrap();
        assert_eq!(decoded_list.len(), 2500);
        assert_eq!(decoded_list[0], 123);
        assert_eq!(decoded_list[2499], 123);
    }

    #[test]
    fn test_compression_mode_never() {
        // Test that Never mode doesn't compress even large messages
        let large_list = k!(long: vec![42; 3000]);
        let message = KdbMessage::new(qmsg_type::synchronous, large_list);

        // Use Never mode
        let mut codec =
            KdbCodec::with_options(false, CompressionMode::Never, ValidationMode::Strict);
        let mut buffer = BytesMut::new();
        codec.encode(message, &mut buffer).unwrap();

        // Check compression flag should be 0
        let header = MessageHeader::from_bytes(&buffer[..HEADER_SIZE]).unwrap();
        assert_eq!(header.compressed, 0, "Never mode should not compress");
    }

    #[test]
    fn test_compression_mode_always() {
        // Test that Always mode compresses large messages even on local connections
        let large_list = k!(long: vec![42; 3000]);
        let message = KdbMessage::new(qmsg_type::synchronous, large_list);

        // Use Always mode with local connection
        let mut codec =
            KdbCodec::with_options(true, CompressionMode::Always, ValidationMode::Strict);
        let mut buffer = BytesMut::new();
        codec.encode(message, &mut buffer).unwrap();

        // Check compression flag should be 1 (if compression succeeded)
        let header = MessageHeader::from_bytes(&buffer[..HEADER_SIZE]).unwrap();
        assert_eq!(
            header.compressed, 1,
            "Always mode should compress even on local"
        );
    }

    #[test]
    fn test_compression_mode_auto_local() {
        // Test that Auto mode doesn't compress on local connections
        let large_list = k!(long: vec![42; 3000]);
        let message = KdbMessage::new(qmsg_type::synchronous, large_list);

        // Use Auto mode with local connection
        let mut codec = KdbCodec::with_options(true, CompressionMode::Auto, ValidationMode::Strict);
        let mut buffer = BytesMut::new();
        codec.encode(message, &mut buffer).unwrap();

        // Check compression flag should be 0
        let header = MessageHeader::from_bytes(&buffer[..HEADER_SIZE]).unwrap();
        assert_eq!(
            header.compressed, 0,
            "Auto mode should not compress local connections"
        );
    }

    #[test]
    fn test_compression_mode_auto_remote() {
        // Test that Auto mode compresses large messages on remote connections
        let large_list = k!(long: vec![42; 3000]);
        let message = KdbMessage::new(qmsg_type::synchronous, large_list);

        // Use Auto mode with remote connection
        let mut codec =
            KdbCodec::with_options(false, CompressionMode::Auto, ValidationMode::Strict);
        let mut buffer = BytesMut::new();
        codec.encode(message, &mut buffer).unwrap();

        // Check compression flag should be 1
        let header = MessageHeader::from_bytes(&buffer[..HEADER_SIZE]).unwrap();
        assert_eq!(
            header.compressed, 1,
            "Auto mode should compress remote large messages"
        );
    }

    #[test]
    fn test_validation_mode_strict_invalid_compressed() {
        // Test that strict mode rejects invalid compressed flag
        let mut codec =
            KdbCodec::with_options(false, CompressionMode::Never, ValidationMode::Strict);
        let mut buffer = BytesMut::new();

        // Create a message with invalid compressed flag (2)
        buffer.extend_from_slice(&[ENCODING, 1, 2, 0]); // compressed = 2 (invalid)
        buffer.extend_from_slice(&[20, 0, 0, 0]); // length = 20 (header + some data)
        buffer.extend_from_slice(&[0; 12]); // dummy payload

        // Try to decode - should fail
        let result = codec.decode(&mut buffer);
        assert!(
            result.is_err(),
            "Strict mode should reject invalid compressed flag"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Invalid compressed flag"),
            "Error message should mention compressed flag, got: {}",
            err
        );
    }

    #[test]
    fn test_validation_mode_strict_invalid_message_type() {
        // Test that strict mode rejects invalid message type
        let mut codec =
            KdbCodec::with_options(false, CompressionMode::Never, ValidationMode::Strict);
        let mut buffer = BytesMut::new();

        // Create a message with invalid message type (3)
        buffer.extend_from_slice(&[ENCODING, 3, 0, 0]); // message_type = 3 (invalid)
        buffer.extend_from_slice(&[20, 0, 0, 0]); // length = 20
        buffer.extend_from_slice(&[0; 12]); // dummy payload

        // Try to decode - should fail
        let result = codec.decode(&mut buffer);
        assert!(
            result.is_err(),
            "Strict mode should reject invalid message type"
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Invalid message type"),
            "Error message should mention message type, got: {}",
            err
        );
    }

    #[test]
    fn test_validation_mode_lenient_accepts_invalid() {
        // Test that lenient mode accepts invalid values
        let mut codec =
            KdbCodec::with_options(false, CompressionMode::Never, ValidationMode::Lenient);

        // Create a small valid K object for the payload
        let small_int = k!(int: 42);
        let payload_bytes = small_int.q_ipc_encode();
        let total_length = (HEADER_SIZE + payload_bytes.len()) as u32;

        let mut buffer = BytesMut::new();
        // Create a message with "invalid" values that lenient mode should accept
        buffer.extend_from_slice(&[ENCODING, 5, 3, 0]); // message_type = 5, compressed = 3 (both "invalid")

        // Add length
        let length_bytes = match ENCODING {
            0 => total_length.to_be_bytes(),
            _ => total_length.to_le_bytes(),
        };
        buffer.extend_from_slice(&length_bytes);
        buffer.extend_from_slice(&payload_bytes);

        // Try to decode - should succeed in lenient mode
        let result = codec.decode(&mut buffer);
        assert!(
            result.is_ok(),
            "Lenient mode should accept non-standard values"
        );
        assert!(
            result.unwrap().is_some(),
            "Should decode message successfully"
        );
    }

    #[test]
    fn test_codec_getters_setters() {
        // Test getting and setting modes
        let mut codec = KdbCodec::new(false);

        // Check defaults
        assert_eq!(codec.compression_mode(), CompressionMode::Auto);
        assert_eq!(codec.validation_mode(), ValidationMode::Strict);

        // Set new modes
        codec.set_compression_mode(CompressionMode::Always);
        codec.set_validation_mode(ValidationMode::Lenient);

        // Verify changes
        assert_eq!(codec.compression_mode(), CompressionMode::Always);
        assert_eq!(codec.validation_mode(), ValidationMode::Lenient);
    }

    #[test]
    fn test_compression_mode_small_message() {
        // Test that even Always mode doesn't compress very small messages
        let small_int = k!(int: 42);
        let message = KdbMessage::new(qmsg_type::synchronous, small_int);

        // Use Always mode
        let mut codec =
            KdbCodec::with_options(false, CompressionMode::Always, ValidationMode::Strict);
        let mut buffer = BytesMut::new();
        codec.encode(message, &mut buffer).unwrap();

        // Small messages should not be compressed (below threshold)
        let header = MessageHeader::from_bytes(&buffer[..HEADER_SIZE]).unwrap();
        assert_eq!(
            header.compressed, 0,
            "Small messages should not be compressed"
        );
    }

    #[test]
    fn test_codec_builder_pattern() {
        // Test builder pattern creates codec with correct settings
        let codec = KdbCodec::builder()
            .is_local(false)
            .compression_mode(CompressionMode::Always)
            .validation_mode(ValidationMode::Lenient)
            .build();

        assert_eq!(codec.compression_mode(), CompressionMode::Always);
        assert_eq!(codec.validation_mode(), ValidationMode::Lenient);
    }

    #[test]
    fn test_codec_builder_with_defaults() {
        // Test builder pattern with default values
        let codec = KdbCodec::builder().build();

        // Should use defaults
        assert_eq!(codec.compression_mode(), CompressionMode::Auto);
        assert_eq!(codec.validation_mode(), ValidationMode::Strict);
    }

    #[test]
    fn test_codec_builder_partial() {
        // Test builder pattern with only some values specified
        let codec = KdbCodec::builder()
            .compression_mode(CompressionMode::Never)
            .build();

        assert_eq!(codec.compression_mode(), CompressionMode::Never);
        assert_eq!(codec.validation_mode(), ValidationMode::Strict); // default
    }
}
