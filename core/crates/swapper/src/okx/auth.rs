use super::model::OkxClientConfig;
use gem_encoding::encode_base64;
use gem_hash::sha2::hmac_sha256;
use std::collections::HashMap;

pub(super) const HEADER_KEY: &str = "OK-ACCESS-KEY";
pub(super) const HEADER_SIGN: &str = "OK-ACCESS-SIGN";
pub(super) const HEADER_TIMESTAMP: &str = "OK-ACCESS-TIMESTAMP";
pub(super) const HEADER_PASSPHRASE: &str = "OK-ACCESS-PASSPHRASE";
pub(super) const HEADER_PROJECT: &str = "OK-ACCESS-PROJECT";

pub(super) fn sign(timestamp: &str, method: &str, path: &str, secret_key: &str) -> String {
    let signature = hmac_sha256(secret_key.as_bytes(), format!("{timestamp}{method}{path}").as_bytes()).expect("HMAC accepts any key length");
    encode_base64(signature.as_slice())
}

pub(super) fn build_headers(config: &OkxClientConfig, timestamp: &str, full_path: &str) -> HashMap<String, String> {
    HashMap::from([
        (HEADER_KEY.to_string(), config.api_key.clone()),
        (HEADER_SIGN.to_string(), sign(timestamp, "GET", full_path, &config.secret_key)),
        (HEADER_TIMESTAMP.to_string(), timestamp.to_string()),
        (HEADER_PASSPHRASE.to_string(), config.passphrase.clone()),
        (HEADER_PROJECT.to_string(), config.project.clone()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        let s = sign("2024-01-01T00:00:00.000Z", "GET", "/api/v6/dex/aggregator/quote", "test_secret");
        assert_eq!(s, sign("2024-01-01T00:00:00.000Z", "GET", "/api/v6/dex/aggregator/quote", "test_secret"));
        assert_eq!(s, "8fQD4JACttJeAVCwsV0KKm92A3g06DzBKllawdpMlGg=");
        assert_ne!(s, sign("2024-01-01T00:00:00.001Z", "GET", "/api/v6/dex/aggregator/quote", "test_secret"));
        assert_ne!(s, sign("2024-01-01T00:00:00.000Z", "POST", "/api/v6/dex/aggregator/quote", "test_secret"));
        assert_ne!(s, sign("2024-01-01T00:00:00.000Z", "GET", "/api/v6/dex/aggregator/swap", "test_secret"));
        assert_ne!(s, sign("2024-01-01T00:00:00.000Z", "GET", "/api/v6/dex/aggregator/quote", "other_secret"));
        assert_ne!(sign("ts", "GET", "/path", "secret"), sign("ts/path", "GET", "", "secret"));
    }

    #[test]
    fn test_build_headers() {
        let config = OkxClientConfig {
            api_key: "key".to_string(),
            secret_key: "secret".to_string(),
            passphrase: "pass".to_string(),
            project: "proj".to_string(),
        };
        let headers = build_headers(&config, "2024-01-01T00:00:00.000Z", "/api/v6/dex/aggregator/quote?a=1");
        assert_eq!(headers.get(HEADER_KEY).unwrap(), "key");
        assert_eq!(headers.get(HEADER_TIMESTAMP).unwrap(), "2024-01-01T00:00:00.000Z");
        assert_eq!(headers.get(HEADER_PASSPHRASE).unwrap(), "pass");
        assert_eq!(headers.get(HEADER_PROJECT).unwrap(), "proj");
        assert!(!headers.get(HEADER_SIGN).unwrap().is_empty());
    }
}
