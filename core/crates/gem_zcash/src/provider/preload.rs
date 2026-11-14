use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainTransactionLoad, TransactionFeeOperation};
use gem_bitcoin::{provider::preload_mapper::map_utxos, signer::load_fee};
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{FeePriority, FeeRate, GasPriceType, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use super::ZcashProvider;
use crate::signer::Plan;

const FEE_RATE: u64 = 10_000;

#[async_trait]
impl<C: Client> ChainTransactionLoad for ZcashProvider<C> {
    fn transaction_fee_estimate_units(&self, _operation: TransactionFeeOperation) -> Option<u64> {
        None
    }

    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let (utxos, node_info) = futures::try_join!(self.client.get_utxos(&input.sender_address), self.client.get_node_info())?;
        let branch_id = node_info.backend.consensus.ok_or("Branch ID not found")?.nextblock;
        Ok(TransactionLoadMetadata::Zcash {
            utxos: map_utxos(utxos, input.sender_address),
            branch_id,
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let fee = load_fee(&input, Plan::fee)?;
        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(FEE_RATE)))])
    }

    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        ChainTransactionLoad::get_utxos(&self.client, address).await
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::{BitcoinClient, testkit::signer_mock::mock_transfer_input};
    use gem_client::testkit::MockClient;
    use primitives::BitcoinChain;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_load() {
        let provider = ZcashProvider::new(BitcoinClient::new(MockClient::new(), BitcoinChain::Zcash));
        let load = provider.get_transaction_load(mock_transfer_input(BitcoinChain::Zcash).input).await.unwrap();

        assert_eq!(load.fee.fee, BigInt::from(10_000u64));
    }
}
