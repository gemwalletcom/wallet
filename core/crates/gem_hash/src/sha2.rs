use bitcoin_hashes::{Hash, hash160::Hash as Hash160};
use hmac::digest::InvalidLength;
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest as Sha2Digest, Sha256, Sha512, Sha512_256};
use zeroize::{Zeroize, Zeroizing};

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

pub fn hash160(bytes: &[u8]) -> [u8; 20] {
    Hash160::hash(bytes).to_byte_array()
}

pub fn sha512_256(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha512_256::new();
    hasher.update(bytes);
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

pub fn sha512_half(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha512::new();
    hasher.update(bytes);
    let result = hasher.finalize();

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result[..32]);
    hash
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Zeroizing<[u8; 32]>, InvalidLength> {
    let mut mac = HmacSha256::new_from_slice(key)?;
    mac.update(data);
    let mut output = mac.finalize().into_bytes();
    let mut hash = Zeroizing::new([0u8; 32]);
    hash.copy_from_slice(&output);
    output.zeroize();
    Ok(hash)
}

pub fn hmac_sha512(key: &[u8], data: &[u8]) -> Result<Zeroizing<[u8; 64]>, InvalidLength> {
    let mut mac = HmacSha512::new_from_slice(key)?;
    mac.update(data);
    let mut output = mac.finalize().into_bytes();
    let mut hash = Zeroizing::new([0u8; 64]);
    hash.copy_from_slice(&output);
    output.zeroize();
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha2_hashes() {
        assert_eq!(gem_encoding::encode_hex(sha256(b"hello")), "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
        assert_eq!(gem_encoding::encode_hex(hash160(b"hello")), "b6a9c8c230722b7c748331a8b450f05566dc7d0f");
        assert_eq!(gem_encoding::encode_hex(sha512_256(b"hello")), "e30d87cfa2a75db545eac4d61baf970366a8357c7f72fa95b52d0accb698f13a");
        assert_eq!(gem_encoding::encode_hex(sha512_half(b"hello")), "9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca7");
    }

    #[test]
    fn test_hmac_sha256() {
        assert_eq!(
            gem_encoding::encode_hex(hmac_sha256(b"key", b"The quick brown fox jumps over the lazy dog").unwrap().as_slice()),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn test_hmac_sha512() {
        assert_eq!(
            gem_encoding::encode_hex(hmac_sha512(b"key", b"The quick brown fox jumps over the lazy dog").unwrap().as_slice()),
            "b42af09057bac1e2d41708e48a902e09b5ff7f12ab428a4fe86653c73dd248fb82f948a549f7b791a5b41915ee4d1ec3935357e4e2317250d0372afa2ebeeb3a"
        );
    }
}
