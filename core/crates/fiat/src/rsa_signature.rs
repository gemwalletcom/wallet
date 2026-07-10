use gem_encoding::{decode_base64, encode_base64};
use pem_rfc7468::decode_vec;
use ring::rand::SystemRandom;
use ring::signature::{RSA_PSS_2048_8192_SHA512, RSA_PSS_SHA512, RsaKeyPair, UnparsedPublicKey};

pub fn generate_rsa_pss_signature(private_key_base_64: &str, message: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let decoded = decode_base64(private_key_base_64)?;
    let (_, key_der) = decode_vec(&decoded)?;

    let key_pair = RsaKeyPair::from_pkcs8(&key_der)?;
    let mut signature = vec![0u8; key_pair.public().modulus_len()];
    let rng = SystemRandom::new();
    key_pair.sign(&RSA_PSS_SHA512, &rng, message.as_bytes(), &mut signature)?;

    Ok(encode_base64(&signature))
}

pub fn verify_rsa_pss_signature(public_key: &str, message: &str, signature_base64: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let (_, key_der) = decode_vec(public_key.as_bytes())?;
    let signature = decode_base64(signature_base64)?;
    let public_key = UnparsedPublicKey::new(&RSA_PSS_2048_8192_SHA512, key_der);

    Ok(public_key.verify(message.as_bytes(), &signature).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRIVATE_KEY: &str = include_str!("../testdata/paybis/rsa_test_private_key.pem");
    const TEST_PUBLIC_KEY: &str = include_str!("../testdata/paybis/rsa_test_public_key.pem");

    #[test]
    fn test_generate_rsa_pss_signature_invalid_key() {
        let result = generate_rsa_pss_signature("invalid_base64", r#"{"test":"data"}"#);

        assert!(result.is_err());
    }

    #[test]
    fn test_rsa_pss_signature() {
        let message = r#"{"test":"data"}"#;
        let private_key = encode_base64(TEST_PRIVATE_KEY.as_bytes());
        let signature = generate_rsa_pss_signature(&private_key, message).unwrap();

        assert!(verify_rsa_pss_signature(TEST_PUBLIC_KEY, message, &signature).unwrap());
        assert!(!verify_rsa_pss_signature(TEST_PUBLIC_KEY, r#"{"test":"tampered"}"#, &signature).unwrap());
    }
}
