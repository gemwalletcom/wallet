use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_encoding::decode_base64;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;

use super::model::TopperOnRamp;

#[derive(Debug, Serialize)]
struct TopperBootstrapClaims {
    iat: u64,
    jti: String,
    sub: String,
    #[serde(flatten)]
    flow: TopperOnRamp,
}

pub struct TopperBootstrapToken {
    widget_id: String,
    key_id: String,
    secret: String,
}

impl TopperBootstrapToken {
    pub fn new(widget_id: String, key_id: String, secret: String) -> Self {
        Self { widget_id, key_id, secret }
    }

    pub fn sign(&self, jti: &str, flow: TopperOnRamp) -> Result<String, Box<dyn Error + Send + Sync>> {
        let issued_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.sign_at(jti, flow, issued_at)
    }

    fn sign_at(&self, jti: &str, flow: TopperOnRamp, issued_at: u64) -> Result<String, Box<dyn Error + Send + Sync>> {
        let header = Header {
            kid: Some(self.key_id.clone()),
            ..Header::new(Algorithm::ES256)
        };
        let claims = TopperBootstrapClaims {
            iat: issued_at,
            jti: jti.to_string(),
            sub: self.widget_id.clone(),
            flow,
        };
        let key = EncodingKey::from_ec_der(&decode_base64(&self.secret)?);
        Ok(encode(&header, &claims, &key)?)
    }
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::jwk::Jwk;
    use jsonwebtoken::{DecodingKey, Validation, decode};

    use super::super::testkit::{TEST_KEY_ID, TEST_SIGNING_KEY, TEST_SIGNING_PUBLIC_KEY, TEST_WIDGET_ID};
    use super::*;

    const CLAIMS: &str = include_str!("../../../testdata/topper/bootstrap_token_claims.json");

    #[test]
    fn test_sign_at() {
        let token = TopperBootstrapToken::new(TEST_WIDGET_ID.to_string(), TEST_KEY_ID.to_string(), TEST_SIGNING_KEY.to_string())
            .sign_at("3f2c1b9e-7a4d-4c8e-9b1f-5d6e7a8b9c0d", TopperOnRamp::mock(), 1_800_000_000)
            .unwrap();
        let key = DecodingKey::from_jwk(&serde_json::from_str::<Jwk>(TEST_SIGNING_PUBLIC_KEY).unwrap()).unwrap();
        let mut validation = Validation::new(Algorithm::ES256);
        validation.required_spec_claims.clear();
        let data = decode::<serde_json::Value>(&token, &key, &validation).unwrap();

        assert_eq!(data.header.typ.as_deref(), Some("JWT"));
        assert_eq!(data.header.kid.as_deref(), Some(TEST_KEY_ID));
        assert_eq!(data.claims, serde_json::from_str::<serde_json::Value>(CLAIMS).unwrap());
    }
}
