use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use chain_traits::{ChainTransactionLoad, TransactionFeeOperation};
use gem_bitcoin::provider::preload_mapper::{map_utxos, preload_output_minimum};
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{
    AssetId, BitcoinChain, Chain, FeePriority, FeeRate, GasPriceType, SignerError, SignerInput, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput, UTXO,
};

use super::ZcashProvider;
use crate::{
    address::{Destination, ParseAddress},
    signer::Plan,
};

const FEE_RATE: u64 = 10_000;

#[async_trait]
impl<C: Client> ChainTransactionLoad for ZcashProvider<C> {
    fn transaction_fee_estimate_units(&self, _operation: TransactionFeeOperation) -> Option<u64> {
        None
    }

    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let utxos = self.client.get_utxos(&input.sender_address).await?;
        let node_info = self.client.get_node_info().await?;
        let branch_id = node_info.backend.consensus.ok_or("Branch ID not found")?.nextblock;
        let (destination, _) = Destination::parse(input.input_type.swap_to_address().unwrap_or(&input.destination_address))?.into_spend();
        let output_minimum = preload_output_minimum(BitcoinChain::Zcash, &input, &destination)?;
        Ok(TransactionLoadMetadata::Zcash {
            utxos: map_utxos(utxos, input.sender_address),
            branch_id,
            output_minimum,
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let fee = Self::transaction_fee(&input)?;
        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(FEE_RATE)))])
    }

    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        ChainTransactionLoad::get_utxos(&self.client, address).await
    }
}

impl<C: Client> ZcashProvider<C> {
    fn transaction_fee(input: &TransactionLoadInput) -> Result<TransactionFee, SignerError> {
        Ok(TransactionFee {
            fee: BigInt::from(Plan::fee(&SignerInput::new(input.clone(), input.default_fee()))?),
            gas_price_type: input.gas_price.clone(),
            gas_limit: BigInt::from(1u8),
            options: HashMap::new(),
            fee_asset: AssetId::from_chain(Chain::Zcash),
        })
    }
}

#[cfg(test)]
mod tests {
    use gem_bitcoin::{
        BitcoinClient,
        testkit::{address_mock::mock_sender_address, signer_mock::mock_transfer_input},
    };
    use gem_client::testkit::MockClient;
    use num_bigint::BigUint;
    use primitives::testkit::zcash_mock;

    use super::*;
    use crate::testkit::TEST_ZCASH_UNIFIED_ADDRESS;

    #[tokio::test]
    async fn test_get_transaction_load() {
        let provider = ZcashProvider::new(BitcoinClient::new(MockClient::new(), BitcoinChain::Zcash));
        let load = provider.get_transaction_load(mock_transfer_input(BitcoinChain::Zcash).input).await.unwrap();

        assert_eq!(load.fee.fee, BigInt::from(10_000u64));

        let mut shielded = zcash_mock::mock_signer_input(mock_sender_address(BitcoinChain::Zcash), TEST_ZCASH_UNIFIED_ADDRESS.to_string()).input;
        shielded.value = BigUint::from(60_000u64);
        let load = provider.get_transaction_load(shielded).await.unwrap();

        assert_eq!(load.fee.fee, BigInt::from(15_000u64), "a send above the balance is priced and left to the amount check");
    }
}
