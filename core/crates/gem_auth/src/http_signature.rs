use std::collections::HashMap;

use alloy_primitives::hex;
use ed25519_dalek::{Signer, SigningKey};
use sfv::{BareItem, Dictionary, FieldType, InnerList, Item, Key, List, ListEntry, Parameters, Parser, StringRef, key_ref};

use crate::device_signature::{device_signing_key, verify_device_signature};

const SIGNATURE_INPUT_HEADER: &str = "Signature-Input";
const SIGNATURE_HEADER: &str = "Signature";
const CONTENT_DIGEST_HEADER: &str = "Content-Digest";
const WALLET_ID_HEADER: &str = "Gem-Wallet-Id";

const SIGNATURE_LABEL: &str = "device";
const SIGNATURE_TAG: &str = "gem-device";
const SIGNATURE_ALGORITHM: &str = "ed25519";
const DIGEST_ALGORITHM: &str = "sha-256";
const SHA256_LENGTH: usize = 32;

const CREATED_PARAMETER: &str = "created";
const KEY_ID_PARAMETER: &str = "keyid";
const ALGORITHM_PARAMETER: &str = "alg";
const TAG_PARAMETER: &str = "tag";
const SIGNATURE_PARAMETERS: usize = 4;

struct SignedComponents<'a> {
    method: &'a str,
    path: &'a str,
    query: Option<&'a str>,
    content_digest: &'a str,
    wallet_id: Option<&'a str>,
}

impl SignedComponents<'_> {
    fn values(&self) -> Vec<(&'static str, String)> {
        [
            ("@method", self.method.to_string()),
            ("@path", self.path.to_string()),
            ("@query", format!("?{}", self.query.unwrap_or_default())),
            ("content-digest", self.content_digest.to_string()),
        ]
        .into_iter()
        .chain(self.wallet_id.map(|wallet_id| ("gem-wallet-id", wallet_id.to_string())))
        .collect()
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
    let params = signature_params(&components, created, &key_id).map_err(|_| "invalid signature parameters")?;
    let (signature_input, signature) = signature_fields(&signing_key, &components, params).ok_or("invalid signature base")?;
    Ok([(CONTENT_DIGEST_HEADER, content_digest), (SIGNATURE_INPUT_HEADER, signature_input), (SIGNATURE_HEADER, signature)]
        .into_iter()
        .chain(wallet_id.map(|wallet_id| (WALLET_ID_HEADER, wallet_id.to_string())))
        .map(|(name, value)| (name.to_string(), value))
        .collect())
}

fn signature_fields(signing_key: &SigningKey, components: &[(&str, String)], params: InnerList) -> Option<(String, String)> {
    let base = signature_base(components, &signature_params_value(&params))?;
    let signature = signing_key.sign(base.as_bytes()).to_bytes();
    Some((
        Dictionary::from_iter([(key(SIGNATURE_LABEL), ListEntry::InnerList(params))]).serialize().unwrap_or_default(),
        Dictionary::from_iter([(key(SIGNATURE_LABEL), ListEntry::from(signature.to_vec()))]).serialize().unwrap_or_default(),
    ))
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
    components: Vec<String>,
    signature_params: String,
    signature: Vec<u8>,
}

