use std::error::Error;

use crate::models::account::Account;
use crate::models::common::{Embedded, StellarAsset, StellarEmbedded};
use crate::models::fee::StellarFees;
use crate::models::node::NodeStatus;
use crate::models::transaction::{Payment, PaymentsQuery, StellarTransactionBroadcast, StellarTransactionStatus};
use crate::models::{AccountEmpty, AccountResult};
use crate::rpc::target::HorizonTarget;

use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits};
use gem_client::{Client, ClientError, ClientExt};
use primitives::Chain;
use serde::de::DeserializeOwned;

use crate::provider::transactions_mapper::encode_transaction_data;

const PAGE_LIMIT: usize = 200;

#[derive(Debug)]
pub struct StellarClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> StellarClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Stellar }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_or_not_found<R: DeserializeOwned + Send>(&self, target: HorizonTarget) -> Result<AccountResult<R>, ClientError> {
        match self.client.get(target).await {
            Ok(value) => Ok(AccountResult::Found(value)),
            Err(ClientError::Http { status: 404, .. }) => Ok(AccountResult::NotFound),
            Err(error) => Err(error),
        }
    }

    pub async fn get_node_status(&self) -> Result<NodeStatus, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(HorizonTarget::GetNodeStatus).await?)
    }

    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<StellarTransactionStatus, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(HorizonTarget::GetTransaction { hash: transaction_id.to_string() }).await?)
    }

    pub async fn get_fees(&self) -> Result<StellarFees, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(HorizonTarget::GetFees).await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<StellarTransactionBroadcast, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(HorizonTarget::SubmitTransaction, &encode_transaction_data(data)).await?)
    }

    pub async fn get_assets_by_issuer(&self, issuer: &str) -> Result<StellarEmbedded<StellarAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(HorizonTarget::GetAssets {
                issuer: issuer.to_string(),
                limit: PAGE_LIMIT,
            })
            .await?)
    }

    pub async fn get_account(&self, account_id: String) -> Result<AccountResult<Account>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_or_not_found(HorizonTarget::GetAccount { address: account_id }).await?)
    }

    pub async fn account_exists(&self, address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let account = self.get_or_not_found::<AccountEmpty>(HorizonTarget::GetAccount { address: address.to_string() }).await?;
        Ok(match account {
            AccountResult::Found(account) => account.id.is_some(),
            AccountResult::NotFound => false,
        })
    }

    pub async fn get_account_payments(&self, account_id: String) -> Result<AccountResult<Embedded<Payment>>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_or_not_found(HorizonTarget::GetAccountPayments {
                address: account_id,
                query: PaymentsQuery::latest(PAGE_LIMIT),
            })
            .await?)
    }

    pub async fn get_transaction_payments(&self, transaction_id: &str) -> Result<AccountResult<Embedded<Payment>>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_or_not_found(HorizonTarget::GetTransactionPayments {
                hash: transaction_id.to_string(),
                query: PaymentsQuery::default(),
            })
            .await?)
    }

    pub async fn get_block_payments(&self, block_number: u64, limit: usize, cursor: Option<String>) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get::<Embedded<Payment>>(HorizonTarget::GetLedgerPayments {
                ledger: block_number,
                query: PaymentsQuery::page(limit, cursor),
            })
            .await?
            ._embedded
            .records)
    }

    pub async fn get_block_payments_all(&self, block_number: u64) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        let mut results: Vec<Payment> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let payments = self.get_block_payments(block_number, PAGE_LIMIT, cursor).await?;
            results.extend(payments.clone());
            cursor = payments.last().map(|x| x.id.clone());

            if payments.len() < PAGE_LIMIT {
                return Ok(results);
            }
        }
    }
}

impl<C: Client> ChainStaking for StellarClient<C> {}

impl<C: Client> ChainPerpetual for StellarClient<C> {}

impl<C: Client> ChainAddressStatus for StellarClient<C> {}

impl<C: Client> chain_traits::ChainAccount for StellarClient<C> {}

impl<C: Client> ChainSimulation for StellarClient<C> {}

impl<C: Client> ChainTraits for StellarClient<C> {}

impl<C: Client> ChainProvider for StellarClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        self.chain
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_client::{CONTENT_TYPE, ContentType};

    use super::*;

    #[tokio::test]
    async fn test_broadcast_transaction() {
        let client = StellarClient::new(MockClient::new().with_post_with_headers(|path, body, headers| {
            assert_eq!(path, "/transactions_async");
            assert_eq!(body, b"tx=AAAA%2B%2F%3D%3D");
            assert_eq!(headers.get(CONTENT_TYPE).map(String::as_str), Some(ContentType::ApplicationFormUrlEncoded.as_str()));
            Ok(br#"{"hash":"abc","tx_status":"PENDING"}"#.to_vec())
        }));

        let broadcast = client.broadcast_transaction("AAAA+/==").await.unwrap();

        assert_eq!(broadcast.hash.as_deref(), Some("abc"));
    }

    #[tokio::test]
    async fn test_account_exists() {
        let client = StellarClient::new(MockClient::new().with_get(|path| match path {
            "/accounts/GFUNDED" => Ok(br#"{"id":"GFUNDED"}"#.to_vec()),
            _ => Err(ClientError::Http { status: 404, body: Vec::new() }),
        }));

        assert!(client.account_exists("GFUNDED").await.unwrap());
        assert!(!client.account_exists("GEMPTY").await.unwrap());
    }
}
