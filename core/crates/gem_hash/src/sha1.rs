use bitcoin_hashes::{Hash, sha1::Hash as Sha1};

pub fn sha1(bytes: &[u8]) -> [u8; 20] {
    Sha1::hash(bytes).to_byte_array()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1() {
        assert_eq!(gem_encoding::encode_hex(sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(gem_encoding::encode_hex(sha1(b"abc")), "a9993e364706816aba3e25717850c26c9cd0d89d");
    }
}