impl DeviceSignature {
    pub fn parse(headers: &DeviceSignatureHeaders) -> Option<Self> {
        let inputs: Dictionary = Parser::new(headers.signature_input).parse().ok()?;
        let signatures: Dictionary = Parser::new(headers.signature).parse().ok()?;
        let Some(ListEntry::InnerList(params)) = inputs.get(SIGNATURE_LABEL) else {
            return None;
        };
        let signature = byte_sequence(signatures.get(SIGNATURE_LABEL)?)?;
        let components = params.items.iter().map(bare_string).collect::<Option<Vec<_>>>()?;
        if params.params.len() != SIGNATURE_PARAMETERS || string_parameter(&params.params, ALGORITHM_PARAMETER)? != SIGNATURE_ALGORITHM || string_parameter(&params.params, TAG_PARAMETER)? != SIGNATURE_TAG {
            return None;
        }
        let created = u64::try_from(params.params.get(CREATED_PARAMETER)?.as_integer()?).ok()?;
        let device_id = string_parameter(&params.params, KEY_ID_PARAMETER)?.to_string();
        let content_digest = headers.content_digest.trim();
        let wallet_id = match headers.wallet_id.map(str::trim) {
            Some("") => return None,
            wallet_id => wallet_id.map(str::to_string),
        };
        Some(Self {
            device_id,
            created,
            wallet_id,
            body_sha256: sha256_digest(content_digest)?,
            content_digest: content_digest.to_string(),
            components,
            signature_params: signature_params_value(params),
            signature,
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
        let covers_request = components.iter().map(|(name, _)| *name).eq(self.components.iter().map(String::as_str));
        covers_request && signature_base(&components, &self.signature_params).is_some_and(|base| verify_device_signature(&self.device_id, &base, &self.signature))
    }
}

fn content_digest(body: &[u8]) -> String {
    Dictionary::from_iter([(key(DIGEST_ALGORITHM), ListEntry::from(gem_hash::sha2::sha256(body).to_vec()))]).serialize().unwrap_or_default()
}

fn signature_params(components: &[(&str, String)], created: u64, key_id: &str) -> Result<InnerList, sfv::Error> {
    let items = components.iter().map(|(name, _)| StringRef::from_str(name).map(Item::new)).collect::<Result<Vec<_>, _>>()?;
    let params = Parameters::from_iter([
        (key(CREATED_PARAMETER), BareItem::try_from(created)?),
        (key(KEY_ID_PARAMETER), StringRef::from_str(key_id)?.into()),
        (key(ALGORITHM_PARAMETER), StringRef::from_str(SIGNATURE_ALGORITHM)?.into()),
        (key(TAG_PARAMETER), StringRef::from_str(SIGNATURE_TAG)?.into()),
    ]);
    Ok(InnerList::with_params(items, params))
}

fn signature_params_value(params: &InnerList) -> String {
    List::from([ListEntry::InnerList(params.clone())]).serialize().unwrap_or_default()
}

fn signature_base(components: &[(&str, String)], signature_params: &str) -> Option<String> {
    let base = components
        .iter()
        .map(|(name, value)| format!("\"{name}\": {value}\n"))
        .chain([format!("\"@signature-params\": {signature_params}")])
        .collect::<String>();
    (base.is_ascii() && components.iter().all(|(_, value)| !value.contains('\n'))).then_some(base)
}

fn sha256_digest(content_digest: &str) -> Option<Vec<u8>> {
    let digests: Dictionary = Parser::new(content_digest).parse().ok()?;
    byte_sequence(digests.get(DIGEST_ALGORITHM)?).filter(|digest| digest.len() == SHA256_LENGTH)
}

fn byte_sequence(entry: &ListEntry) -> Option<Vec<u8>> {
    match entry {
        ListEntry::Item(item) if item.params.is_empty() => item.bare_item.as_byte_sequence().map(<[u8]>::to_vec),
        _ => None,
    }
}

fn bare_string(item: &Item) -> Option<String> {
    match item.bare_item.as_string() {
        Some(value) if item.params.is_empty() => Some(value.as_str().to_string()),
        _ => None,
    }
}

fn string_parameter<'a>(params: &'a Parameters, name: &str) -> Option<&'a str> {
    params.get(name)?.as_string().map(StringRef::as_str)
}

