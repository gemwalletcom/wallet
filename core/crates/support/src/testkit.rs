use primitives::hex;
use std::collections::HashMap;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

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
    let signature = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap().chain_update(format!("{timestamp}.{body}")).finalize().into_bytes();
    HashMap::from([(TIMESTAMP_HEADER.to_string(), timestamp.to_string()), (SIGNATURE_HEADER.to_string(), format!("{SIGNATURE_PREFIX}{}", hex::encode(signature)))])
}
