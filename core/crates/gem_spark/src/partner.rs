use std::collections::HashMap;
use std::sync::{PoisonError, RwLock};

use async_trait::async_trait;
use primitives::time::unix_timestamp;
use primitives::{LightningToken, SECONDS_PER_DAY};
use spark_wallet::{HeaderProvider, HeaderProviderError};

const PARTNER_HEADER: &str = "x-partner-jwt";
const EXPIRY_MARGIN: u64 = SECONDS_PER_DAY;

#[derive(Default)]
pub struct PartnerHeaders(RwLock<Option<LightningToken>>);

impl PartnerHeaders {
    pub fn needs_token(&self) -> bool {
        self.token(unix_timestamp()).is_none()
    }

    pub fn set(&self, token: LightningToken) {
        *self.0.write().unwrap_or_else(PoisonError::into_inner) = Some(token);
    }

    fn token(&self, now: u64) -> Option<String> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .filter(|token| token.expires_at > now.saturating_add(EXPIRY_MARGIN))
            .map(|token| token.token.clone())
    }
}

#[async_trait]
impl HeaderProvider for PartnerHeaders {
    async fn headers(&self) -> Result<HashMap<String, String>, HeaderProviderError> {
        Ok(self.token(unix_timestamp()).map(|token| HashMap::from([(PARTNER_HEADER.to_string(), token)])).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token() {
        let headers = PartnerHeaders::default();
        assert_eq!(headers.token(0), None);

        headers.set(LightningToken {
            token: "jwt".to_string(),
            expires_at: 2 * SECONDS_PER_DAY,
        });

        assert_eq!(headers.token(SECONDS_PER_DAY - 1), Some("jwt".to_string()));
        assert_eq!(headers.token(SECONDS_PER_DAY), None);
    }
}
