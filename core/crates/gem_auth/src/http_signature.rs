use std::collections::HashMap;

use alloy_primitives::hex;
use ed25519_dalek::Signer;
use gem_encoding::{decode_base64, encode_base64};

use crate::device_signature::{device_signing_key, verify_device_signature};

const SIGNATURE_INPUT_HEADER: &str = "Signature-Input";
const SIGNATURE_HEADER: &str = "Signature";
const CONTENT_DIGEST_HEADER: &str = "Content-Digest";
const WALLET_ID_HEADER: &str = "Gem-Wallet-Id";

const SIGNATURE_LABEL: &str = "device";
const DIGEST_ALGORITHM: &str = "sha-256";
const DEVICE_COMPONENTS: [&str; 4] = ["@method", "@path", "@query", "content-digest"];
const WALLET_COMPONENT: &str = "gem-wallet-id";
const SIGNATURE_LENGTH: usize = 64;
const SHA256_LENGTH: usize = 32;

struct SignedComponents<'a> {
    method: &'a str,
    path: &'a str,
    query: Option<&'a str>,
    content_digest: &'a str,
    wallet_id: Option<&'a str>,
}

impl SignedComponents<'_> {
    fn values(&self) -> Vec<(&'static str, String)> {
        let values = [self.method.to_string(), self.path.to_string(), format!("?{}", self.query.unwrap_or_default()), self.content_digest.to_string()]
            .into_iter()
            .chain(self.wallet_id.map(str::to_string));
        covered_components(self.wallet_id.is_some()).zip(values).collect()
    }
}

pub fn sign_device_request(private_key: &[u8], method: &str, target: &str, wallet_id: Option<&str>, body: &[u8], created: u64) -> Result<HashMap<String, String>, &'static str> {
    let signing_key = device_signing_key(private_key)?;
    let key_id = hex::encode(signing_key.verifying_key().to_bytes());
    let content_digest = content_digest(body);
    let (path, query) = target.split_once('?').map_or((target, None), |(path, query)| (path, Some(query)));
    let components = SignedComponents {
        method,
        path,
        query,
        content_digest: &content_digest,
        wallet_id,
    }
    .values();
    let signature_params = device_signature_params(wallet_id.is_some(), created, &key_id);
    let base = signature_base(&components, &signature_params).ok_or("invalid signature base")?;
    let signature = signing_key.sign(base.as_bytes()).to_bytes();
    Ok([
        (CONTENT_DIGEST_HEADER, content_digest),
        (SIGNATURE_INPUT_HEADER, format!("{SIGNATURE_LABEL}={signature_params}")),
        (SIGNATURE_HEADER, format!("{SIGNATURE_LABEL}=:{}:", encode_base64(&signature))),
    ]
    .into_iter()
    .chain(wallet_id.map(|wallet_id| (WALLET_ID_HEADER, wallet_id.to_string())))
    .map(|(name, value)| (name.to_string(), value))
    .collect())
}

pub struct DeviceSignatureHeaders<'a> {
    pub signature_input: &'a str,
    pub signature: &'a str,
    pub content_digest: &'a str,
    pub wallet_id: Option<&'a str>,
}

impl<'a> DeviceSignatureHeaders<'a> {
    pub fn from_fields(field: impl Fn(&str) -> Option<&'a str>) -> Result<Self, &'static str> {
        Ok(Self {
            signature_input: field(SIGNATURE_INPUT_HEADER).ok_or(SIGNATURE_INPUT_HEADER)?,
            signature: field(SIGNATURE_HEADER).ok_or(SIGNATURE_HEADER)?,
            content_digest: field(CONTENT_DIGEST_HEADER).ok_or(CONTENT_DIGEST_HEADER)?,
            wallet_id: field(WALLET_ID_HEADER),
        })
    }
}

pub struct DeviceSignature {
    pub device_id: String,
    pub created: u64,
    pub wallet_id: Option<String>,
    pub body_sha256: Vec<u8>,
    content_digest: String,
    signature_params: String,
    signature: Vec<u8>,
}

impl DeviceSignature {
    pub fn from_fields<'a>(field: impl Fn(&str) -> Option<&'a str>) -> Option<Self> {
        Self::parse(&DeviceSignatureHeaders::from_fields(field).ok()?)
    }

