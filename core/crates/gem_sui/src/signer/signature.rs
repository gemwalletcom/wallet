use primitives::SignerError;
use signer::Ed25519KeyPair;

use crate::types::{UserSignature, personal_message_signing_digest};

pub const SUI_PERSONAL_MESSAGE_SIGNATURE_LEN: usize = UserSignature::ED25519_LENGTH;

pub fn sign_personal_message(message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
    let digest = personal_message_signing_digest(message).map_err(|error| SignerError::InvalidInput(error.to_string()))?;
    sign_digest(&digest, private_key)
}

pub fn sign_digest(digest: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    UserSignature::ed25519(&key_pair.sign(digest), &key_pair.public_key_bytes)
        .map(|signature| signature.to_base64())
        .map_err(|error| SignerError::InvalidInput(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::decode_base64;
    use primitives::hex;

    #[test]
    fn test_sui_sign_personal_message() {
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").expect("valid hex");
        let message = b"Hello, world!".to_vec();

        let signature_base64 = sign_personal_message(&message, &private_key).expect("signing succeeds");

        let signature_bytes = decode_base64(&signature_base64).unwrap();
        assert_eq!(signature_bytes.len(), SUI_PERSONAL_MESSAGE_SIGNATURE_LEN, "signature layout length");
        assert_eq!(signature_bytes[0], 0x00, "expected Ed25519 flag prefix");

        let expected_base64 = "ALmKZNcvdmYgYloqKMAq7eSw5neV1mSEKfZProHEh8Ddw+6aJvLpuViFqZCHqwKdCqtzN8a+7jIDQSxbvmt04QDTaUUhl8KlZIHl4tPovwPeI0n2emMVGVaCIgjCM0re4g==";
        assert_eq!(signature_base64, expected_base64);
    }

    #[test]
    fn test_sign_digest_rejects_invalid_key_length() {
        let result = sign_digest(&[0u8; 32], &[0u8; 16]);
        assert!(result.is_err());
    }
}
