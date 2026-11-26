use primitives::hex;
use std::collections::HashMap;

use gem_hash::sha2::hmac_sha256;

use crate::SupportMessageLink;
use crate::webhook::{SIGNATURE_HEADER, SIGNATURE_PREFIX, TIMESTAMP_HEADER};

impl SupportMessageLink {
    pub fn mock(title: &str, url: &str, subtitle: Option<&str>) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
            subtitle: subtitle.map(str::to_string),
        }
    }
}

pub fn chatwoot_webhook_headers(secret: &str, body: &str, timestamp: i64) -> HashMap<String, String> {
    let signature = hmac_sha256(secret.as_bytes(), format!("{timestamp}.{body}").as_bytes()).unwrap();
    HashMap::from([
        (TIMESTAMP_HEADER.to_string(), timestamp.to_string()),
        (SIGNATURE_HEADER.to_string(), format!("{SIGNATURE_PREFIX}{}", hex::encode(signature.as_slice()))),
    ])
}
