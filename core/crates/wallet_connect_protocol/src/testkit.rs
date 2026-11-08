use std::collections::BTreeMap;

use serde_json::value::RawValue;

use crate::authenticate::AuthPayload;
use crate::crypto::{Key, parse_key};
use crate::sign::{Metadata, ProposalNamespace, SessionNamespace, SessionProposal};
use crate::verify::{Jwk, VerifyPublicKey};

pub const TEST_SELF_SECRET: &str = "a8c0f6a0d6e2f7d4f0e1d2c3b4a5968778695a4b3c2d1e0f1f2e3d4c5b6a7988";
pub const TEST_PEER_SECRET: &str = "1f2e3d4c5b6a79881a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f7081";
pub const TEST_SELF_PUBLIC_KEY: &str = "73d95c3e17b12dc3df0c53fe2b98e7db4a0eca06305ebb12086e1237b7d1e559";
pub const TEST_PEER_PUBLIC_KEY: &str = "ff3b9f3f8fe885b110e185c1c935167e24107ecb8d00bb79f8e75e568a920a20";
pub const TEST_SYM_KEY: &str = "47b79ea50ddea8931b2a16e9c04264f8fff1850e95b5d578d92c9dbdb1ea630d";
pub const TEST_TOPIC: &str = "eec4db5e6f3e1b8aab5f72a3a8f293882aad0e966f2d7170ccdbdbe1f4f5bca6";
pub const TEST_PLAINTEXT: &str = r#"{"id":1,"jsonrpc":"2.0","method":"wc_sessionPing","params":{}}"#;
pub const TEST_TYPE0_ENVELOPE: &str = "AAABAgMEBQYHCAkKCw7nLVDE+qwiAcmAfZGoTgQoj3jY5jlff+fP7nf8foejJJoc7wtQ6JMMGugNhF8F9q3cCku6XO7MCVGlMDRCG8K5e1ELx4JwSON5SYtEBg==";
pub const TEST_TYPE1_ENVELOPE: &str = "AXPZXD4XsS3D3wxT/iuY59tKDsoGMF67EghuEje30eVZAAECAwQFBgcICQoLDuctUMT6rCIByYB9kahOBCiPeNjmOV9/58/ud/x+h6MkmhzvC1Dokwwa6A2EXwX2rdwKS7pc7swJUaUwNEIbwrl7UQvHgnBI43lJi0QG";

pub fn test_key(value: &str) -> Key {
    parse_key(value).unwrap()
}

pub fn raw_json(json: &str) -> Box<RawValue> {
    RawValue::from_string(json.to_string()).unwrap()
}

pub fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

impl Metadata {
    pub fn mock() -> Self {
        Self {
            name: "Example".to_string(),
            description: "Example dapp".to_string(),
            url: "https://example.com".to_string(),
            icons: vec!["https://example.com/icon.png".to_string()],
            redirect: None,
        }
    }
}

impl SessionNamespace {
    pub fn mock(chains: &[&str], accounts: &[&str], methods: &[&str], events: &[&str]) -> Self {
        Self {
            chains: Some(strings(chains)),
            accounts: strings(accounts),
            methods: strings(methods),
            events: strings(events),
        }
    }
}

impl ProposalNamespace {
    pub fn mock(chains: &[&str], methods: &[&str], events: &[&str]) -> Self {
        Self {
            chains: (!chains.is_empty()).then(|| strings(chains)),
            methods: strings(methods),
            events: strings(events),
        }
    }
}

impl SessionProposal {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("../testdata/session_proposal.json")).unwrap()
    }

    pub fn mock_with_required(key: &str, chains: &[&str], methods: &[&str], events: &[&str]) -> Self {
        Self {
            required_namespaces: BTreeMap::from([(key.to_string(), ProposalNamespace::mock(chains, methods, events))]),
            optional_namespaces: None,
            ..Self::mock()
        }
    }

    pub fn mock_with_optional(key: &str, chains: &[&str], methods: &[&str]) -> Self {
        Self {
            required_namespaces: BTreeMap::new(),
            optional_namespaces: Some(BTreeMap::from([(key.to_string(), ProposalNamespace::mock(chains, methods, &[]))])),
            ..Self::mock()
        }
    }
}

impl AuthPayload {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("../testdata/authenticate_payload.json")).unwrap()
    }
}

pub const TEST_ATTESTATION: &str = "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MjM0NzE3MjMsImlkIjoiYjNmYmZhMDUxZDJhNjdkNGRmNTYzM2IyMjc0NDAyNTUxMTg1NzQwZGQwMjA3YWM0OWI1M2RiYTcxOTc0YTgzNCIsIm9yaWdpbiI6Imh0dHBzOi8vcmVhY3QtZGFwcC12Mi1naXQtY2hvcmUtdmVyaWZ5LXYyLXNhbXBsZXMtd2FsbGV0Y29ubmVjdDEudmVyY2VsLmFwcCIsImlzU2NhbSI6bnVsbCwiaXNWZXJpZmllZCI6dHJ1ZX0.8RQwiEEfTGn8p3INRdHpi88dpzetKCp3nscfLtWG2cVE2dU0dWgV2ncqnh_RWmygqEnWCPUlH1RMwS1nWbZzrQ";
pub const TEST_ATTESTATION_ID: &str = "b3fbfa051d2a67d4df5633b2274402551185740dd0207ac49b53dba71974a834";

impl VerifyPublicKey {
    pub fn mock() -> Self {
        serde_json::from_str(include_str!("../testdata/verify_public_key.json")).unwrap()
    }
}

const TEST_VERIFY_SECRET: [u8; 32] = [7; 32];

pub fn test_verify_key(expires_at: u64) -> VerifyPublicKey {
    let point = p256::ecdsa::SigningKey::from_slice(&TEST_VERIFY_SECRET).unwrap().verifying_key().to_sec1_point(false);
    VerifyPublicKey {
        public_key: Jwk {
            x: gem_encoding::encode_base64_url(point.x().unwrap()),
            y: gem_encoding::encode_base64_url(point.y().unwrap()),
        },
        expires_at,
    }
}

pub fn test_attestation(attestation_id: &str, origin: &str, is_scam: bool, expires_at: u64) -> String {
    use p256::ecdsa::signature::Signer;
    let signing_key = p256::ecdsa::SigningKey::from_slice(&TEST_VERIFY_SECRET).unwrap();
    let claims = serde_json::json!({"exp": expires_at, "id": attestation_id, "origin": origin, "isScam": is_scam, "isVerified": true});
    let signing_input = format!("{}.{}", gem_encoding::encode_base64_url(br#"{"alg":"ES256","typ":"JWT"}"#), gem_encoding::encode_base64_url(claims.to_string().as_bytes()));
    let signature: p256::ecdsa::Signature = signing_key.sign(signing_input.as_bytes());
    format!("{signing_input}.{}", gem_encoding::encode_base64_url(&signature.to_bytes()))
}
