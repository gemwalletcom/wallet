use alloy_primitives::hex;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use gem_encoding::{decode_base64, encode_base64};
use zeroize::Zeroizing;

pub const GEM_AUTH_SCHEME: &str = "Gem ";

const ED25519_SEED_LENGTH: usize = 32;

pub fn device_body_hash(body: &[u8]) -> String {
    hex::encode(gem_hash::sha2::sha256(body))
}

pub fn device_auth_message(timestamp: &str, method: &str, path: &str, wallet_id: &str, body_hash: &str) -> String {
    format!("{timestamp}.{method}.{path}.{wallet_id}.{body_hash}")
}

pub fn device_public_key(private_key: &[u8]) -> Result<[u8; 32], &'static str> {
    let seed: Zeroizing<[u8; ED25519_SEED_LENGTH]> = Zeroizing::new(private_key.try_into().map_err(|_| "invalid device private key length")?);
    Ok(SigningKey::from_bytes(&seed).verifying_key().to_bytes())
}

pub fn build_device_auth_header(private_key: &[u8], method: &str, path: &str, wallet_id: &str, body: &[u8], timestamp_ms: u64) -> Result<String, &'static str> {
    let seed: Zeroizing<[u8; ED25519_SEED_LENGTH]> = Zeroizing::new(private_key.try_into().map_err(|_| "invalid device private key length")?);
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = hex::encode(signing_key.verifying_key().to_bytes());
    let body_hash = device_body_hash(body);
    let timestamp = timestamp_ms.to_string();
    let message = device_auth_message(&timestamp, method, path, wallet_id, &body_hash);
    let signature = hex::encode(signing_key.sign(message.as_bytes()).to_bytes());
    let payload = format!("{public_key}.{timestamp}.{wallet_id}.{body_hash}.{signature}");
    Ok(format!("{GEM_AUTH_SCHEME}{}", encode_base64(payload.as_bytes())))
}

pub fn verify_device_auth(header_value: &str, method: &str, path: &str, body: &[u8]) -> bool {
    let Some(payload) = parse_device_auth(header_value) else {
        return false;
    };
    if payload.body_hash != device_body_hash(body) {
        return false;
    }
    let message = device_auth_message(&payload.timestamp, method, path, payload.wallet_id.as_deref().unwrap_or(""), &payload.body_hash);
    verify_device_signature(&payload.device_id, &message, &payload.signature)
}

#[derive(Debug, PartialEq)]
pub enum AuthScheme {
    Gem,
    Legacy,
}

pub struct DeviceAuthPayload {
    pub scheme: AuthScheme,
    pub device_id: String,
    pub timestamp: String,
    pub wallet_id: Option<String>,
    pub body_hash: String,
    pub signature: Vec<u8>,
}

pub fn parse_device_auth(header_value: &str) -> Option<DeviceAuthPayload> {
    let encoded = header_value.strip_prefix(GEM_AUTH_SCHEME)?;
    let decoded = decode_base64(encoded).ok()?;
    let payload = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = payload.splitn(5, '.').collect();
    if parts.len() != 5 {
        return None;
    }
    Some(DeviceAuthPayload {
        scheme: AuthScheme::Gem,
        device_id: parts[0].to_string(),
        timestamp: parts[1].to_string(),
        wallet_id: if parts[2].is_empty() { None } else { Some(parts[2].to_string()) },
        body_hash: parts[3].to_string(),
        signature: hex::decode(parts[4]).ok()?,
    })
}

// TODO: remove base64 fallback once all clients use hex signatures
pub fn decode_signature(value: &str) -> Option<Vec<u8>> {
    hex::decode(value).ok().or_else(|| decode_base64(value).ok())
}

