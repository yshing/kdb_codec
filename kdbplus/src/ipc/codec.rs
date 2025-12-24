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
use super::{qtype, Error, K, Result};
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
    /// Whether the connection is local (affects compression)
    is_local: bool,
}

impl KdbCodec {
    /// Create a new KdbCodec
    ///
    /// # Parameters
    /// - `is_local`: Whether the connection is within the same host (affects compression)
    pub fn new(is_local: bool) -> Self {
        KdbCodec { is_local }
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

/// Response from decoder including message type and K object
#[derive(Debug)]
pub struct KdbResponse {
    /// The message type
    pub message_type: u8,
    /// The K object payload
    pub payload: K,
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

        // Determine if compression should be attempted
        // Compression is triggered when entire message size is more than 2000 bytes
        // and the connection is not local
        let should_compress = message_length > COMPRESSION_THRESHOLD - HEADER_SIZE && !self.is_local;

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

/// Special encoder for string messages (text queries)
impl Encoder<(&str, u8)> for KdbCodec {
    type Error = io::Error;

    fn encode(&mut self, item: (&str, u8), dst: &mut BytesMut) -> io::Result<()> {
        let (text, message_type) = item;
        let byte_message = text.as_bytes();
        let message_length = byte_message.len() as u32;
        // Header + string type indicator (1) + attribute (1) + length (4) + message
        let total_length = HEADER_SIZE as u32 + 6 + message_length;

        let header = MessageHeader {
            encoding: ENCODING,
            message_type,
            compressed: 0,
            _unused: 0,
            length: total_length,
        };

        dst.reserve(total_length as usize);
        dst.put_slice(&header.to_bytes());

        // String type and attribute
        dst.put_u8(qtype::STRING as u8);
        dst.put_u8(0); // attribute

        // Length of string
        let length_bytes = match ENCODING {
            0 => message_length.to_be_bytes(),
            _ => message_length.to_le_bytes(),
        };
        dst.put_slice(&length_bytes);

        // String content
        dst.put_slice(byte_message);

        Ok(())
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Decoder Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

impl Decoder for KdbCodec {
    type Item = KdbResponse;
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

        Ok(Some(KdbResponse {
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
/// # Parameter
/// - `raw`: Serialized message (including header).
/// # Returns
/// - `(bool, Vec<u8>)`: Tuple of (compressed successfully, resulting bytes)
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
/// # Parameter
/// - `compressed`: Compressed serialized message (header already removed, starts with uncompressed size).
/// - `encoding`:
///   - `0`: Big Endian
///   - `1`: Little Endian.
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
        0 => {
            i32::from_be_bytes(
                compressed[0..4]
                    .try_into()
                    .expect("Invalid compressed data: header size field must be 4 bytes"),
            )
        }
        _ => {
            i32::from_le_bytes(
                compressed[0..4]
                    .try_into()
                    .expect("Invalid compressed data: header size field must be 4 bytes"),
            )
        }
    };
    
    // Validate size is positive and reasonable
    if size_with_header < 8 {
        panic!("Invalid compressed data: size {} is less than minimum header size", size_with_header);
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
    use crate::ipc::{qattribute, qmsg_type};

    #[test]
    fn test_compress_decompress_roundtrip() {
        // Create a message with a large K object that should be compressed
        let large_list = K::new_long_list(vec![1; 3000], qattribute::NONE);
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
        let small_list = K::new_long_list(vec![1, 2, 3, 4, 5], qattribute::NONE);
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
        let large_list = K::new_long_list(vec![42; 3000], qattribute::NONE);
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

        // Encode a simple string query
        codec
            .encode(("1+1", qmsg_type::synchronous), &mut buffer)
            .unwrap();

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
}
