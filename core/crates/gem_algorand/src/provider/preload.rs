use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{AssetId, Chain, FeeRate, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{provider::state_mapper::map_transaction_params_to_fee, rpc::AlgorandProvider};

#[async_trait]
impl<C: Client> ChainTransactionLoad for AlgorandProvider<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, _input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let params = self.get_transactions_params().await?;
        let metadata = TransactionLoadMetadata::Algorand {
            sequence: params.last_round,
            block_hash: params.genesis_hash,
            chain_id: params.genesis_id,
        };

        Ok(TransactionLoadData {
            fee: TransactionFee::new_from_fee(BigInt::from(params.min_fee), AssetId::from_chain(Chain::Algorand)),
            metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![map_transaction_params_to_fee(&self.get_transactions_params().await?)])
    }
}
