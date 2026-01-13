//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Load Libraries
//++++++++++++++++++++++++++++++++++++++++++++++++++//

use super::*;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Global Variable
//++++++++++++++++++++++++++++++++++++++++++++++++++//

// %% System encoding %%//vvvvvvvvvvvvvvvvvvvvvvvvvvv/

/// Endian of OS used to serialize `K` object.
/// - 0: Big Endian
/// - 1: Little Endian
#[cfg(target_endian = "big")]
pub const ENCODING: u8 = 0;

/// Endian of OS used to serialize `K` object.
/// - 0: Big Endian
/// - 1: Little Endian
#[cfg(target_endian = "little")]
pub const ENCODING: u8 = 1;

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Implementation
//++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% K %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

impl K {
    /// Serialize q object to bytes in a manner of q function `-8!` without the IPC message
    ///  header (encoding, message type, compressed, reserved null byte and total message length).
    pub fn q_ipc_encode(&self) -> Vec<u8> {
        let mut stream = Vec::new();
        serialize_q(self, &mut stream);
        stream
    }

    /// Serialize q object to complete IPC message bytes including the 8-byte IPC message header,
    /// optionally attempting kdb+ IPC compression.
    ///
    /// When `compress` is true, this will attempt to compress the message using the kdb+ IPC
    /// compression algorithm (equivalent to q `-18!`). If compression does not reduce the message
    /// to less than half its original size, the uncompressed message is returned.
    pub fn ipc_msg_encode(&self, msg_type: u8, compress: bool) -> Vec<u8> {
        let payload_bytes = self.q_ipc_encode();
        let message_length = payload_bytes.len();
        let total_length = (MessageHeader::size() + message_length) as u32;

        if compress {
            // Prepare raw message with placeholder header and payload
            let mut raw = Vec::with_capacity(MessageHeader::size() + message_length);
            raw.extend_from_slice(&[ENCODING, msg_type, 0, 0, 0, 0, 0, 0]);
            raw.extend_from_slice(&payload_bytes);

            // Try to compress
            let (was_compressed, mut bytes) = compress_sync(raw);
            if was_compressed {
                return bytes;
            }

            // Not compressed: write correct total length into header
            let total_length_bytes = match ENCODING {
                0 => total_length.to_be_bytes(),
                _ => total_length.to_le_bytes(),
            };
            bytes[4..8].copy_from_slice(&total_length_bytes);
            return bytes;
        }

        // Uncompressed message
        let header = MessageHeader {
            encoding: ENCODING,
            message_type: msg_type,
            compressed: 0,
            _unused: 0,
            length: total_length,
        };

        let mut out = Vec::with_capacity(MessageHeader::size() + message_length);
        out.extend_from_slice(&header.to_bytes());
        out.extend_from_slice(&payload_bytes);
        out
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::qmsg_type;

    fn read_u32(bytes: &[u8]) -> u32 {
        match ENCODING {
            0 => u32::from_be_bytes(bytes.try_into().unwrap()),
            _ => u32::from_le_bytes(bytes.try_into().unwrap()),
        }
    }

    #[test]
    fn ipc_msg_encode_uncompressed_has_valid_header_and_length() {
        let k = K::new_int(42);
        let payload = k.q_ipc_encode();
        let msg = k.ipc_msg_encode(qmsg_type::synchronous, false);

        assert_eq!(msg[0], ENCODING);
        assert_eq!(msg[1], qmsg_type::synchronous);
        assert_eq!(msg[2], 0);
        assert_eq!(msg[3], 0);

        let length = read_u32(&msg[4..8]);
        assert_eq!(length as usize, msg.len());
        assert_eq!(length as usize, MessageHeader::size() + payload.len());

        assert_eq!(&msg[MessageHeader::size()..], payload.as_slice());
    }

    #[test]
    fn ipc_msg_encode_with_compression_produces_compressed_frame_and_roundtrips() {
        // Highly compressible payload
        let k = K::new_byte_list(vec![0u8; 20_000], qattribute::NONE);
        let payload = k.q_ipc_encode();
        let msg = k.ipc_msg_encode(qmsg_type::synchronous, true);

        assert_eq!(msg[0], ENCODING);
        assert_eq!(msg[1], qmsg_type::synchronous);
        assert_eq!(msg[2], 1);
        assert_eq!(msg[3], 0);

        let compressed_total_len = read_u32(&msg[4..8]) as usize;
        assert_eq!(compressed_total_len, msg.len());

        let uncompressed_total_len = read_u32(&msg[8..12]) as usize;
        assert_eq!(uncompressed_total_len, MessageHeader::size() + payload.len());

        let decompressed_payload = decompress_sync(msg[8..].to_vec(), msg[0], None).unwrap();
        assert_eq!(decompressed_payload, payload);
    }

    #[test]
    fn ipc_msg_encode_with_compression_falls_back_to_uncompressed_when_not_worth_it() {
        // Pseudo-random-ish bytes should not compress to < half.
        let data: Vec<u8> = (0u32..5000)
            .map(|i| ((i.wrapping_mul(31).wrapping_add(7)) % 256) as u8)
            .collect();
        let k = K::new_byte_list(data, qattribute::NONE);
        let payload = k.q_ipc_encode();

        let msg = k.ipc_msg_encode(qmsg_type::synchronous, true);

        assert_eq!(msg[0], ENCODING);
        assert_eq!(msg[1], qmsg_type::synchronous);
        assert_eq!(msg[2], 0);
        assert_eq!(msg[3], 0);

        let total_len = read_u32(&msg[4..8]) as usize;
        assert_eq!(total_len, msg.len());
        assert_eq!(total_len, MessageHeader::size() + payload.len());
        assert_eq!(&msg[MessageHeader::size()..], payload.as_slice());
    }

    #[test]
    fn ipc_msg_decode_uncompressed_roundtrips() {
        let original = K::new_int(42);
        let msg = original.ipc_msg_encode(qmsg_type::synchronous, false);

        let (header, decoded) = K::ipc_msg_decode(&msg).unwrap();

        assert_eq!(header.encoding, ENCODING);
        assert_eq!(header.message_type, qmsg_type::synchronous);
        assert_eq!(header.compressed, 0);
        assert_eq!(header.length as usize, msg.len());

        assert_eq!(decoded.get_type(), qtype::INT_ATOM);
        assert_eq!(decoded.get_int().unwrap(), 42);
    }

    #[test]
    fn ipc_msg_decode_compressed_roundtrips() {
        // Highly compressible payload
        let original = K::new_byte_list(vec![0u8; 20_000], qattribute::NONE);
        let msg = original.ipc_msg_encode(qmsg_type::asynchronous, true);

        let (header, decoded) = K::ipc_msg_decode(&msg).unwrap();

        assert_eq!(header.encoding, ENCODING);
        assert_eq!(header.message_type, qmsg_type::asynchronous);
        assert_eq!(header.compressed, 1);

        assert_eq!(decoded.get_type(), qtype::BYTE_LIST);
        let decoded_list = decoded.as_vec::<u8>().unwrap();
        assert_eq!(decoded_list.len(), 20_000);
        assert!(decoded_list.iter().all(|&b| b == 0));
    }

    #[test]
    fn ipc_msg_decode_complex_object_roundtrips() {
        // Test with a symbol list
        let original = K::new_symbol_list(
            vec!["hello".to_string(), "world".to_string(), "kdb".to_string()],
            qattribute::NONE,
        );
        let msg = original.ipc_msg_encode(qmsg_type::response, false);

        let (header, decoded) = K::ipc_msg_decode(&msg).unwrap();

        assert_eq!(header.message_type, qmsg_type::response);
        assert_eq!(decoded.get_type(), qtype::SYMBOL_LIST);

        let decoded_list = decoded.as_vec::<String>().unwrap();
        assert_eq!(*decoded_list, vec!["hello".to_string(), "world".to_string(), "kdb".to_string()]);
    }

    #[test]
    fn ipc_msg_decode_fails_on_invalid_header() {
        let invalid_msg = vec![1, 2, 3]; // Too short for a header
        let result = K::ipc_msg_decode(&invalid_msg);
        assert!(result.is_err());
    }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++//
// >> Private Functions
//++++++++++++++++++++++++++++++++++++++++++++++++++//

fn serialize_q(obj: &K, stream: &mut Vec<u8>) {
    match obj.0.qtype {
        qtype::BOOL_ATOM | qtype::BYTE_ATOM | qtype::CHAR => serialize_byte(obj, stream),
        qtype::GUID_ATOM => serialize_guid(obj, stream),
        qtype::SHORT_ATOM => serialize_short(obj, stream),
        qtype::INT_ATOM
        | qtype::MONTH_ATOM
        | qtype::DATE_ATOM
        | qtype::MINUTE_ATOM
        | qtype::SECOND_ATOM
        | qtype::TIME_ATOM => serialize_int(obj, stream),
        qtype::LONG_ATOM | qtype::TIMESTAMP_ATOM | qtype::TIMESPAN_ATOM => {
            serialize_long(obj, stream)
        }
        qtype::REAL_ATOM => serialize_real(obj, stream),
        qtype::FLOAT_ATOM | qtype::DATETIME_ATOM => serialize_float(obj, stream),
        qtype::SYMBOL_ATOM => serialize_symbol(obj, stream),
        qtype::COMPOUND_LIST => serialize_compound_list(obj, stream),
        qtype::BOOL_LIST | qtype::BYTE_LIST => serialize_byte_list(obj, stream),
        qtype::GUID_LIST => serialize_guid_list(obj, stream),
        qtype::SHORT_LIST => serialize_short_list(obj, stream),
        qtype::INT_LIST
        | qtype::MONTH_LIST
        | qtype::DATE_LIST
        | qtype::MINUTE_LIST
        | qtype::SECOND_LIST
        | qtype::TIME_LIST => serialize_int_list(obj, stream),
        qtype::LONG_LIST | qtype::TIMESTAMP_LIST | qtype::TIMESPAN_LIST => {
            serialize_long_list(obj, stream)
        }
        qtype::REAL_LIST => serialize_real_list(obj, stream),
        qtype::FLOAT_LIST | qtype::DATETIME_LIST => serialize_float_list(obj, stream),
        qtype::STRING => serialize_string(obj, stream),
        qtype::SYMBOL_LIST => serialize_symbol_list(obj, stream),
        qtype::TABLE => serialize_table(obj, stream),
        qtype::DICTIONARY | qtype::SORTED_DICTIONARY => serialize_dictionary(obj, stream),
        qtype::LAMBDA => serialize_lambda(obj, stream),
        qtype::UNARY_PRIMITIVE => serialize_unary_primitive_or_null(obj, stream),
        qtype::BINARY_PRIMITIVE => serialize_opaque_payload_type(obj, stream),
        qtype::PROJECTION => serialize_opaque_payload_type(obj, stream),
        qtype::COMPOSITION => serialize_opaque_payload_type(obj, stream),
        qtype::EACH => serialize_opaque_payload_type(obj, stream),
        qtype::OVER => serialize_opaque_payload_type(obj, stream),
        qtype::SCAN => serialize_opaque_payload_type(obj, stream),
        qtype::EACH_PRIOR => serialize_opaque_payload_type(obj, stream),
        qtype::EACH_LEFT => serialize_opaque_payload_type(obj, stream),
        qtype::EACH_RIGHT => serialize_opaque_payload_type(obj, stream),
        qtype::FOREIGN => serialize_opaque_payload_type(obj, stream),
        _ => unimplemented!(),
    };
}

fn serialize_unary_primitive_or_null(obj: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(qtype::UNARY_PRIMITIVE as u8);

    // Data
    match &obj.0.value {
        k0_inner::null(()) => {
            // (::) encodes as unary primitive id 0
            stream.push(0x00);
        }
        k0_inner::opaque(payload) => {
            stream.extend_from_slice(payload);
        }
        _ => {
            // Preserve historical behavior: treat qtype 101 as null if caller constructed it
            // without the opaque payload.
            stream.push(0x00);
        }
    }
}

fn serialize_opaque_payload_type(obj: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(obj.0.qtype as u8);

    // Data
    if let k0_inner::opaque(payload) = &obj.0.value {
        stream.extend_from_slice(payload);
    } else {
        // No payload stored; encode as just the type byte.
        // This is roundtrip-unsafe but avoids panicking.
    }
}

fn serialize_lambda(lambda: &K, stream: &mut Vec<u8>) {
    let (context, body) = lambda.as_lambda().unwrap();

    // Type
    stream.push(qtype::LAMBDA as u8);

    // Context: null terminated string ("" for root)
    stream.extend_from_slice(context.as_bytes());
    stream.push(0x00);

    // Body: char vector (type 10)
    stream.push(qtype::STRING as u8);
    stream.push(qattribute::NONE as u8);

    let bytes = body.as_bytes();
    let length = match ENCODING {
        0 => (bytes.len() as u32).to_be_bytes(),
        _ => (bytes.len() as u32).to_le_bytes(),
    };
    stream.extend_from_slice(&length);
    stream.extend_from_slice(bytes);
}

fn serialize_guid(guid: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0xfe);
    // Element
    stream.extend_from_slice(&guid.get_guid().unwrap());
}

fn serialize_byte(byte: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(byte.0.qtype as u8);
    // Element
    stream.push(byte.get_byte().unwrap());
}

fn serialize_short(short: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0xfb);
    // Element
    stream.extend_from_slice(&match ENCODING {
        0 => short.get_short().unwrap().to_be_bytes(),
        _ => short.get_short().unwrap().to_le_bytes(),
    });
}

