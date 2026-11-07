use gem_encoding::{decode_base64, encode_base64};

use crate::ProtocolError;
use crate::crypto::{KEY_LENGTH, open, seal};

const TYPE_SYMMETRIC: u8 = 0;
const TYPE_SENDER_KEY: u8 = 1;

pub fn encrypt(key: &[u8; KEY_LENGTH], plaintext: &[u8]) -> Result<String, ProtocolError> {
    Ok(encode_base64(&[[TYPE_SYMMETRIC].as_slice(), &seal(key, plaintext)?].concat()))
}

pub fn encrypt_with_sender_key(key: &[u8; KEY_LENGTH], sender_public_key: &[u8; KEY_LENGTH], plaintext: &[u8]) -> Result<String, ProtocolError> {
    Ok(encode_base64(&[[TYPE_SENDER_KEY].as_slice(), sender_public_key, &seal(key, plaintext)?].concat()))
}

pub fn decrypt(key: &[u8; KEY_LENGTH], message: &str) -> Result<Vec<u8>, ProtocolError> {
    let envelope = decode_base64(message).map_err(|_| ProtocolError::InvalidEnvelope)?;
    match envelope.split_first() {
        Some((&TYPE_SYMMETRIC, sealed)) => open(key, sealed),
        _ => Err(ProtocolError::InvalidEnvelope),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_PLAINTEXT, TEST_SELF_PUBLIC_KEY, TEST_SYM_KEY, TEST_TYPE0_ENVELOPE, TEST_TYPE1_ENVELOPE, test_key};

    #[test]
    fn test_decrypt() {
        let key = test_key(TEST_SYM_KEY);

        assert_eq!(decrypt(&key, TEST_TYPE0_ENVELOPE).unwrap(), TEST_PLAINTEXT.as_bytes());
        assert_eq!(decrypt(&key, TEST_TYPE1_ENVELOPE), Err(ProtocolError::InvalidEnvelope), "a wallet never receives sender-key envelopes");
        assert_eq!(decrypt(&key, ""), Err(ProtocolError::InvalidEnvelope));
        assert_eq!(decrypt(&key, "not base64!"), Err(ProtocolError::InvalidEnvelope));
        assert_eq!(decrypt(&key, "AQ=="), Err(ProtocolError::InvalidEnvelope));
    }

    #[test]
    fn test_encrypt() {
        let key = test_key(TEST_SYM_KEY);
        let sender = test_key(TEST_SELF_PUBLIC_KEY);

        assert_eq!(decrypt(&key, &encrypt(&key, b"payload").unwrap()).unwrap(), b"payload");

        let envelope = decode_base64(&encrypt_with_sender_key(&key, &sender, b"payload").unwrap()).unwrap();
        assert_eq!(envelope[0], TYPE_SENDER_KEY);
        assert_eq!(&envelope[1..33], sender.as_slice());
        assert_eq!(open(&key, &envelope[33..]).unwrap(), b"payload");
    }
}
