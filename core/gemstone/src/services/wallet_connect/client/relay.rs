use std::sync::Arc;

use chrono::Utc;
use gem_encoding::encode_hex;
use primitives::Platform;
use wallet_connect_protocol::crypto::{self, Key};
use wallet_connect_protocol::relay;

use super::model::GemWalletConnectRelay;
use crate::config::wallet_connect::get_wallet_connect_config;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemSecureStore;

const RELAY_KEY: &str = "wallet_connect_relay_key";
const RELAY_URL: &str = "wss://relay.walletconnect.org";
const RELAY_ORIGIN: &str = "gemwallet.com";

#[uniffi::export]
pub fn wallet_connect_relay(store: Arc<dyn GemSecureStore>, platform: Platform) -> Result<GemWalletConnectRelay, GemServiceError> {
    let seed = relay_seed(store.as_ref())?;
    let subject = encode_hex(crypto::generate_key()?.as_slice());
    let issued_at = u64::try_from(Utc::now().timestamp()).unwrap_or_default();
    let token = relay::auth_token(&seed, RELAY_URL, &subject, issued_at);
    Ok(GemWalletConnectRelay {
        url: format!("{RELAY_URL}/?projectId={}&auth={token}&ua=wc-2/gem/{}", get_wallet_connect_config().project_id, platform.as_str()),
        origin: RELAY_ORIGIN.to_string(),
    })
}

fn relay_seed(store: &dyn GemSecureStore) -> Result<Key, GemServiceError> {
    if let Some(seed) = store.get(RELAY_KEY.to_string())?.and_then(|value| crypto::parse_key(&value).ok()) {
        return Ok(seed);
    }
    let seed = crypto::generate_key()?;
    store.set(RELAY_KEY.to_string(), encode_hex(seed.as_slice()))?;
    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::preferences::testkit::MemorySecureStore;
    use gem_encoding::decode_base64_url;
    use serde_json::Value;

    #[test]
    fn test_wallet_connect_relay() {
        let store = Arc::new(MemorySecureStore::default());
        let claims = |url: &str| -> Value {
            let token = url.split("auth=").nth(1).unwrap().split('&').next().unwrap();
            serde_json::from_slice(&decode_base64_url(token.split('.').nth(1).unwrap()).unwrap()).unwrap()
        };

        let first = wallet_connect_relay(store.clone(), Platform::Android).unwrap();
        let second = wallet_connect_relay(store.clone(), Platform::IOS).unwrap();

        assert_eq!(first.origin, "gemwallet.com");
        assert!(first.url.starts_with("wss://relay.walletconnect.org/?projectId=3bc07cd7179d11ea65335fb9377702b6&auth="));
        assert!(first.url.ends_with("&ua=wc-2/gem/android"));
        assert!(second.url.ends_with("&ua=wc-2/gem/ios"));
        assert_eq!(claims(&first.url)["iss"], claims(&second.url)["iss"], "the relay identity is kept between connections");
        assert_ne!(claims(&first.url)["sub"], claims(&second.url)["sub"]);
        assert_eq!(claims(&first.url)["aud"], "wss://relay.walletconnect.org");

        store.set(RELAY_KEY.to_string(), "not a key".to_string()).unwrap();
        assert_ne!(
            claims(&wallet_connect_relay(store, Platform::IOS).unwrap().url)["iss"],
            claims(&first.url)["iss"],
            "a damaged relay key is replaced, it only names this install to the relay"
        );
    }
}
