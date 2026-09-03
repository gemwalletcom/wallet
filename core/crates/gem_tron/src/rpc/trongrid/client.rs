use crate::rpc::provider::TronAccountProvider;
use crate::rpc::trongrid::{
    mapper::TronGridMapper,
    model::{Data, TronGridAccount, TronGridTransaction},
    target::TronGridTarget,
};
use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};
use gem_client::{Client, ClientExt};
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;

const TRANSACTIONS_PAGE_SIZE: usize = 200;
type TransactionsTarget = fn(String, usize, Option<String>) -> TronGridTarget;

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

    async fn get_transaction_page(&self, target: TronGridTarget) -> Result<Data<Vec<TronGridTransaction>>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&target.path()).headers(self.headers()).await?)
    }

    async fn get_transaction_pages(
        &self,
        address: &str,
        limit: usize,
        page_size: usize,
        target: TransactionsTarget,
    ) -> Result<Vec<TronGridTransaction>, Box<dyn Error + Send + Sync>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut transactions = Vec::with_capacity(limit);
        let mut fingerprint = None;
        let max_pages = limit.div_ceil(page_size);

        for _ in 0..max_pages {
            let page_limit = (limit - transactions.len()).min(page_size);
            let page = self.get_transaction_page(target(address.to_string(), page_limit, fingerprint.clone())).await?;
            let is_empty = page.data.is_empty();

            transactions.extend(page.data);
            if transactions.len() >= limit || is_empty {
                break;
            }

            let Some(next_fingerprint) = page.meta.and_then(|meta| meta.fingerprint) else {
                break;
            };
            fingerprint = Some(next_fingerprint);
        }

        transactions.truncate(limit);
        Ok(transactions)
    }

    pub async fn get_transactions(&self, address: &str, limit: usize) -> Result<Vec<TronGridTransaction>, Box<dyn Error + Send + Sync>> {
        self.get_transaction_pages(address, limit, TRANSACTIONS_PAGE_SIZE, TronGridTarget::GetTransactions).await
    }

    pub async fn get_trc20_transactions(&self, address: &str, limit: usize) -> Result<Vec<TronGridTransaction>, Box<dyn Error + Send + Sync>> {
        self.get_transaction_pages(address, limit, TRANSACTIONS_PAGE_SIZE, TronGridTarget::GetTrc20Transactions)
            .await
    }

    pub async fn get_accounts(&self, address: &str) -> Result<Data<Vec<TronGridAccount>>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&TronGridTarget::GetAccount(address.to_string()).path()).headers(self.headers()).await?)
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for TronGridClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let (transactions, trc20_transactions) = futures::try_join!(self.get_transactions(&address, limit), self.get_trc20_transactions(&address, limit))?;
        Ok(TransactionsResult::TransactionRequests(TronGridMapper::map_transaction_requests(
            transactions,
            trc20_transactions,
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gem_client::testkit::MockClient;

    use super::*;

    const TRANSACTIONS_RESPONSE: &str = include_str!("../../../testdata/transactions_by_address.json");
    const TRANSACTIONS_PAGE_2_RESPONSE: &str = include_str!("../../../testdata/transactions_by_address_page_2.json");

    #[tokio::test]
    async fn test_get_transaction_pages() {
        let paths = Arc::new(Mutex::new(Vec::new()));
        let paths_handler = paths.clone();
        let client = TronGridClient::new(
            MockClient::new().with_get(move |path| {
                paths_handler.lock().unwrap().push(path.to_string());
                if path.contains("fingerprint=") {
                    Ok(TRANSACTIONS_PAGE_2_RESPONSE.as_bytes().to_vec())
                } else {
                    Ok(TRANSACTIONS_RESPONSE.as_bytes().to_vec())
                }
            }),
            String::new(),
        );

        let transactions = client.get_transaction_pages("address", 6, 4, TronGridTarget::GetTransactions).await.unwrap();

        assert_eq!(transactions.len(), 6);
        assert_eq!(transactions.last().unwrap().transaction_id, "page-two-transaction-2");
        assert_eq!(
            *paths.lock().unwrap(),
            vec![
                "/v1/accounts/address/transactions?limit=4",
                "/v1/accounts/address/transactions?limit=2&fingerprint=2NgPQPX6mkxX1nyEcdNgHL1S2oc6C2YNniHptY2Nbq9Ja5fetDtG2WmdZHJ6LSz5JcuhU5ofSuCDwavxdAmdx4HY3kqMowHJwVn1V9kHL4MLPEgGQSNYpmjSu9z6tWbuTpaLashp8XLgJgxbyK1kTinb6THXLug265TJGCo9LU4VbEDG8kbG9AgpSsQqcSm3AxRhcu56RqnsdVDTTM4gTjJ1uRc",
            ]
        );
    }
}