    pub fn parse(headers: &DeviceSignatureHeaders) -> Option<Self> {
        let wallet_id = match headers.wallet_id.map(str::trim) {
            Some("") => return None,
            wallet_id => wallet_id.map(str::to_string),
        };
        let signature_params = headers.signature_input.trim().strip_prefix(SIGNATURE_LABEL)?.strip_prefix('=')?;
        let created = parameter(signature_params, ";created=", ';')?.parse::<u64>().ok()?;
        let device_id = parameter(signature_params, ";keyid=\"", '"')?;
        if signature_params != device_signature_params(wallet_id.is_some(), created, device_id) {
            return None;
        }
        let content_digest = headers.content_digest.trim();
        Some(Self {
            device_id: device_id.to_string(),
            created,
            wallet_id,
            body_sha256: byte_sequence(content_digest, DIGEST_ALGORITHM, SHA256_LENGTH)?,
            content_digest: content_digest.to_string(),
            signature_params: signature_params.to_string(),
            signature: byte_sequence(headers.signature.trim(), SIGNATURE_LABEL, SIGNATURE_LENGTH)?,
        })
    }

    pub fn verify(&self, method: &str, path: &str, query: Option<&str>) -> bool {
        let components = SignedComponents {
            method,
            path,
            query,
            content_digest: &self.content_digest,
            wallet_id: self.wallet_id.as_deref(),
        }
        .values();
        signature_base(&components, &self.signature_params).is_some_and(|base| verify_device_signature(&self.device_id, &base, &self.signature))
    }
}

fn covered_components(signs_wallet: bool) -> impl Iterator<Item = &'static str> {
    DEVICE_COMPONENTS.into_iter().chain(signs_wallet.then_some(WALLET_COMPONENT))
}

fn device_signature_params(signs_wallet: bool, created: u64, key_id: &str) -> String {
    let components = covered_components(signs_wallet).map(|name| format!("\"{name}\"")).collect::<Vec<_>>().join(" ");
    format!(r#"({components});created={created};keyid="{key_id}""#)
}

fn content_digest(body: &[u8]) -> String {
    format!("{DIGEST_ALGORITHM}=:{}:", encode_base64(&gem_hash::sha2::sha256(body)))
}

fn signature_base(components: &[(&str, String)], signature_params: &str) -> Option<String> {
    let base = components
        .iter()
        .map(|(name, value)| format!("\"{name}\": {value}\n"))
        .chain([format!("\"@signature-params\": {signature_params}")])
        .collect::<String>();
    (base.is_ascii() && components.iter().all(|(_, value)| !value.contains('\n'))).then_some(base)
}

fn parameter<'a>(signature_params: &'a str, prefix: &str, end: char) -> Option<&'a str> {
    signature_params.split_once(prefix)?.1.split(end).next()
}

