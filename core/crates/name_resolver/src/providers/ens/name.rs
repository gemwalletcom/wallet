use alloy_primitives::{B256, keccak256};

const VARIATION_SELECTOR: char = '\u{fe0f}';

pub fn namehash(name: &str) -> B256 {
    if name.is_empty() {
        return B256::ZERO;
    }
    name.replace(VARIATION_SELECTOR, "")
        .rsplit('.')
        .fold(B256::ZERO, |node, label| keccak256([node.as_slice(), keccak256(label).as_slice()].concat()))
}

pub fn dns_encode(name: &str) -> Result<Vec<u8>, &'static str> {
    if name.is_empty() {
        return Ok(vec![0]);
    }
    let name = name.replace(VARIATION_SELECTOR, "");
    let mut encoded = Vec::with_capacity(name.len() + 2);
    for label in name.split('.') {
        if label.is_empty() {
            return Err("ENS name contains an empty label");
        }
        encoded.push(u8::try_from(label.len()).map_err(|_| "ENS name label exceeds 255 bytes")?);
        encoded.extend_from_slice(label.as_bytes());
    }
    encoded.push(0);
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use alloy_primitives::b256;

    use super::*;

    #[test]
    fn test_namehash() {
        assert_eq!(namehash(""), B256::ZERO);
        assert_eq!(namehash("eth"), b256!("0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae"));
        assert_eq!(namehash("foo.eth"), b256!("0xde9b09fd7c5f901e23a3f19fecc54828e9c848539801e86591bd9801b019f84f"));
        assert_eq!(namehash("❤\u{fe0f}.eth"), namehash("❤.eth"));
    }

    #[test]
    fn test_dns_encode() {
        assert_eq!(dns_encode(""), Ok(vec![0]));
        assert_eq!(dns_encode("vitalik.eth"), Ok(b"\x07vitalik\x03eth\x00".to_vec()));
        assert_eq!(dns_encode("a..eth"), Err("ENS name contains an empty label"));
        assert_eq!(dns_encode(&format!("{}.eth", "a".repeat(256))), Err("ENS name label exceeds 255 bytes"));
    }
}
