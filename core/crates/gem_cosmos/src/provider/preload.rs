use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use gem_client::Client;
use primitives::{FeeRate, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{
    provider::{preload_mapper::calculate_transaction_fee, state_mapper::calculate_fee_rates},
    rpc::client::CosmosClient,
};

#[async_trait]
impl<C: Client> ChainTransactionLoad for CosmosClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info(&input.sender_address).await?;
        Ok(TransactionLoadMetadata::Cosmos {
            account_number: account.account_number,
            sequence: account.sequence,
            chain_id: self.get_chain().as_chain().network_id().to_string(),
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info(&input.sender_address).await?;
        let fee = calculate_transaction_fee(&input.input_type, self.get_chain(), &input.gas_price)?;

        Ok(TransactionLoadData {
            fee,
            metadata: TransactionLoadMetadata::Cosmos {
                account_number: account.account_number,
                sequence: account.sequence,
                chain_id: self.get_chain().as_chain().network_id().to_string(),
            },
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(calculate_fee_rates(self.get_chain(), self.get_base_fee().await?))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use chain_traits::ChainTransactionLoad;
    use num_bigint::BigInt;
    use primitives::{Asset, TransactionInputType, chain_cosmos::CosmosChain};

    use crate::provider::testkit::create_test_client;

    #[tokio::test]
    async fn test_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for chain in [CosmosChain::Cosmos, CosmosChain::Osmosis, CosmosChain::Injective] {
            let input_type = TransactionInputType::Transfer { asset: Asset::from_chain(chain.as_chain()) };
            let rates = create_test_client(chain).get_transaction_fee_rates(input_type).await?;

            assert!(rates.iter().all(|rate| rate.gas_price_type.gas_price() > BigInt::ZERO));
        }

        Ok(())
    }
}
