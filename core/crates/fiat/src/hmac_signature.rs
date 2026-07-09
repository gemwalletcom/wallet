use gem_encoding::encode_base64;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn new_hmac_sha256(secret_key: &str) -> HmacSha256 {
    HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size")
}

fn generate_hmac_bytes(secret_key: &str, message: &str) -> Vec<u8> {
    let mut mac = new_hmac_sha256(secret_key);
    mac.update(message.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

pub fn generate_hmac_signature(secret_key: &str, message: &str) -> String {
    encode_base64(&generate_hmac_bytes(secret_key, message))
}

pub fn generate_hmac_signature_hex(secret_key: &str, message: &str) -> String {
    hex::encode(generate_hmac_bytes(secret_key, message))
}

pub fn verify_hmac_signature_hex(secret_key: &str, message: &str, signature: &str) -> bool {
    let Ok(signature) = hex::decode(signature) else {
        return false;
    };

    let mut mac = new_hmac_sha256(secret_key);
    mac.update(message.as_bytes());
    mac.verify_slice(&signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hmac_signature() {
        let secret = "test_secret";
        let message = "test_message";
        let signature = generate_hmac_signature(secret, message);
        assert_eq!(signature, "ZaIJF7XWibQHwbbgx6qd5AIh78SB/+WPJIXFHYIqzs4=");
    }

    #[test]
    fn test_generate_hmac_signature_hex() {
        let secret = "test_secret";
        let message = "test_message";
        let signature = generate_hmac_signature_hex(secret, message);
        assert_eq!(signature, "65a20917b5d689b407c1b6e0c7aa9de40221efc481ffe58f2485c51d822acece");
        assert!(verify_hmac_signature_hex(secret, message, &signature));
        assert!(!verify_hmac_signature_hex(secret, "wrong_message", &signature));
    }
}
