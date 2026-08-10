use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use futures::try_join;
use gem_client::Client;
use primitives::{FeeRate, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{
    provider::{
        preload_mapper::{address_to_public_key, map_transaction_fee, map_transaction_preload},
        state_mapper::map_gas_price_to_priorities,
    },
    rpc::NearProvider,
};

#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for NearProvider<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let public_key = address_to_public_key(&input.sender_address)?;
        let (access_key, block) = try_join!(self.get_account_access_key(&input.sender_address, &public_key), self.get_latest_block(),)?;
        Ok(map_transaction_preload(&access_key, &block))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let protocol_config = self.get_protocol_config().await?;
        Ok(TransactionLoadData {
            fee: map_transaction_fee(&input, &protocol_config),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        map_gas_price_to_priorities(&gas_price)
    }
}