pub fn verify_device_signature(public_key_hex: &str, message: &str, signature: &[u8]) -> bool {
    let Ok(pk_bytes) = hex::decode(public_key_hex) else {
        return false;
    };
    let Ok(pk_array): Result<[u8; 32], _> = pk_bytes.try_into() else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pk_array) else {
        return false;
    };
    let Ok(sig_array): Result<[u8; 64], _> = signature.try_into() else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_array);
    verifying_key.verify_strict(message.as_bytes(), &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;
    use ed25519_dalek::{Signer, SigningKey};
    use gem_encoding::encode_base64;

    #[test]
    fn test_verify_valid_signature() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());

        assert!(verify_device_signature(&public_key_hex, message, &signature.to_bytes()));
    }

    #[test]
    fn test_reject_invalid_signature() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        assert!(!verify_device_signature(&public_key_hex, message, &[0u8; 64]));
    }

    #[test]
    fn test_reject_tampered_message() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());

        let tampered = "v1.1706000000000.POST./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(!verify_device_signature(&public_key_hex, tampered, &signature.to_bytes()));
    }

    #[test]
    fn test_reject_wrong_public_key() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let wrong_key = SigningKey::from_bytes(&[2u8; 32]);
        let wrong_public_key_hex = hex::encode(wrong_key.verifying_key().as_bytes());
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());

        assert!(!verify_device_signature(&wrong_public_key_hex, message, &signature.to_bytes()));
    }

    #[test]
    fn test_reject_invalid_signature_length() {
        assert!(!verify_device_signature("aabb", "msg", &[0u8; 2]));
    }

    #[test]
    fn test_parse_device_auth() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let timestamp = "1706000000000";
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let wallet_id = "multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let signature = signing_key.sign(b"test");
        let signature_hex = hex::encode(signature.to_bytes());

        let payload = format!("{}.{}.{}.{}.{}", public_key_hex, timestamp, wallet_id, body_hash, signature_hex);
        let encoded = encode_base64(payload.as_bytes());
        let header = format!("Gem {}", encoded);

        let result = parse_device_auth(&header).unwrap();
        assert_eq!(result.device_id, public_key_hex);
        assert_eq!(result.timestamp, timestamp);
        assert_eq!(result.wallet_id.as_deref(), Some(wallet_id));
        assert_eq!(result.body_hash, body_hash);
        assert_eq!(result.signature, signature.to_bytes());
    }

    #[test]
    fn test_parse_device_auth_invalid() {
        assert!(parse_device_auth("Bearer token").is_none());
        assert!(parse_device_auth("Gem !!!").is_none());
        let encoded = encode_base64(b"only.two.parts");
        assert!(parse_device_auth(&format!("Gem {}", encoded)).is_none());
    }

    #[test]
    fn test_parse_device_auth_empty_wallet_id() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let timestamp = "1706000000000";
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(b"test");
        let signature_hex = hex::encode(signature.to_bytes());

        let payload = format!("{}.{}..{}.{}", public_key_hex, timestamp, body_hash, signature_hex);
        let encoded = encode_base64(payload.as_bytes());
        let header = format!("Gem {}", encoded);

        let result = parse_device_auth(&header).unwrap();
        assert_eq!(result.device_id, public_key_hex);
        assert_eq!(result.timestamp, timestamp);
        assert_eq!(result.wallet_id, None);
        assert_eq!(result.body_hash, body_hash);
    }

    #[test]
    fn test_verify_signature_with_wallet_id() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let wallet_id = "multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let message = format!("1706000000000.GET./v1/devices/abc.{}.{}", wallet_id, body_hash);
        let signature = signing_key.sign(message.as_bytes());

        assert!(verify_device_signature(&public_key_hex, &message, &signature.to_bytes()));
    }

    #[test]
    fn test_verify_signature_empty_wallet_id() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let message = format!("1706000000000.GET./v1/devices/abc..{}", body_hash);
        let signature = signing_key.sign(message.as_bytes());

        assert!(verify_device_signature(&public_key_hex, &message, &signature.to_bytes()));
    }

    // RFC 8032 Ed25519 test vector (shared with the iOS/Android device-key fixtures).
    const FIXTURE_PRIVATE_KEY_HEX: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    const FIXTURE_PUBLIC_KEY_HEX: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    const FIXTURE_DEVICE_AUTH_SIGNATURE_HEX: &str =
        "121bb28074b00114a7b267ae8a6292c9ffe56db6254b0c65389fd726dbdeffe95b15e6f48d3a3980f7a983da44a3de24c0771d0d8723cef2ced6d08d343a2101";

    #[test]
    fn test_build_device_auth_header_parity() {
        let private_key = hex::decode(FIXTURE_PRIVATE_KEY_HEX).unwrap();

        assert_eq!(hex::encode(device_public_key(&private_key).unwrap()), FIXTURE_PUBLIC_KEY_HEX);
        let signing_key = SigningKey::from_bytes(&private_key.clone().try_into().unwrap());
        assert_eq!(hex::encode(signing_key.sign(b"device-auth").to_bytes()), FIXTURE_DEVICE_AUTH_SIGNATURE_HEX);

        let header = build_device_auth_header(&private_key, "GET", "/v1/path", "wallet1", b"body", 123).unwrap();
        assert!(verify_device_auth(&header, "GET", "/v1/path", b"body"));
        assert!(!verify_device_auth(&header, "POST", "/v1/path", b"body"));
        assert!(!verify_device_auth(&header, "GET", "/v1/path", b"tampered"));

        let payload = parse_device_auth(&header).unwrap();
        assert_eq!(payload.device_id, FIXTURE_PUBLIC_KEY_HEX);
        assert_eq!(payload.timestamp, "123");
        assert_eq!(payload.wallet_id.as_deref(), Some("wallet1"));
        assert_eq!(payload.body_hash, device_body_hash(b"body"));

        let golden = parse_device_auth(&build_device_auth_header(&private_key, "POST", "/v2/devices", "multicoin_0xabc", br#"{"device":"android"}"#, 123).unwrap()).unwrap();
        assert_eq!(golden.body_hash, "f7f90abc33e204d8a7f7821efc63eae3f72a0513161b92d61156528860f1d75b");
        assert_eq!(
            hex::encode(golden.signature),
            "0435dc3dbeff5d054e09d990ffb10d000d50d7a9e92f98da69ad4d57d4ee503c38aa48d9f0a329be7a564c56e0a58e2de1c9a408c356fe5ff075a997fe2e1b0b"
        );
    }
}
