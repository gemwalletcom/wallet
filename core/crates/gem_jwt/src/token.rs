use gem_encoding::{decode_base64_url, encode_base64_url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::key::Algorithm;
use crate::{JwtError, SigningKey, VerifyingKey};

const TOKEN_TYPE: &str = "JWT";

#[derive(Serialize)]
struct Header<'a> {
    typ: &'a str,
    alg: Algorithm,
    #[serde(skip_serializing_if = "Option::is_none")]
    kid: Option<&'a str>,
}

#[derive(Deserialize)]
struct HeaderAlgorithm {
    alg: Algorithm,
}

pub fn encode(key: &SigningKey, key_id: Option<&str>, claims: &impl Serialize) -> Result<String, JwtError> {
    let header = Header {
        typ: TOKEN_TYPE,
        alg: key.algorithm(),
        kid: key_id,
    };
    let message = format!("{}.{}", encode_part(&header)?, encode_part(claims)?);
    let signature = key.sign(message.as_bytes())?;
    Ok(format!("{message}.{signature}"))
}

pub fn decode<T: DeserializeOwned>(key: &VerifyingKey, token: &str) -> Result<T, JwtError> {
    let (message, signature) = token.rsplit_once('.').ok_or(JwtError::InvalidToken)?;
    let (header, claims) = message.split_once('.').ok_or(JwtError::InvalidToken)?;
    let header: HeaderAlgorithm = decode_part(header).ok_or(JwtError::InvalidToken)?;
    if header.alg != key.algorithm() {
        return Err(JwtError::InvalidToken);
    }
    if !key.verify(message.as_bytes(), signature) {
        return Err(JwtError::InvalidSignature);
    }
    decode_part(claims).ok_or(JwtError::InvalidClaims)
}

fn encode_part(part: &impl Serialize) -> Result<String, JwtError> {
    Ok(encode_base64_url(&serde_json::to_vec(part).map_err(|_| JwtError::InvalidClaims)?))
}

fn decode_part<T: DeserializeOwned>(part: &str) -> Option<T> {
    serde_json::from_slice(&decode_base64_url(part).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    const CLAIMS: &str = r#"{"iss":"joe","exp":1300819380,"http://example.com/is_root":true}"#;
    const RFC_7515_HS256_KEY: &str = "AyM1SysPpbyDfgZld3umj1qzKObwVMkoqQ-EstJQLr_T-1qS0gZH75aKtMN3Yj0iPS4hcgUuTwjAzZr1Z9CAow";
    const RFC_7515_HS256_TOKEN: &str = "eyJ0eXAiOiJKV1QiLA0KICJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJqb2UiLA0KICJleHAiOjEzMDA4MTkzODAsDQogImh0dHA6Ly9leGFtcGxlLmNvbS9pc19yb290Ijp0cnVlfQ.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const RFC_7515_ES256_KEY: &str = r#"{"kty":"EC","crv":"P-256","x":"f83OJ3D2xF1Bg8vub9tLe1gHMzV76e8Tus9uPHvRVEU","y":"x_FEzRu9m36HLN_tue659LNpXW6pCyStikYjKIWI5a0"}"#;
    const RFC_7515_ES256_TOKEN: &str =
        "eyJhbGciOiJFUzI1NiJ9.eyJpc3MiOiJqb2UiLA0KICJleHAiOjEzMDA4MTkzODAsDQogImh0dHA6Ly9leGFtcGxlLmNvbS9pc19yb290Ijp0cnVlfQ.DtEhU3ljbEg8L38VWAfUAqOyKAM6-Xx-F4GawxaepmXFCgfTjDxw5djxLa8ISlSApmWQxfKTUJqPP3-Kg6NU1Q";

    #[test]
    fn test_encode() {
        let claims: Value = serde_json::from_str(CLAIMS).unwrap();

        assert_eq!(
            encode(&SigningKey::hs256(b"secret").unwrap(), None, &claims).unwrap(),
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJqb2UiLCJleHAiOjEzMDA4MTkzODAsImh0dHA6Ly9leGFtcGxlLmNvbS9pc19yb290Ijp0cnVlfQ.x2cs4hRCGTt26GSwzk9DHqnt1Qk6jN-s9OEB7EBTAQI"
        );

        let token = encode(&SigningKey::mock(), Some("key_id"), &claims).unwrap();
        let (header, _) = token.split_once('.').unwrap();

        assert_eq!(decode_base64_url(header).unwrap(), br#"{"typ":"JWT","alg":"ES256","kid":"key_id"}"#);
        assert_eq!(decode::<Value>(&VerifyingKey::mock(), &token).unwrap(), claims);
    }

    #[test]
    fn test_decode() {
        let claims: Value = serde_json::from_str(CLAIMS).unwrap();
        let hs256 = VerifyingKey::hs256(&decode_base64_url(RFC_7515_HS256_KEY).unwrap()).unwrap();
        let es256 = VerifyingKey::es256(RFC_7515_ES256_KEY).unwrap();
        let (message, signature) = RFC_7515_HS256_TOKEN.rsplit_once('.').unwrap();

        assert_eq!(decode::<Value>(&hs256, RFC_7515_HS256_TOKEN).unwrap(), claims);
        assert_eq!(decode::<Value>(&es256, RFC_7515_ES256_TOKEN).unwrap(), claims);
        assert_eq!(decode::<Value>(&VerifyingKey::hs256(b"secret").unwrap(), RFC_7515_HS256_TOKEN), Err(JwtError::InvalidSignature));
        assert_eq!(decode::<Value>(&VerifyingKey::mock(), RFC_7515_ES256_TOKEN), Err(JwtError::InvalidSignature));
        assert_eq!(decode::<Value>(&hs256, &format!("{message}x.{signature}")), Err(JwtError::InvalidSignature));
        assert_eq!(decode::<Value>(&hs256, &format!("{message}.")), Err(JwtError::InvalidSignature));
        assert_eq!(decode::<Value>(&es256, RFC_7515_HS256_TOKEN), Err(JwtError::InvalidToken));
        assert_eq!(decode::<Value>(&hs256, RFC_7515_ES256_TOKEN), Err(JwtError::InvalidToken));
        assert_eq!(decode::<Value>(&hs256, "eyJhbGciOiJub25lIn0.eyJpc3MiOiJqb2UifQ."), Err(JwtError::InvalidToken));
        assert_eq!(decode::<Value>(&hs256, message), Err(JwtError::InvalidToken));
        assert_eq!(decode::<Vec<String>>(&hs256, RFC_7515_HS256_TOKEN), Err(JwtError::InvalidClaims));
    }
}