fn serialize_int(int: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(int.0.qtype as u8);
    // Element
    stream.extend_from_slice(&match ENCODING {
        0 => int.get_int().unwrap().to_be_bytes(),
        _ => int.get_int().unwrap().to_le_bytes(),
    });
}

fn serialize_long(long: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(long.0.qtype as u8);
    // Element
    stream.extend_from_slice(&match ENCODING {
        0 => long.get_long().unwrap().to_be_bytes(),
        _ => long.get_long().unwrap().to_le_bytes(),
    });
}

fn serialize_real(real: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0xf8);
    // Element
    stream.extend_from_slice(&match ENCODING {
        0 => real.get_real().unwrap().to_be_bytes(),
        _ => real.get_real().unwrap().to_le_bytes(),
    });
}

fn serialize_float(float: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(float.0.qtype as u8);
    // Element
    stream.extend_from_slice(&match ENCODING {
        0 => float.get_float().unwrap().to_be_bytes(),
        _ => float.get_float().unwrap().to_le_bytes(),
    });
}

fn serialize_symbol(symbol: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0xf5);
    // Element
    stream.extend_from_slice(symbol.get_symbol().unwrap().as_bytes());
    // Null byte
    stream.push(0x00);
}

fn serialize_guid_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0x02);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<U>().unwrap();
    // Length of vector
    let length = match ENCODING {
        0 => (vector.len() as u32).to_be_bytes(),
        _ => (vector.len() as u32).to_le_bytes(),
    };
    stream.extend_from_slice(&length);
    vector
        .iter()
        .for_each(|element| stream.extend_from_slice(element));
}

