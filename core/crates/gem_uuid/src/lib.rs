use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;

const HEX: &[u8; 16] = b"0123456789abcdef";
const HYPHENS: [usize; 4] = [8, 13, 18, 23];

pub fn v4() -> String {
    let state = RandomState::new();
    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&state.hash_one(0u8).to_le_bytes());
    bytes[8..].copy_from_slice(&state.hash_one(1u8).to_le_bytes());
    format(versioned(bytes, 4))
}

#[cfg(feature = "v5")]
pub fn v5(namespace: &[u8; 16], name: &[u8]) -> String {
    let hash = gem_hash::sha1::sha1(&[namespace.as_slice(), name].concat());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    format(versioned(bytes, 5))
}

pub fn version(value: &str) -> Option<u8> {
    let bytes = value.as_bytes();
    let canonical = bytes.len() == 36 && bytes.iter().enumerate().all(|(index, byte)| if HYPHENS.contains(&index) { *byte == b'-' } else { HEX.contains(byte) });
    if !canonical {
        return None;
    }
    HEX.iter().position(|digit| *digit == bytes[14]).map(|version| version as u8)
}

fn versioned(mut bytes: [u8; 16], version: u8) -> [u8; 16] {
    bytes[6] = (bytes[6] & 0x0f) | (version << 4);
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    bytes
}

fn format(bytes: [u8; 16]) -> String {
    let mut text = String::with_capacity(36);
    for (index, byte) in bytes.iter().enumerate() {
        if [4, 6, 8, 10].contains(&index) {
            text.push('-');
        }
        text.push(char::from(HEX[usize::from(byte >> 4)]));
        text.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_v4() {
        let ids: Vec<String> = (0..2_000).map(|_| v4()).collect();
        assert!(ids.iter().all(|id| version(id) == Some(4) && b"89ab".contains(&id.as_bytes()[19])));
        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
    }

    #[test]
    fn test_v5() {
        assert_eq!(v5(b"GemKeystoreV4\0\0\0", b"multicoin_0x5a8f70b44aFa00Cb70615D9c9CCb9A24933ED2D3"), "f32a9e95-4904-533b-95fe-ebbe6cfb7554");
        assert_eq!(
            v5(&[0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8], b"python.org"),
            "886313e1-3b8a-5372-9b90-0c9aee199e5d"
        );
    }

    #[test]
    fn test_version() {
        assert_eq!(version("550e8400-e29b-41d4-a716-446655440000"), Some(4));
        assert_eq!(version("550e8400-e29b-11d4-a716-446655440000"), Some(1));
        for invalid in [
            "",
            "550E8400-E29B-41D4-A716-446655440000",
            "550e8400e29b41d4a716446655440000",
            "{550e8400-e29b-41d4-a716-446655440000}",
            "550e8400-e29b-41d4-a716-44665544000g",
            "550e8400-e29b-41d4-a716-446655440000.json",
        ] {
            assert_eq!(version(invalid), None, "{invalid}");
        }
    }
}