fn byte_sequence(field: &str, key: &str, length: usize) -> Option<Vec<u8>> {
    let value = field.strip_prefix(key)?.strip_prefix("=:")?.strip_suffix(':')?;
    decode_base64(value).ok().filter(|bytes| bytes.len() == length)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY: [u8; 32] = [1u8; 32];
    const CREATED: u64 = 1_762_870_000;
    const WALLET_ID: &str = "view_near_alice.near";
    const RFC_9421_PRIVATE_KEY_HEX: &str = "9f8362f87a484a954e6e740c5b4c0e84229139a20aa8ab56ff66586f6a7d29c5";
    const RFC_9421_PUBLIC_KEY_HEX: &str = "26b40b8f93fff3d897112f7ebc582b232dbd72517d082fe83cfb30ddce43d1bb";

    fn parse(headers: &HashMap<String, String>) -> Option<DeviceSignature> {
        DeviceSignature::from_fields(|name| headers.get(name).map(String::as_str))
    }

    fn accepted(headers: &HashMap<String, String>, method: &str, path: &str) -> bool {
        parse(headers).is_some_and(|signature| signature.verify(method, path, None))
    }

    fn replaced(headers: &HashMap<String, String>, name: &str, value: &str) -> HashMap<String, String> {
        without(headers, name).into_iter().chain([(name.to_string(), value.to_string())]).collect()
    }

    fn without(headers: &HashMap<String, String>, name: &str) -> HashMap<String, String> {
        headers.iter().filter(|(key, _)| key.as_str() != name).map(|(key, value)| (key.clone(), value.clone())).collect()
    }

    #[test]
    fn test_signature_base_rfc_9421_ed25519_vector() {
        let signature_params = r#"("date" "@method" "@path" "@authority" "content-type" "content-length");created=1618884473;keyid="test-key-ed25519""#;
        let components = [
            ("date", "Tue, 20 Apr 2021 02:07:55 GMT"),
            ("@method", "POST"),
            ("@path", "/foo"),
            ("@authority", "example.com"),
            ("content-type", "application/json"),
            ("content-length", "18"),
        ]
        .map(|(name, value)| (name, value.to_string()));
        let signing_key = device_signing_key(&hex::decode(RFC_9421_PRIVATE_KEY_HEX).unwrap()).unwrap();

        let base = signature_base(&components, signature_params).unwrap();

        assert_eq!(hex::encode(signing_key.verifying_key().to_bytes()), RFC_9421_PUBLIC_KEY_HEX);
        assert_eq!(
            encode_base64(&signing_key.sign(base.as_bytes()).to_bytes()),
            "wqcAqbmYJ2ji2glfAMaRy4gruYYnx2nEFN2HN6jrnDnQCK1u02Gb04v9EDgwUPiu4A0w6vuQv5lIp5WPpBKRCw=="
        );
    }

    #[test]
    fn test_content_digest_rfc_9530_vectors() {
        assert_eq!(content_digest(br#"{"hello": "world"}"#), "sha-256=:X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=:");
        assert_eq!(content_digest(b""), "sha-256=:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=:");
    }

    #[test]
    fn test_sign_device_request() {
        let headers = sign_device_request(&PRIVATE_KEY, "GET", "/v1/devices/subscriptions", None, b"", CREATED).unwrap();
        let signature = parse(&headers).unwrap();

        assert_eq!(headers.len(), 3);
        assert_eq!(
            headers[SIGNATURE_INPUT_HEADER],
            format!(r#"device=("@method" "@path" "@query" "content-digest");created={CREATED};keyid="{}""#, signature.device_id)
        );
        assert_eq!(signature.device_id, hex::encode(crate::device_public_key(&PRIVATE_KEY).unwrap()));
        assert_eq!(signature.created, CREATED);
        assert_eq!(signature.wallet_id, None);
        assert!(signature.verify("GET", "/v1/devices/subscriptions", None));
        assert!(!signature.verify("POST", "/v1/devices/subscriptions", None));
        assert!(!signature.verify("GET", "/v1/devices", None));
    }

    #[test]
    fn test_sign_wallet_request() {
        let body = br#"{"code":"alice"}"#;
        let headers = sign_device_request(&PRIVATE_KEY, "POST", "/v1/devices/transactions?asset_id=near&limit=5", Some(WALLET_ID), body, CREATED).unwrap();
        let signature = parse(&headers).unwrap();

        assert_eq!(headers[WALLET_ID_HEADER], WALLET_ID);
        assert_eq!(signature.wallet_id.as_deref(), Some(WALLET_ID));
        assert_eq!(signature.body_sha256, gem_hash::sha2::sha256(body));
        assert!(signature.verify("POST", "/v1/devices/transactions", Some("asset_id=near&limit=5")));
        assert!(!signature.verify("POST", "/v1/devices/transactions", Some("asset_id=bitcoin&limit=5")));
        assert!(!signature.verify("POST", "/v1/devices/transactions", None));
    }

    #[test]
    fn test_rejects_changed_headers() {
        let device = sign_device_request(&PRIVATE_KEY, "GET", "/v1/devices/assets", None, b"", CREATED).unwrap();
        let wallet = sign_device_request(&PRIVATE_KEY, "GET", "/v1/devices/assets", Some(WALLET_ID), b"", CREATED).unwrap();
        let other_digest = content_digest(b"other");

        assert!(accepted(&wallet, "GET", "/v1/devices/assets"));
        for headers in [
            replaced(&wallet, WALLET_ID_HEADER, "view_near_bob.near"),
            without(&wallet, WALLET_ID_HEADER),
            replaced(&device, WALLET_ID_HEADER, WALLET_ID),
            replaced(&device, CONTENT_DIGEST_HEADER, &other_digest),
        ] {
            assert!(!accepted(&headers, "GET", "/v1/devices/assets"));
        }
    }

    #[test]
    fn test_parse_rejects_other_header_forms() {
        let headers = sign_device_request(&PRIVATE_KEY, "GET", "/v1/devices/assets", Some(WALLET_ID), b"", CREATED).unwrap();
        let signature_input = &headers[SIGNATURE_INPUT_HEADER];

        for invalid in [
            replaced(&headers, SIGNATURE_INPUT_HEADER, &format!(r#"{signature_input};alg="rsa-pss-sha512""#)),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &format!(r#"{signature_input};nonce="1""#)),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replace(";keyid=", ";expires=1;keyid=")),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replace(r#" "gem-wallet-id""#, "")),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replace("created=", "created=+")),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replace("created=", "created=0")),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replace("\" \"", "\"  \"")),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replacen("device=", "other=", 1)),
            replaced(&headers, SIGNATURE_HEADER, r#"device="signature""#),
            replaced(&headers, CONTENT_DIGEST_HEADER, "sha-512=:AAAA:"),
            replaced(&headers, WALLET_ID_HEADER, " "),
            without(&headers, SIGNATURE_HEADER),
        ] {
            assert!(parse(&invalid).is_none());
        }
    }
}
