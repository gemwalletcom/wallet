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
        self.api_token
            .as_ref()
            .map(|token| HashMap::from([(String::from("Authorization"), format!("Bearer {token}"))]))
            .unwrap_or_default()
    }

    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponseResult, SwapperError> {
        self.client
            .post(NearIntentsTarget::Quote, request)
            .headers(self.build_headers())
            .await
            .map_err(SwapperError::from)
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
        let transactions: Vec<ExplorerTransaction> = self
            .client
            .get(NearIntentsExplorerTarget::Transactions {
                query: ExplorerTransactionsQuery {
                    search: hash.to_string(),
                    number_of_transactions: TRANSACTIONS_SEARCH_LIMIT,
                },
            })
            .await
            .map_err(SwapperError::from)?;
        Ok(transactions.into_iter().find(|tx| tx.origin_chain_tx_hashes.iter().any(|h| h.eq_ignore_ascii_case(hash))))
    }
}
