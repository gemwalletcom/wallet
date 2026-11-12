use ed25519_dalek::{Signer, SigningKey};
use gem_encoding::{encode_base58, encode_base64_url};
use serde::Serialize;

use crate::crypto::KEY_LENGTH;

const DID_KEY_PREFIX: &str = "did:key:z";
const ED25519_MULTICODEC: [u8; 2] = [0xed, 0x01];
const JWT_HEADER: &str = r#"{"alg":"EdDSA","typ":"JWT"}"#;
const TOKEN_LIFETIME_SECONDS: u64 = 86_400;

#[derive(Serialize)]
struct Claims<'a> {
    iss: String,
    sub: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
}

pub fn client_id(public_key: &[u8; KEY_LENGTH]) -> String {
    format!("{DID_KEY_PREFIX}{}", encode_base58([ED25519_MULTICODEC.as_slice(), public_key].concat()))
}

pub fn auth_token(seed: &[u8; KEY_LENGTH], audience: &str, subject: &str, issued_at: u64) -> String {
    let signing_key = SigningKey::from_bytes(seed);
    let claims = Claims {
        iss: client_id(&signing_key.verifying_key().to_bytes()),
        sub: subject,
        aud: audience,
        iat: issued_at,
        exp: issued_at + TOKEN_LIFETIME_SECONDS,
    };
    let claims = serde_json::to_vec(&claims).unwrap_or_default();
    let signing_input = format!("{}.{}", encode_base64_url(JWT_HEADER.as_bytes()), encode_base64_url(&claims));
    let signature = signing_key.sign(signing_input.as_bytes());
    format!("{signing_input}.{}", encode_base64_url(&signature.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Verifier};
    use gem_encoding::{decode_base64_url, decode_hex};

    #[test]
    fn test_client_id() {
        let public_key: [u8; KEY_LENGTH] = decode_hex("884ab67f787b69e534bfdba8d5beb4e719700e90ac06317ed177d49e5a33be5a").unwrap().try_into().unwrap();

        assert_eq!(client_id(&public_key), "did:key:z6MkodHZwneVRShtaLf8JKYkxpDGp1vGZnpGmdBpX8M2exxH");
    }

    #[test]
    fn test_auth_token() {
        let seed = [7u8; KEY_LENGTH];
        let token = auth_token(&seed, "wss://relay.walletconnect.org", "c479fe5dc464e771e78b193d239a65b58d278cad1c34bfb0b5716e5bb514928e", 1_656_910_097);
        let parts: Vec<&str> = token.split('.').collect();
        let verifying_key = SigningKey::from_bytes(&seed).verifying_key();

        assert_eq!(decode_base64_url(parts[0]).unwrap(), JWT_HEADER.as_bytes());
        assert_eq!(
            String::from_utf8(decode_base64_url(parts[1]).unwrap()).unwrap(),
            format!(
                r#"{{"iss":"{}","sub":"c479fe5dc464e771e78b193d239a65b58d278cad1c34bfb0b5716e5bb514928e","aud":"wss://relay.walletconnect.org","iat":1656910097,"exp":1656996497}}"#,
                client_id(&verifying_key.to_bytes())
            )
        );
        let signature = Signature::from_slice(&decode_base64_url(parts[2]).unwrap()).unwrap();
        assert!(verifying_key.verify(format!("{}.{}", parts[0], parts[1]).as_bytes(), &signature).is_ok());
    }
}
