use gem_encoding::{decode_base64, encode_base64};

use crate::SuiError;

const SCHEME_ED25519: u8 = 0;
const ED25519_SIGNATURE_LENGTH: usize = 64;
const ED25519_PUBLIC_KEY_LENGTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserSignature(Vec<u8>);

impl UserSignature {
    pub const ED25519_LENGTH: usize = 1 + ED25519_SIGNATURE_LENGTH + ED25519_PUBLIC_KEY_LENGTH;

    pub fn ed25519(signature: &[u8], public_key: &[u8]) -> Result<Self, SuiError> {
        if signature.len() != ED25519_SIGNATURE_LENGTH {
            return Err(SuiError::invalid_input(format!("Expected {ED25519_SIGNATURE_LENGTH} byte ed25519 signature, got {}", signature.len())));
        }
        if public_key.len() != ED25519_PUBLIC_KEY_LENGTH {
            return Err(SuiError::invalid_input(format!("Expected {ED25519_PUBLIC_KEY_LENGTH} byte ed25519 public key, got {}", public_key.len())));
        }
        Ok(Self([&[SCHEME_ED25519], signature, public_key].concat()))
    }

    pub fn from_base64(value: &str) -> Result<Self, SuiError> {
        let bytes = decode_base64(value).map_err(SuiError::from_display)?;
        match bytes.first() {
            Some(&SCHEME_ED25519) if bytes.len() == Self::ED25519_LENGTH => Ok(Self(bytes)),
            Some(_) | None => Err(SuiError::invalid_input("Unsupported Sui signature")),
        }
    }

    pub fn scheme(&self) -> u8 {
        SCHEME_ED25519
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_base64(&self) -> String {
        encode_base64(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_signature_ed25519() {
        let signature = UserSignature::ed25519(&[1; ED25519_SIGNATURE_LENGTH], &[2; ED25519_PUBLIC_KEY_LENGTH]).unwrap();

        assert_eq!(signature.as_bytes(), [vec![SCHEME_ED25519], vec![1; ED25519_SIGNATURE_LENGTH], vec![2; ED25519_PUBLIC_KEY_LENGTH]].concat());
        assert_eq!(UserSignature::from_base64(&signature.to_base64()).unwrap(), signature);
        assert!(UserSignature::ed25519(&[1; ED25519_SIGNATURE_LENGTH - 1], &[2; ED25519_PUBLIC_KEY_LENGTH]).is_err());
        assert!(UserSignature::ed25519(&[1; ED25519_SIGNATURE_LENGTH], &[2; ED25519_PUBLIC_KEY_LENGTH + 1]).is_err());
        assert!(UserSignature::from_base64(&encode_base64(&[vec![1], vec![0; UserSignature::ED25519_LENGTH - 1]].concat())).is_err());
    }
}
