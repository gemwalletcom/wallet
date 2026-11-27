use aws_lc_rs::hmac;
use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair, ParsedPublicKey};
use gem_encoding::{decode_base64_url, encode_base64_url};
use serde::{Deserialize, Serialize};

use crate::JwtError;

const UNCOMPRESSED_POINT_TAG: u8 = 0x04;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Algorithm {
    HS256,
    ES256,
}

#[derive(Deserialize)]
struct EllipticCurveJwk {
    x: String,
    y: String,
}

pub enum SigningKey {
    Hs256(Box<hmac::Key>),
    Es256(EcdsaKeyPair),
}

impl SigningKey {
    pub fn hs256(secret: &[u8]) -> Result<Self, JwtError> {
        hmac_key(secret).map(Self::Hs256)
    }

    pub fn es256(pkcs8: &[u8]) -> Result<Self, JwtError> {
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8).map(Self::Es256).map_err(|_| JwtError::InvalidKey)
    }

    pub fn sign(&self, message: &[u8]) -> Result<String, JwtError> {
        match self {
            Self::Hs256(key) => Ok(encode_base64_url(hmac::sign(key, message).as_ref())),
            Self::Es256(key) => {
                let signature = key.sign(&SystemRandom::new(), message).map_err(|_| JwtError::InvalidKey)?;
                Ok(encode_base64_url(signature.as_ref()))
            }
        }
    }

    pub(crate) fn algorithm(&self) -> Algorithm {
        match self {
            Self::Hs256(_) => Algorithm::HS256,
            Self::Es256(_) => Algorithm::ES256,
        }
    }
}

pub enum VerifyingKey {
    Hs256(Box<hmac::Key>),
    Es256(ParsedPublicKey),
}

impl VerifyingKey {
    pub fn hs256(secret: &[u8]) -> Result<Self, JwtError> {
        hmac_key(secret).map(Self::Hs256)
    }

    pub fn es256(jwk: &str) -> Result<Self, JwtError> {
        let jwk: EllipticCurveJwk = serde_json::from_slice(jwk.as_bytes()).map_err(|_| JwtError::InvalidKey)?;
        let x = decode_base64_url(&jwk.x).map_err(|_| JwtError::InvalidKey)?;
        let y = decode_base64_url(&jwk.y).map_err(|_| JwtError::InvalidKey)?;
        let point = [&[UNCOMPRESSED_POINT_TAG], x.as_slice(), y.as_slice()].concat();
        ParsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, point).map(Self::Es256).map_err(|_| JwtError::InvalidKey)
    }

    pub fn verify(&self, message: &[u8], signature: &str) -> bool {
        let Ok(signature) = decode_base64_url(signature) else {
            return false;
        };
        match self {
            Self::Hs256(key) => hmac::verify(key, message, &signature).is_ok(),
            Self::Es256(key) => key.verify_sig(message, &signature).is_ok(),
        }
    }

    pub(crate) fn algorithm(&self) -> Algorithm {
        match self {
            Self::Hs256(_) => Algorithm::HS256,
            Self::Es256(_) => Algorithm::ES256,
        }
    }
}

fn hmac_key(secret: &[u8]) -> Result<Box<hmac::Key>, JwtError> {
    if secret.is_empty() {
        return Err(JwtError::InvalidKey);
    }
    Ok(Box::new(hmac::Key::new(hmac::HMAC_SHA256, secret)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        assert_eq!(SigningKey::hs256(b"secret").unwrap().sign(b"header.payload").unwrap(), "Z6i9S6ZA2aIWif9sETeqAT86NRYSiUvBz2wzPI7aP0s");
        assert!(VerifyingKey::mock().verify(b"header.payload", &SigningKey::mock().sign(b"header.payload").unwrap()));
    }

    #[test]
    fn test_verify() {
        let hs256 = SigningKey::hs256(b"secret").unwrap().sign(b"header.payload").unwrap();
        let es256 = SigningKey::mock().sign(b"header.payload").unwrap();

        assert!(VerifyingKey::hs256(b"secret").unwrap().verify(b"header.payload", &hs256));
        assert!(!VerifyingKey::hs256(b"secret").unwrap().verify(b"header.other", &hs256));
        assert!(!VerifyingKey::hs256(b"other").unwrap().verify(b"header.payload", &hs256));
        assert!(!VerifyingKey::hs256(b"secret").unwrap().verify(b"header.payload", &es256));
        assert!(!VerifyingKey::mock().verify(b"header.other", &es256));
        assert!(!VerifyingKey::mock().verify(b"header.payload", &hs256));
        assert!(!VerifyingKey::mock().verify(b"header.payload", &format!("{es256}=")));
    }

    #[test]
    fn test_hs256() {
        assert_eq!(SigningKey::hs256(b"").err(), Some(JwtError::InvalidKey));
        assert_eq!(VerifyingKey::hs256(b"").err(), Some(JwtError::InvalidKey));
    }

    #[test]
    fn test_es256() {
        assert_eq!(SigningKey::es256(b"not a key").err(), Some(JwtError::InvalidKey));
        assert_eq!(VerifyingKey::es256(r#"{"kty":"EC","crv":"P-256"}"#).err(), Some(JwtError::InvalidKey));
        assert_eq!(VerifyingKey::es256(r#"{"kty":"EC","crv":"P-256","x":"AA","y":"AA"}"#).err(), Some(JwtError::InvalidKey));
    }
}