fn serialize_byte_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(list.0.qtype as u8);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<G>().unwrap();
    // Length of vector
    let length = match ENCODING {
        0 => (vector.len() as u32).to_be_bytes(),
        _ => (vector.len() as u32).to_le_bytes(),
    };
    stream.extend_from_slice(&length);
    stream.extend_from_slice(vector.as_slice());
}

fn serialize_short_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0x05);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<H>().unwrap();
    match ENCODING {
        0 => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_be_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_be_bytes());
            });
        }
        _ => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_le_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_le_bytes());
            });
        }
    }
}

fn serialize_int_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(list.0.qtype as u8);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<I>().unwrap();
    match ENCODING {
        0 => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_be_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_be_bytes());
            });
        }
        _ => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_le_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_le_bytes());
            });
        }
    }
}

fn serialize_long_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(list.0.qtype as u8);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<J>().unwrap();
    match ENCODING {
        0 => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_be_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_be_bytes());
            });
        }
        _ => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_le_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_le_bytes());
            });
        }
    }
}

fn serialize_real_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0x08);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<E>().unwrap();
    match ENCODING {
        0 => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_be_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_be_bytes());
            });
        }
        _ => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_le_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_le_bytes());
            });
        }
    }
}

fn serialize_float_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(list.0.qtype as u8);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<F>().unwrap();
    match ENCODING {
        0 => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_be_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_be_bytes());
            });
        }
        _ => {
            // Length of vector
            stream.extend_from_slice(&(vector.len() as u32).to_le_bytes());
            // Data
            vector.iter().for_each(|element| {
                stream.extend_from_slice(&element.to_le_bytes());
            });
        }
    }
}

