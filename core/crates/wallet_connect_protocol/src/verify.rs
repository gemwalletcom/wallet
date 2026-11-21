use gem_encoding::{decode_base64_url, encode_hex};
use gem_hash::sha2::sha256;
use primitives::WalletConnectionVerificationStatus;
use serde::{Deserialize, Serialize};

const VERIFY_URL: &str = "https://verify.walletconnect.org";
pub const PUBLIC_KEY_URL: &str = "https://verify.walletconnect.org/v3/public-key";
const UNCOMPRESSED_POINT: u8 = 0x04;
const COORDINATE_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attestation {
    pub origin: Option<String>,
    pub is_scam: Option<bool>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyPublicKey {
    pub public_key: Jwk,
    pub expires_at: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Jwk {
    pub x: String,
    pub y: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttestationClaims {
    exp: u64,
    id: String,
    origin: String,
    is_scam: Option<bool>,
    is_verified: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignedMessage {
    pub public_key: Vec<u8>,
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Verification {
    pub origin: Option<String>,
    pub status: WalletConnectionVerificationStatus,
}

impl Verification {
    pub fn unknown() -> Self {
        Self {
            origin: None,
            status: WalletConnectionVerificationStatus::Unknown,
        }
    }
}

impl Attestation {
    pub fn verification(self) -> Verification {
        let status = match (self.is_scam, self.is_verified, &self.origin) {
            (Some(true), _, _) => WalletConnectionVerificationStatus::Malicious,
            (_, Some(false), _) | (_, _, None) => WalletConnectionVerificationStatus::Unknown,
            (_, _, Some(_)) => WalletConnectionVerificationStatus::Verified,
        };
        Verification { origin: self.origin, status }
    }
}

pub fn attestation_id(encrypted_message: &str) -> String {
    encode_hex(sha256(encrypted_message.as_bytes()))
}

pub fn attestation_url(decrypted_message: &[u8]) -> String {
    format!("{VERIFY_URL}/attestation/{}?v2Supported=true", encode_hex(sha256(decrypted_message)))
}

pub fn attested(jwt: &str, key: &VerifyPublicKey, attestation_id: &str, now: u64, is_valid_es256: impl FnOnce(SignedMessage) -> bool) -> Option<Attestation> {
    let (signing_input, signature) = jwt.rsplit_once('.')?;
    let (_, claims) = signing_input.split_once('.')?;
    let signed = SignedMessage {
        public_key: [vec![UNCOMPRESSED_POINT], decode_exact(&key.public_key.x, COORDINATE_LENGTH)?, decode_exact(&key.public_key.y, COORDINATE_LENGTH)?].concat(),
        message: signing_input.as_bytes().to_vec(),
        signature: decode_exact(signature, SIGNATURE_LENGTH)?,
    };
    if !is_valid_es256(signed) {
        return None;
    }
    let claims: AttestationClaims = serde_json::from_slice(&decode_base64_url(claims).ok()?).ok()?;
    (claims.id == attestation_id && claims.exp > now).then_some(Attestation {
        origin: Some(claims.origin),
        is_scam: claims.is_scam,
        is_verified: claims.is_verified,
    })
}

fn decode_exact(value: &str, length: usize) -> Option<Vec<u8>> {
    decode_base64_url(value).ok().filter(|bytes| bytes.len() == length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_ATTESTATION, TEST_ATTESTATION_ID};

    #[test]
    fn test_verification() {
        let status = |json: &str| serde_json::from_str::<Attestation>(json).unwrap().verification().status;

        assert_eq!(status(r#"{"origin":"https://example.com","isScam":false}"#), WalletConnectionVerificationStatus::Verified);
        assert_eq!(status(r#"{"origin":"https://example.com","isScam":true,"isVerified":true}"#), WalletConnectionVerificationStatus::Malicious);
        assert_eq!(status(r#"{"origin":"https://example.com","isVerified":false}"#), WalletConnectionVerificationStatus::Unknown);
        assert_eq!(serde_json::from_str::<Attestation>("{}").unwrap().verification(), Verification::unknown());
    }

    #[test]
    fn test_attested() {
        let key = VerifyPublicKey::mock();
        let issued = 1_723_000_000;
        let signed_by_the_server = |signed: SignedMessage| signed == SignedMessage::mock();

        assert_eq!(
            attested(TEST_ATTESTATION, &key, TEST_ATTESTATION_ID, issued, signed_by_the_server),
            Some(Attestation {
                origin: Some("https://react-dapp-v2-git-chore-verify-v2-samples-walletconnect1.vercel.app".to_string()),
                is_scam: None,
                is_verified: Some(true),
            })
        );
        assert_eq!(attested(TEST_ATTESTATION, &key, TEST_ATTESTATION_ID, issued, |_| false), None, "the platform's signature check decides");
        assert_eq!(
            attested(TEST_ATTESTATION, &key, "another message", issued, signed_by_the_server),
            None,
            "an attestation proves only the message it was issued for"
        );
        assert_eq!(attested(TEST_ATTESTATION, &key, TEST_ATTESTATION_ID, 1_723_471_723, signed_by_the_server), None, "an expired attestation proves nothing");

        let forged = TEST_ATTESTATION.replace("eyJleHAiOjE3MjM0NzE3MjMs", "eyJleHAiOjE3MjM0NzE3MjQs");
        assert_eq!(attested(&forged, &key, TEST_ATTESTATION_ID, issued, signed_by_the_server), None, "the signed message covers the claims a dapp could edit");

        let other_key = VerifyPublicKey {
            public_key: Jwk {
                x: key.public_key.y.clone(),
                y: key.public_key.x.clone(),
            },
            ..key.clone()
        };
        assert_eq!(attested(TEST_ATTESTATION, &other_key, TEST_ATTESTATION_ID, issued, signed_by_the_server), None);

        let short_key = VerifyPublicKey {
            public_key: Jwk {
                x: "svgM5g5vWe137w6YTE76hLQNYIwLTQOe2vKYmgHy2Q".to_string(),
                y: key.public_key.y.clone(),
            },
            ..key.clone()
        };
        let truncated = &TEST_ATTESTATION[..TEST_ATTESTATION.len() - 2];
        assert_eq!(attested(TEST_ATTESTATION, &short_key, TEST_ATTESTATION_ID, issued, |_| true), None, "a coordinate is exactly 32 bytes");
        assert_eq!(attested(truncated, &key, TEST_ATTESTATION_ID, issued, |_| true), None, "a signature is exactly r and s, 32 bytes each");
        assert_eq!(attested("not.a.jwt", &key, TEST_ATTESTATION_ID, issued, |_| true), None);
    }

    #[test]
    fn test_attestation_url() {
        assert_eq!(attestation_url(b"{}"), format!("{VERIFY_URL}/attestation/{}?v2Supported=true", encode_hex(sha256(b"{}"))));
        assert_eq!(attestation_id("AAAB"), encode_hex(sha256(b"AAAB")));
    }
}
