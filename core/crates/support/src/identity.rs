use gem_hash::sha2::{InvalidLength, hmac_sha256};
use primitives::hex;

pub fn identifier_hash(secret: &str, identifier: &str) -> Result<String, InvalidLength> {
    Ok(hex::encode(hmac_sha256(secret.as_bytes(), identifier.as_bytes())?.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_hash() {
        assert_eq!(identifier_hash("test_chatwoot_identity_secret", "device-1").unwrap(), "b0ebd99b8c54a05c673c35f704f8549ebbe67f76dc1bcfe7a3acd8e6f33ec850");
        assert_ne!(identifier_hash("other_secret", "device-1").unwrap(), identifier_hash("test_chatwoot_identity_secret", "device-1").unwrap());
    }
}
