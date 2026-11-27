use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

use async_trait::async_trait;
use gem_client::Client;
use primitives::time::unix_timestamp;
use primitives::{SECONDS_PER_DAY, SECONDS_PER_MINUTE};
use spark_wallet::{HeaderProvider, HeaderProviderError};

use super::client::PartnerClient;
use super::model::{PartnerToken, map_token};

const PARTNER_HEADER: &str = "x-partner-jwt";
const REFRESH_MARGIN: u64 = SECONDS_PER_DAY;
const RETRY_INTERVAL: u64 = SECONDS_PER_MINUTE;

#[derive(Default)]
struct State {
    token: RwLock<Option<PartnerToken>>,
    refresh_at: AtomicU64,
}

impl State {
    fn token(&self, now: u64) -> Option<String> {
        let token = self.token.read().unwrap_or_else(PoisonError::into_inner);
        token.as_ref().filter(|token| token.expires_at > now).map(|token| token.token.clone())
    }

    fn set_token(&self, token: PartnerToken) {
        self.refresh_at.fetch_max(token.expires_at.saturating_sub(REFRESH_MARGIN), Ordering::Relaxed);
        *self.token.write().unwrap_or_else(PoisonError::into_inner) = Some(token);
    }

    fn begin_refresh(&self, now: u64) -> bool {
        self.refresh_at.try_update(Ordering::Relaxed, Ordering::Relaxed, |at| (at <= now).then_some(now.saturating_add(RETRY_INTERVAL))).is_ok()
    }
}

pub struct PartnerHeaders<C: Client> {
    client: Option<Arc<PartnerClient<C>>>,
    state: Arc<State>,
}

impl<C: Client> PartnerHeaders<C> {
    pub fn new(client: C, key: Option<&str>) -> Self {
        Self {
            client: key.filter(|key| !key.is_empty()).map(|key| Arc::new(PartnerClient::new(client, key.to_string()))),
            state: Arc::default(),
        }
    }
}

#[async_trait]
impl<C: Client + 'static> HeaderProvider for PartnerHeaders<C> {
    async fn headers(&self) -> Result<HashMap<String, String>, HeaderProviderError> {
        let Some(client) = &self.client else {
            return Ok(HashMap::new());
        };
        let now = unix_timestamp();
        if self.state.begin_refresh(now) {
            let (client, state) = (client.clone(), self.state.clone());
            tokio::spawn(async move {
                if let Some(token) = client.get_token().await.ok().and_then(map_token) {
                    state.set_token(token);
                }
            });
        }
        Ok(self.state.token(now).map(|token| HashMap::from([(PARTNER_HEADER.to_string(), token)])).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_encoding::encode_base64_url;

    use super::*;

    #[test]
    fn test_state() {
        let state = State::default();

        assert!(state.begin_refresh(100));
        assert!(!state.begin_refresh(100 + RETRY_INTERVAL - 1));
        assert!(state.begin_refresh(100 + RETRY_INTERVAL));
        assert_eq!(state.token(100), None);

        let expires_at = 100 + 7 * SECONDS_PER_DAY;
        state.set_token(PartnerToken { token: "jwt".to_string(), expires_at });

        assert_eq!(state.token(expires_at - 1), Some("jwt".to_string()));
        assert_eq!(state.token(expires_at), None);
        assert!(!state.begin_refresh(expires_at - REFRESH_MARGIN - 1));
        assert!(state.begin_refresh(expires_at - REFRESH_MARGIN));
    }

    #[tokio::test]
    async fn test_headers_without_key() {
        assert!(PartnerHeaders::new(MockClient::new(), None).headers().await.unwrap().is_empty());
        assert!(PartnerHeaders::new(MockClient::new(), Some("")).headers().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_headers() {
        let jwt = format!("header.{}.signature", encode_base64_url(format!(r#"{{"exp":{}}}"#, unix_timestamp() + 7 * SECONDS_PER_DAY).as_bytes()));
        let body = format!(r#"{{"token":"{jwt}"}}"#).into_bytes();
        let client = MockClient::new().with_get_with_headers(move |path, headers| {
            assert_eq!(path, "/api/jwt");
            assert_eq!(headers.get("Authorization").map(String::as_str), Some("Bearer key"));
            Ok(body.clone())
        });
        let headers = PartnerHeaders::new(client, Some("key"));

        assert!(headers.headers().await.unwrap().is_empty());
        while headers.state.token(unix_timestamp()).is_none() {
            tokio::task::yield_now().await;
        }

        assert_eq!(headers.headers().await.unwrap(), HashMap::from([(PARTNER_HEADER.to_string(), jwt)]));
    }
}
