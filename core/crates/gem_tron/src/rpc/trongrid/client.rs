use crate::models::Transaction;
use crate::rpc::provider::TronAccountProvider;
use crate::rpc::trongrid::model::{Data, TronGridAccount};
use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use gem_client::{Client, ClientExt};
use primitives::Chain;
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;

#[derive(Clone)]
pub struct TronGridClient<C: Client> {
    client: C,
    api_key: String,
}

impl<C: Client> TronGridClient<C> {
    pub fn new(client: C, api_key: String) -> Self {
        Self { client, api_key }
    }

    fn headers(&self) -> HashMap<String, String> {
        if self.api_key.is_empty() {
            HashMap::new()
        } else {
            let mut headers = HashMap::new();
            headers.insert("TRON-PRO-API-KEY".to_string(), self.api_key.clone());
            headers
        }
    }

    pub async fn get_transactions(&self, address: &str, limit: usize) -> Result<Data<Vec<Transaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions?limit={}", address, limit);
        Ok(self.client.get_with_headers(path, self.headers()).await?)
    }

    pub async fn get_accounts(&self, address: &str) -> Result<Data<Vec<TronGridAccount>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}", address);
        Ok(self.client.get_with_headers(path, self.headers()).await?)
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for TronGridClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let transactions = self.get_transactions(&address, limit).await?.data;
        Ok(TransactionsResult::TransactionRequests(
            transactions
                .into_iter()
                .map(|transaction| TransactionIdRequest::new(Chain::Tron, transaction.transaction_id, None))
                .collect(),
        ))
    }
}

#[async_trait]
impl<C: Client> TronAccountProvider for TronGridClient<C> {
    async fn get_accounts_by_address(&self, address: &str) -> Result<Vec<TronGridAccount>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_accounts(address).await?.data)
    }
}
