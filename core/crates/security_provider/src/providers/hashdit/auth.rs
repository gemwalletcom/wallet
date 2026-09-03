use std::collections::HashMap;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

pub(super) fn signed_headers(app_id: &str, app_secret: &str, timestamp: &str, nonce: &str, method: &str, path: &str, body: &str) -> HashMap<String, String> {
    HashMap::from([
        ("X-Signature-appid".to_string(), app_id.to_string()),
        ("X-Signature-signature".to_string(), sign(app_id, app_secret, timestamp, nonce, method, path, body)),
        ("X-Signature-timestamp".to_string(), timestamp.to_string()),
        ("X-Signature-nonce".to_string(), nonce.to_string()),
    ])
}

fn sign(app_id: &str, app_secret: &str, timestamp: &str, nonce: &str, method: &str, path: &str, body: &str) -> String {
    let message = match path.split_once('?') {
        Some((url, query)) => format!("{app_id};{timestamp};{nonce};{method};{url};{query};{body}"),
        None => format!("{app_id};{timestamp};{nonce};{method};{path};{body}"),
    };
    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
