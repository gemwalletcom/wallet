use gem_encoding::encode_base64;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

fn generate_hmac_from_bytes(key_bytes: &[u8], message: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key_bytes).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    encode_base64(&mac.finalize().into_bytes())
}

pub fn generate_hmac_signature(secret_key: &str, message: &str) -> String {
    generate_hmac_from_bytes(secret_key.as_bytes(), message)
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
}
