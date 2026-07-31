use async_trait::async_trait;
use chain_traits::ChainToken;
use std::error::Error;

use gem_client::Client;
use primitives::Asset;

use crate::rpc::{TronClient, TronProvider};

#[async_trait]
impl<C: Client> ChainToken for TronProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        TronClient::get_token_data(self, token_id).await
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.starts_with("T") && token_id.len() >= 30
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use gem_client::{ClientError, testkit::MockClient};

    use super::*;

    #[tokio::test]
    async fn test_get_token_data_forwards_to_client() {
        let called = Arc::new(AtomicBool::new(false));
        let handler_called = called.clone();
        let client = MockClient::new().with_post(move |_, _| {
            handler_called.store(true, Ordering::Relaxed);
            Err(ClientError::Http { status: 503, body: Vec::new() })
        });
        let provider = TronProvider::new_rpc_only(TronClient::new(client));

        let result = ChainToken::get_token_data(&provider, "token".to_string()).await;

        assert!(result.is_err());
        assert!(called.load(Ordering::Relaxed));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_USDT_TOKEN_ID, create_test_client};

    #[tokio::test]
    async fn test_get_token_data() {
        let tron_client = create_test_client();

        let asset = ChainToken::get_token_data(&tron_client, TEST_USDT_TOKEN_ID.to_string()).await.unwrap();

        assert_eq!(asset.symbol, "USDT");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.id.token_id, Some(TEST_USDT_TOKEN_ID.to_string()));
    }

    #[tokio::test]
    async fn test_get_is_token_address() {
        let tron_client = create_test_client();

        assert!(tron_client.get_is_token_address(TEST_USDT_TOKEN_ID));
        assert!(!tron_client.get_is_token_address("TLyqzVGLV1srkB7dToTAEqgDSfPtXRJZYH"));
        assert!(!tron_client.get_is_token_address("invalid"));
    }
}
