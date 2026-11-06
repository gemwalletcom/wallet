use hmac::digest::InvalidLength;
use hmac::{Hmac, KeyInit, Mac};
use primitives::hex;
use sha2::Sha256;

pub fn identifier_hash(secret: &str, identifier: &str) -> Result<String, InvalidLength> {
    let mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?.chain_update(identifier);
    Ok(hex::encode(mac.finalize().into_bytes()))
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
