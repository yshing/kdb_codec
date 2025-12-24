//! # KDB+ Codec
//!
//! This module provides a tokio-util codec implementation for the kdb+ IPC protocol.
//! It implements the `Encoder` and `Decoder` traits to work with `Framed` streams,
//! providing a cleaner and more idiomatic async Rust interface.

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::serialize::ENCODING;
use super::{qtype, Error, K, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Constants
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Size of the kdb+ IPC message header in bytes
const HEADER_SIZE: usize = 8;

/// Minimum message size (header only)
const MIN_MESSAGE_SIZE: usize = HEADER_SIZE;

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
        let should_compress = message_length > COMPRESSION_THRESHOLD - HEADER_SIZE && !self.is_local;

        if should_compress {
            // Reserve space for header + payload
            dst.reserve(HEADER_SIZE + message_length);

            // Create a temporary buffer with header placeholder and payload
            let mut temp_buf = BytesMut::with_capacity(HEADER_SIZE + message_length);
            temp_buf.put_bytes(0, HEADER_SIZE); // Placeholder for header
            temp_buf.put_slice(&payload_bytes);

            // Try to compress
            // Note: Compression is handled by the compress function which is async in the original code
            // For the codec pattern, we'll handle it synchronously here or mark it as uncompressed
            // In a production implementation, you might want to handle compression differently

            // For now, we'll write uncompressed (simplified)
            let header = MessageHeader {
                encoding: ENCODING,
                message_type: item.message_type,
                compressed: 0,
                _unused: 0,
                length: total_length,
            };

            dst.put_slice(&header.to_bytes());
            dst.put_slice(&payload_bytes);
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
            // Note: The original code has async decompress function
            // For codec, we need sync decompression
            // This is a simplified version - you may need to implement proper decompression
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Compressed messages not yet supported in codec",
            ));
        } else {
            payload_data.to_vec()
        };

        // Deserialize the K object
        // Note: The original deserialize is async, but for codec we need sync
        // This will require refactoring the deserialize module
        // For now, we'll return an error indicating this needs implementation
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Deserialization needs to be adapted for sync codec pattern",
        ))
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Helper Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Convert IO errors to our Error type
pub fn io_error_to_kdb_error(err: io::Error) -> Error {
    Error::NetworkError(err.to_string())
}
