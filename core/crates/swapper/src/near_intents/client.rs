use crate::{SwapperError, config::get_swap_proxy_url};
use gem_client::{Client, ClientExt};
use std::{collections::HashMap, fmt::Debug};

use super::model::{ExplorerTransaction, ExplorerTransactionsQuery, QuoteRequest, QuoteResponseResult};
use super::target::{NearIntentsExplorerTarget, NearIntentsTarget};

const TRANSACTIONS_SEARCH_LIMIT: usize = 10;

pub fn base_url() -> String {
    get_swap_proxy_url("near-intents/1click")
}

pub fn explorer_url() -> String {
    get_swap_proxy_url("near-intents/explorer")
}

#[derive(Clone, Debug)]
pub struct NearIntentsClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    client: C,
    api_token: Option<String>,
}

impl<C> NearIntentsClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(client: C, api_key: Option<String>) -> Self {
        Self { client, api_token: api_key }
    }

    fn build_headers(&self) -> HashMap<String, String> {
        self.api_token.as_ref().map(|token| HashMap::from([(String::from("Authorization"), format!("Bearer {token}"))])).unwrap_or_default()
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponseResult, SwapperError> {
        self.client.post(NearIntentsTarget::Quote, request).headers(self.build_headers()).await.map_err(SwapperError::from)
    }
}

#[derive(Debug)]
pub struct NearIntentsExplorer<C: Client> {
    client: C,
}

impl<C: Client + Send + Sync + Debug> NearIntentsExplorer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn search_transaction(&self, hash: &str) -> Result<Option<ExplorerTransaction>, SwapperError> {
        Ok(self.search(hash).await?.into_iter().find(|transaction| transaction.origin_chain_tx_hashes.iter().any(|h| h.eq_ignore_ascii_case(hash))))
    }

    pub async fn search_deposit(&self, deposit_address: &str, deposit_memo: Option<&str>) -> Result<Option<ExplorerTransaction>, SwapperError> {
        Ok(self
            .search(deposit_address)
            .await?
            .into_iter()
            .find(|transaction| transaction.deposit_address.eq_ignore_ascii_case(deposit_address) && transaction.deposit_memo.as_deref() == deposit_memo))
    }

    async fn search(&self, query: &str) -> Result<Vec<ExplorerTransaction>, SwapperError> {
        self.client
            .get(NearIntentsExplorerTarget::Transactions {
                query: ExplorerTransactionsQuery {
                    search: query.to_string(),
                    number_of_transactions: TRANSACTIONS_SEARCH_LIMIT,
                },
            })
            .await
            .map_err(SwapperError::from)
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    const DEPOSIT_ADDRESS: &str = "0x5d4A8B7d32f42C6a1aFC0a55867ad17C2278e86C";

    fn explorer() -> NearIntentsExplorer<MockClient> {
        NearIntentsExplorer::new(MockClient::new().with_get(|_| Ok(include_bytes!("testdata/tx_status_polygon_usdc_to_litecoin.json").to_vec())))
    }

    #[tokio::test]
    async fn test_search_deposit() {
        let explorer = explorer();

        assert_eq!(explorer.search_deposit(&DEPOSIT_ADDRESS.to_lowercase(), None).await.unwrap().unwrap().deposit_address, DEPOSIT_ADDRESS);
        assert!(explorer.search_deposit(DEPOSIT_ADDRESS, Some("memo")).await.unwrap().is_none());
        assert!(explorer.search_deposit("0x0000000000000000000000000000000000000000", None).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_search_transaction_requires_origin_hash() {
        let explorer = explorer();

        assert!(explorer.search_transaction("0x6a083d43a80f236ba5952dfdccaf99e3935a1dadef7d9f55e64857cccd2ac408").await.unwrap().is_some());
        assert!(explorer.search_transaction(DEPOSIT_ADDRESS).await.unwrap().is_none());
    }
}
