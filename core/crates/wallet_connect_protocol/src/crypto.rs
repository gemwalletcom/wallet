use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use curve25519_dalek::montgomery::MontgomeryPoint;
use gem_crypto::random;
use gem_encoding::{decode_hex, encode_hex};
use gem_hash::sha2::{hmac_sha256, sha256};
use zeroize::Zeroizing;

use crate::ProtocolError;

pub const KEY_LENGTH: usize = 32;
const NONCE_LENGTH: usize = 12;
const HKDF_FIRST_BLOCK: [u8; 1] = [1];

pub type Key = Zeroizing<[u8; KEY_LENGTH]>;

pub fn generate_key() -> Result<Key, ProtocolError> {
    random::bytes::<KEY_LENGTH>().map(Zeroizing::new).map_err(|_| ProtocolError::Random)
}

pub fn public_key(secret: &[u8; KEY_LENGTH]) -> [u8; KEY_LENGTH] {
    MontgomeryPoint::mul_base_clamped(*secret).to_bytes()
}

pub fn shared_key(secret: &[u8; KEY_LENGTH], peer_public_key: &[u8; KEY_LENGTH]) -> Result<Key, ProtocolError> {
    let shared_secret = Zeroizing::new(MontgomeryPoint(*peer_public_key).mul_clamped(*secret).to_bytes());
    if *shared_secret == [0u8; KEY_LENGTH] {
        return Err(ProtocolError::InvalidKey);
    }
    let pseudo_random_key = hmac_sha256(&[], shared_secret.as_slice()).map_err(|_| ProtocolError::InvalidKey)?;
    hmac_sha256(pseudo_random_key.as_slice(), &HKDF_FIRST_BLOCK).map_err(|_| ProtocolError::InvalidKey)
}

pub fn topic(key: &[u8]) -> String {
    encode_hex(sha256(key))
}

pub fn parse_key(value: &str) -> Result<Key, ProtocolError> {
    let bytes = Zeroizing::new(decode_hex(value).map_err(|_| ProtocolError::InvalidKey)?);
    let key: [u8; KEY_LENGTH] = bytes.as_slice().try_into().map_err(|_| ProtocolError::InvalidKey)?;
    Ok(Zeroizing::new(key))
}

pub fn seal(key: &[u8; KEY_LENGTH], plaintext: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let nonce = random::bytes::<NONCE_LENGTH>().map_err(|_| ProtocolError::Random)?;
    seal_with_nonce(key, &nonce, plaintext)
}

pub fn open(key: &[u8; KEY_LENGTH], sealed: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    if sealed.len() < NONCE_LENGTH {
        return Err(ProtocolError::InvalidEnvelope);
    }
    let (nonce, ciphertext) = sealed.split_at(NONCE_LENGTH);
    let nonce = Nonce::try_from(nonce).map_err(|_| ProtocolError::InvalidEnvelope)?;
    cipher(key).decrypt(&nonce, ciphertext).map_err(|_| ProtocolError::Decryption)
}

fn seal_with_nonce(key: &[u8; KEY_LENGTH], nonce: &[u8; NONCE_LENGTH], plaintext: &[u8]) -> Result<Vec<u8>, ProtocolError> {
    let ciphertext = cipher(key).encrypt(&Nonce::from(*nonce), plaintext).map_err(|_| ProtocolError::InvalidEnvelope)?;
    Ok([nonce.as_slice(), &ciphertext].concat())
}

fn cipher(key: &[u8; KEY_LENGTH]) -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new(key.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_PEER_PUBLIC_KEY, TEST_PEER_SECRET, TEST_PLAINTEXT, TEST_SELF_PUBLIC_KEY, TEST_SELF_SECRET, TEST_SYM_KEY, TEST_TOPIC, TEST_TYPE0_ENVELOPE, test_key};
    use gem_encoding::decode_base64;

    #[test]
    fn test_shared_key() {
        let self_secret = test_key(TEST_SELF_SECRET);
        let peer_secret = test_key(TEST_PEER_SECRET);

        assert_eq!(encode_hex(public_key(&self_secret)), TEST_SELF_PUBLIC_KEY);
        assert_eq!(encode_hex(public_key(&peer_secret)), TEST_PEER_PUBLIC_KEY);
        assert_eq!(encode_hex(shared_key(&self_secret, &public_key(&peer_secret)).unwrap().as_slice()), TEST_SYM_KEY);
        assert_eq!(encode_hex(shared_key(&peer_secret, &public_key(&self_secret)).unwrap().as_slice()), TEST_SYM_KEY, "both sides derive the same key");
        assert_eq!(shared_key(&self_secret, &[0u8; KEY_LENGTH]), Err(ProtocolError::InvalidKey), "a low-order peer key gives no shared secret");
    }

    #[test]
    fn test_topic() {
        assert_eq!(topic(test_key(TEST_SYM_KEY).as_slice()), TEST_TOPIC);
    }

    #[test]
    fn test_seal_and_open() {
        let key = test_key(TEST_SYM_KEY);
        let nonce = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let expected = decode_base64(TEST_TYPE0_ENVELOPE).unwrap()[1..].to_vec();

        assert_eq!(seal_with_nonce(&key, &nonce, TEST_PLAINTEXT.as_bytes()).unwrap(), expected);
        assert_eq!(open(&key, &expected).unwrap(), TEST_PLAINTEXT.as_bytes());
        assert_eq!(open(&key, &seal(&key, b"random nonce").unwrap()).unwrap(), b"random nonce");

        let mut tampered = expected.clone();
        tampered[20] ^= 1;
        assert_eq!(open(&key, &tampered), Err(ProtocolError::Decryption));
        assert_eq!(open(&test_key(TEST_SELF_SECRET), &expected), Err(ProtocolError::Decryption));
        assert_eq!(open(&key, &[0u8; 4]), Err(ProtocolError::InvalidEnvelope));
    }

    #[test]
    fn test_parse_key() {
        assert_eq!(parse_key(TEST_SYM_KEY).unwrap(), test_key(TEST_SYM_KEY));
        assert_eq!(parse_key("abcd"), Err(ProtocolError::InvalidKey));
        assert_eq!(parse_key("zz"), Err(ProtocolError::InvalidKey));
    }
}
