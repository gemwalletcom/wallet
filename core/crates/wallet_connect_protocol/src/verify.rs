use gem_encoding::{decode_base64_url, encode_hex};
use gem_hash::sha2::sha256;
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use primitives::WalletConnectionVerificationStatus;
use serde::{Deserialize, Serialize};

const VERIFY_URL: &str = "https://verify.walletconnect.org";
pub const PUBLIC_KEY_URL: &str = "https://verify.walletconnect.org/v3/public-key";
const UNCOMPRESSED_POINT: u8 = 0x04;

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

pub fn attested(jwt: &str, key: &VerifyPublicKey, attestation_id: &str, now: u64) -> Option<Attestation> {
    let (signing_input, signature) = jwt.rsplit_once('.')?;
    let (_, claims) = signing_input.split_once('.')?;
    let point = [vec![UNCOMPRESSED_POINT], decode_base64_url(&key.public_key.x).ok()?, decode_base64_url(&key.public_key.y).ok()?].concat();
    let verifying_key = VerifyingKey::from_sec1_bytes(&point).ok()?;
    let signature = Signature::from_slice(&decode_base64_url(signature).ok()?).ok()?;
    verifying_key.verify(signing_input.as_bytes(), &signature).ok()?;
    let claims: AttestationClaims = serde_json::from_slice(&decode_base64_url(claims).ok()?).ok()?;
    (claims.id == attestation_id && claims.exp > now).then_some(Attestation {
        origin: Some(claims.origin),
        is_scam: claims.is_scam,
        is_verified: claims.is_verified,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_ATTESTATION, TEST_ATTESTATION_ID, test_attestation, test_verify_key};

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

        assert_eq!(
            attested(TEST_ATTESTATION, &key, TEST_ATTESTATION_ID, issued),
            Some(Attestation {
                origin: Some("https://react-dapp-v2-git-chore-verify-v2-samples-walletconnect1.vercel.app".to_string()),
                is_scam: None,
                is_verified: Some(true),
            })
        );
        assert_eq!(attested(TEST_ATTESTATION, &key, "another message", issued), None, "an attestation proves only the message it was issued for");
        assert_eq!(attested(TEST_ATTESTATION, &key, TEST_ATTESTATION_ID, 1_723_471_723), None, "an expired attestation proves nothing");

        let forged = TEST_ATTESTATION.replace("eyJleHAiOjE3MjM0NzE3MjMs", "eyJleHAiOjE3MjM0NzE3MjQs");
        assert_eq!(attested(&forged, &key, TEST_ATTESTATION_ID, issued), None, "claims a dapp edited fail the server's signature");

        let other_key = VerifyPublicKey {
            public_key: Jwk {
                x: key.public_key.y.clone(),
                y: key.public_key.x.clone(),
            },
            ..key
        };
        assert_eq!(attested(TEST_ATTESTATION, &other_key, TEST_ATTESTATION_ID, issued), None);
        assert_eq!(attested("not.a.jwt", &VerifyPublicKey::mock(), TEST_ATTESTATION_ID, issued), None);
    }

    #[test]
    fn test_attested_by_the_test_key() {
        let attestation = test_attestation("message", "https://example.com", true, 200);

        assert_eq!(
            attested(&attestation, &test_verify_key(200), "message", 100).map(Attestation::verification).map(|verification| verification.status),
            Some(WalletConnectionVerificationStatus::Malicious)
        );
        assert_eq!(attested(&attestation, &VerifyPublicKey::mock(), "message", 100), None);
    }

    #[test]
    fn test_attestation_url() {
        assert_eq!(attestation_url(b"{}"), format!("{VERIFY_URL}/attestation/{}?v2Supported=true", encode_hex(sha256(b"{}"))));
        assert_eq!(attestation_id("AAAB"), encode_hex(sha256(b"AAAB")));
    }
}
