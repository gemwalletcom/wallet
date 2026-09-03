use std::error::Error;

use crate::models::account::BitcoinAccount;
use crate::models::block::{BitcoinNodeInfo, Block};
use crate::models::fee::BitcoinFeeResult;
use crate::models::transaction::{AddressDetails, BitcoinTransactionBroadcastResult, BitcoinUTXO, Transaction};
use crate::rpc::target::BlockbookTarget;
use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainSimulation, ChainStaking, ChainToken, ChainTraits};
use gem_client::{Client, ClientExt};
use primitives::{BitcoinChain, chain::Chain};

#[derive(Debug)]
pub struct BitcoinClient<C: Client> {
    client: C,
    pub chain: BitcoinChain,
}

impl<C: Client> BitcoinClient<C> {
    pub fn new(client: C, chain: BitcoinChain) -> Self {
        Self { client, chain }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain.get_chain()
    }

    pub async fn get_block(&self, block_number: u64, page: usize) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(BlockbookTarget::GetBlock { height: block_number, page }).await?)
    }

    pub async fn get_address_details(&self, address: &str, limit: usize) -> Result<AddressDetails, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(BlockbookTarget::GetAddressTransactions {
                address: address.to_string(),
                page_size: limit,
            })
            .await?)
    }

    pub async fn get_transaction(&self, txid: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(BlockbookTarget::GetTransaction { hash: txid.to_string() }).await?)
    }

    pub async fn get_balance(&self, address: &str) -> Result<BitcoinAccount, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(BlockbookTarget::GetAddress { address: address.to_string() }).await?)
    }

    pub async fn get_node_info(&self) -> Result<BitcoinNodeInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(BlockbookTarget::GetNodeInfo).await?)
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<BitcoinTransactionBroadcastResult, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post(BlockbookTarget::SendTransaction, &data).await?)
    }

    pub async fn get_utxos(&self, address: &str) -> Result<Vec<BitcoinUTXO>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(BlockbookTarget::GetUtxos { address: address.to_string() }).await?)
    }

    pub async fn get_fee_priority(&self, blocks: i32) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get::<BitcoinFeeResult>(BlockbookTarget::EstimateFee { blocks }).await?.result)
    }
}

impl<C: Client> ChainStaking for BitcoinClient<C> {}

impl<C: Client> ChainPerpetual for BitcoinClient<C> {}

impl<C: Client> ChainAddressStatus for BitcoinClient<C> {}

impl<C: Client> ChainToken for BitcoinClient<C> {}

impl<C: Client> ChainSimulation for BitcoinClient<C> {}

impl<C: Client> ChainTraits for BitcoinClient<C> {}

impl<C: Client> chain_traits::ChainProvider for BitcoinClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        self.chain.get_chain()
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_client::{CONTENT_TYPE, ContentType};

    use super::*;

    #[tokio::test]
    async fn test_broadcast_transaction() {
        let client = BitcoinClient::new(
            MockClient::new().with_post_with_headers(|path, body, headers| {
                assert_eq!(path, "/api/v2/sendtx/");
                assert_eq!(body, br#""0100beef""#);
                assert_eq!(headers.get(CONTENT_TYPE).map(String::as_str), Some(ContentType::TextPlain.as_str()));
                Ok(br#"{"result":"txid"}"#.to_vec())
            }),
            BitcoinChain::Bitcoin,
        );

        let broadcast = client.broadcast_transaction("0100beef".to_string()).await.unwrap();

        assert_eq!(broadcast.result.as_deref(), Some("txid"));
    }
}