fn key(name: &str) -> Key {
    key_ref(name).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::encode_base64;

    const PRIVATE_KEY: [u8; 32] = [1u8; 32];
    const CREATED: u64 = 1_762_870_000;
    const WALLET_ID: &str = "view_near_alice.near";
    const RFC_9421_PRIVATE_KEY_HEX: &str = "9f8362f87a484a954e6e740c5b4c0e84229139a20aa8ab56ff66586f6a7d29c5";
    const RFC_9421_PUBLIC_KEY_HEX: &str = "26b40b8f93fff3d897112f7ebc582b232dbd72517d082fe83cfb30ddce43d1bb";

    fn parse(headers: &HashMap<String, String>) -> Option<DeviceSignature> {
        DeviceSignature::parse(&DeviceSignatureHeaders::from_fields(|name| headers.get(name).map(String::as_str)).ok()?)
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
        let inputs: Dictionary = Parser::new(&format!("sig-b26={signature_params}")).parse().unwrap();
        let Some(ListEntry::InnerList(params)) = inputs.get("sig-b26") else { panic!("missing sig-b26") };
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

        let base = signature_base(&components, &signature_params_value(params)).unwrap();

        assert_eq!(hex::encode(signing_key.verifying_key().to_bytes()), RFC_9421_PUBLIC_KEY_HEX);
        assert_eq!(signature_params_value(params), signature_params);
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
        let headers = sign_device_request(&PRIVATE_KEY, "GET", "/v4/devices/subscriptions", None, b"", CREATED).unwrap();
        let signature = parse(&headers).unwrap();

        assert_eq!(headers.len(), 3);
        assert_eq!(
            headers[SIGNATURE_INPUT_HEADER],
            format!(r#"device=("@method" "@path" "@query" "content-digest");created={CREATED};keyid="{}";alg="ed25519";tag="gem-device""#, signature.device_id)
        );
        assert_eq!(signature.device_id, hex::encode(crate::device_public_key(&PRIVATE_KEY).unwrap()));
        assert_eq!(signature.created, CREATED);
        assert_eq!(signature.wallet_id, None);
        assert!(signature.verify("GET", "/v4/devices/subscriptions", None));
        assert!(!signature.verify("POST", "/v4/devices/subscriptions", None));
        assert!(!signature.verify("GET", "/v4/devices", None));
    }

    #[test]
    fn test_sign_wallet_request() {
        let body = br#"{"code":"alice"}"#;
        let headers = sign_device_request(&PRIVATE_KEY, "POST", "/v4/devices/transactions?asset_id=near&limit=5", Some(WALLET_ID), body, CREATED).unwrap();
        let signature = parse(&headers).unwrap();

        assert_eq!(headers[WALLET_ID_HEADER], WALLET_ID);
        assert_eq!(signature.wallet_id.as_deref(), Some(WALLET_ID));
        assert_eq!(signature.body_sha256, gem_hash::sha2::sha256(body));
        assert!(signature.verify("POST", "/v4/devices/transactions", Some("asset_id=near&limit=5")));
        assert!(!signature.verify("POST", "/v4/devices/transactions", Some("asset_id=bitcoin&limit=5")));
        assert!(!signature.verify("POST", "/v4/devices/transactions", None));
    }

    #[test]
    fn test_verify_rejects_changed_headers() {
        let device = sign_device_request(&PRIVATE_KEY, "GET", "/v4/devices/assets", None, b"", CREATED).unwrap();
        let wallet = sign_device_request(&PRIVATE_KEY, "GET", "/v4/devices/assets", Some(WALLET_ID), b"", CREATED).unwrap();
        let other_digest = content_digest(b"other");

        for headers in [
            replaced(&wallet, WALLET_ID_HEADER, "view_near_bob.near"),
            without(&wallet, WALLET_ID_HEADER),
            replaced(&device, WALLET_ID_HEADER, WALLET_ID),
            replaced(&device, CONTENT_DIGEST_HEADER, &other_digest),
        ] {
            assert!(!parse(&headers).unwrap().verify("GET", "/v4/devices/assets", None));
        }
    }

    #[test]
    fn test_verify_rejects_signature_input_missing_a_signed_component() {
        let signing_key = device_signing_key(&PRIVATE_KEY).unwrap();
        let key_id = hex::encode(signing_key.verifying_key().to_bytes());
        let digest = content_digest(b"");
        let components = SignedComponents {
            method: "GET",
            path: "/v4/devices/assets",
            query: None,
            content_digest: &digest,
            wallet_id: Some(WALLET_ID),
        }
        .values();
        let params = signature_params(&components[..4], CREATED, &key_id).unwrap();
        let (signature_input, signature) = signature_fields(&signing_key, &components, params).unwrap();
        let headers = HashMap::from([
            (SIGNATURE_INPUT_HEADER.to_string(), signature_input),
            (SIGNATURE_HEADER.to_string(), signature),
            (CONTENT_DIGEST_HEADER.to_string(), digest.clone()),
            (WALLET_ID_HEADER.to_string(), WALLET_ID.to_string()),
        ]);

        assert!(!parse(&headers).unwrap().verify("GET", "/v4/devices/assets", None));
    }

    #[test]
    fn test_parse_rejects_invalid_headers() {
        let headers = sign_device_request(&PRIVATE_KEY, "GET", "/v4/devices/assets", Some(WALLET_ID), b"", CREATED).unwrap();
        let signature_input = &headers[SIGNATURE_INPUT_HEADER];

        for invalid in [
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replace(r#"alg="ed25519""#, r#"alg="rsa-pss-sha512""#)),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &signature_input.replace(r#";tag="gem-device""#, "")),
            replaced(&headers, SIGNATURE_INPUT_HEADER, &format!(r#"{signature_input};nonce="1""#)),
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
