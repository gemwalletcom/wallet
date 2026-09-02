use crate::rpc::provider::TronAccountProvider;
use crate::rpc::trongrid::{
    mapper::TronGridMapper,
    model::{Data, TronGridAccount, TronGridTransaction},
};
use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};
use gem_client::{Client, ClientExt};
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

    pub async fn get_transactions(&self, address: &str, limit: usize) -> Result<Data<Vec<TronGridTransaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions?limit={}", address, limit);
        Ok(self.client.get_with_headers(path, self.headers()).await?)
    }

    pub async fn get_trc20_transactions(&self, address: &str, limit: usize) -> Result<Data<Vec<TronGridTransaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions/trc20?limit={}", address, limit);
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
        let (transactions, trc20_transactions) = futures::try_join!(self.get_transactions(&address, limit), self.get_trc20_transactions(&address, limit))?;
        Ok(TransactionsResult::TransactionRequests(TronGridMapper::map_transaction_requests(
            transactions.data,
            trc20_transactions.data,
            limit,
        )))
    }
}

#[async_trait]
impl<C: Client> TronAccountProvider for TronGridClient<C> {
    async fn get_accounts_by_address(&self, address: &str) -> Result<Vec<TronGridAccount>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_accounts(address).await?.data)
    }
}
