//! Tests for enum and foreign object serialization/deserialization round-trip

use kdb_codec::*;

#[test]
fn test_enum_atom_roundtrip() {
    // Create an enum atom with domain and value
    let value: i32 = 42;
    let domain = "sym";
    
    // Build the K object with domain
    let mut bytes = vec![qtype::ENUM_ATOM as u8];
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00);
    bytes.extend_from_slice(&value.to_le_bytes());
    
    // Decode
    let k = K::q_ipc_decode(&bytes, 1).unwrap();
    assert_eq!(k.get_type(), qtype::ENUM_ATOM);
    assert_eq!(k.get_int().unwrap(), value);
    
    // Encode back
    let encoded = k.q_ipc_encode();
    
    // Should match original
    assert_eq!(encoded, bytes);
    
    // Decode again to verify
    let k2 = K::q_ipc_decode(&encoded, 1).unwrap();
    assert_eq!(k2.get_int().unwrap(), value);
}

#[test]
fn test_enum_list_roundtrip() {
    // Create an enum list with domain and values
    let values = vec![1i32, 2, 3];
    let domain = "sym";
    
    let mut bytes = vec![
        qtype::ENUM_LIST as u8,
        qattribute::NONE as u8,
    ];
    bytes.extend_from_slice(&(values.len() as i32).to_le_bytes());
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00);
    for &v in &values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    
    // Decode
    let k = K::q_ipc_decode(&bytes, 1).unwrap();
    assert_eq!(k.get_type(), qtype::ENUM_LIST);
    let list = k.as_vec::<I>().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0], 1);
    assert_eq!(list[1], 2);
    assert_eq!(list[2], 3);
    
    // Encode back
    let encoded = k.q_ipc_encode();
    
    // Should match original
    assert_eq!(encoded, bytes);
    
    // Decode again to verify
    let k2 = K::q_ipc_decode(&encoded, 1).unwrap();
    let list2 = k2.as_vec::<I>().unwrap();
    assert_eq!(list2.len(), 3);
    assert_eq!(list2[0], 1);
    assert_eq!(list2[1], 2);
    assert_eq!(list2[2], 3);
}

#[test]
fn test_foreign_object_roundtrip() {
    // Create a foreign object with payload
    let payload = vec![0xAA, 0xBB, 0xCC, 0xDD];
    
    let mut bytes = vec![
        qtype::FOREIGN as u8,
        qattribute::NONE as u8,
    ];
    bytes.extend_from_slice(&(payload.len() as i32).to_le_bytes());
    bytes.extend_from_slice(&payload);
    
    // Decode
    let k = K::q_ipc_decode(&bytes, 1).unwrap();
    assert_eq!(k.get_type(), qtype::FOREIGN);
    let stored = k.as_vec::<G>().unwrap();
    assert_eq!(stored.as_slice(), payload.as_slice());
    
    // Encode back
    let encoded = k.q_ipc_encode();
    
    // Should match original
    assert_eq!(encoded, bytes);
    
    // Decode again to verify
    let k2 = K::q_ipc_decode(&encoded, 1).unwrap();
    let stored2 = k2.as_vec::<G>().unwrap();
    assert_eq!(stored2.as_slice(), payload.as_slice());
}

#[test]
fn test_enum_atom_with_empty_domain() {
    // Test enum atom with empty domain name
    let value: i32 = 100;
    let mut bytes = vec![qtype::ENUM_ATOM as u8];
    bytes.push(0x00); // Empty domain name (just null terminator)
    bytes.extend_from_slice(&value.to_le_bytes());
    
    let k = K::q_ipc_decode(&bytes, 1).unwrap();
    assert_eq!(k.get_int().unwrap(), value);
    
    // Encode back
    let encoded = k.q_ipc_encode();
    assert_eq!(encoded, bytes);
}

#[test]
fn test_enum_list_with_long_domain_name() {
    // Test enum list with a longer domain name
    let domain = "my_custom_enum_domain";
    let values = vec![5i32, 10];
    
    let mut bytes = vec![
        qtype::ENUM_LIST as u8,
        qattribute::UNIQUE as u8,
    ];
    bytes.extend_from_slice(&(values.len() as i32).to_le_bytes());
    bytes.extend_from_slice(domain.as_bytes());
    bytes.push(0x00);
    for &v in &values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    
    let k = K::q_ipc_decode(&bytes, 1).unwrap();
    assert_eq!(k.get_type(), qtype::ENUM_LIST);
    assert_eq!(k.get_attribute(), qattribute::UNIQUE);
    
    // Encode and verify
    let encoded = k.q_ipc_encode();
    assert_eq!(encoded, bytes);
}
