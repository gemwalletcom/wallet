use gem_encoding::decode_base64_url;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PartnerToken {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Deserialize)]
pub(super) struct PartnerTokenResponse {
    pub token: String,
}

#[derive(Deserialize)]
struct PartnerTokenClaims {
    exp: u64,
}

pub(super) fn map_token(response: PartnerTokenResponse) -> Option<PartnerToken> {
    let claims = decode_base64_url(response.token.split('.').nth(1)?).ok()?;
    let expires_at = serde_json::from_slice::<PartnerTokenClaims>(&claims).ok()?.exp;
    Some(PartnerToken { token: response.token, expires_at })
}

#[cfg(test)]
mod tests {
    use gem_encoding::encode_base64_url;

    use super::*;

    fn response(token: String) -> PartnerTokenResponse {
        PartnerTokenResponse { token }
    }

    #[test]
    fn test_map_token() {
        let token = format!("header.{}.signature", encode_base64_url(br#"{"sub":"Gem","exp":1800000000}"#));

        assert_eq!(map_token(response(token.clone())), Some(PartnerToken { token, expires_at: 1_800_000_000 }));
        assert_eq!(map_token(response("header.signature".to_string())), None);
        assert_eq!(map_token(response(format!("header.{}.signature", encode_base64_url(b"{}")))), None);
    }
}