fn serialize_string(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0x0a);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_string().unwrap().as_bytes();
    // Length of vector
    stream.extend_from_slice(&match ENCODING {
        0 => (vector.len() as u32).to_be_bytes(),
        _ => (vector.len() as u32).to_le_bytes(),
    });
    // Data
    stream.extend_from_slice(&vector);
}

fn serialize_symbol_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0x0b);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<S>().unwrap();
    // Length of vector
    stream.extend_from_slice(&match ENCODING {
        0 => (vector.len() as u32).to_be_bytes(),
        _ => (vector.len() as u32).to_le_bytes(),
    });
    // Data
    vector.iter().for_each(|element| {
        stream.extend_from_slice(&element.as_bytes());
        stream.push(0x00);
    });
}

fn serialize_compound_list(list: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(list.0.qtype as u8);
    // Attribute
    stream.push(list.0.attribute as u8);
    // Length and data
    let vector = list.as_vec::<K>().unwrap();
    // Length and data
    stream.extend_from_slice(&match ENCODING {
        0 => (vector.len() as u32).to_be_bytes(),
        _ => (vector.len() as u32).to_le_bytes(),
    });
    // Data
    vector.iter().for_each(|element| {
        serialize_q(element, stream);
    });
}

fn serialize_table(table: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(0x62);
    // Attribute (e.g. `s#` for sorted table)
    stream.push(table.0.attribute as u8);
    // Dictionary qtype marker (99)
    stream.push(0x63);
    // Retrieve underying dictionary
    let vector = table.get_dictionary().unwrap().as_vec::<K>().unwrap();
    // Serialize keys
    serialize_symbol_list(&vector[0], stream);
    // Serialize values
    serialize_compound_list(&vector[1], stream);
}

fn serialize_dictionary(dictionary: &K, stream: &mut Vec<u8>) {
    // Type
    stream.push(dictionary.0.qtype as u8);
    // Data
    let vector = dictionary.as_vec::<K>().unwrap();
    // Serialize keys
    serialize_q(&vector[0], stream);
    // Serialize values
    serialize_q(&vector[1], stream);
}

fn serialize_null(stream: &mut Vec<u8>) {
    // Backwards-compatible helper for historical callers.
    stream.push(0x65);
    stream.push(0x00);
}
