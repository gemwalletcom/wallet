use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeeRate, GasPriceType, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::{provider::preload_mapper::map_transaction_preload, rpc::XrpClient};

#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for XrpClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        let destination = input.input_type.swap_to_address().or(Some(input.destination_address.as_str())).filter(|address| !address.is_empty());
        let (sender, destination_exists) = futures::try_join!(self.get_account_info_full(&input.sender_address), self.destination_exists(destination))?;
        map_transaction_preload(sender, destination_exists)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: input.default_fee(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let fees = self.get_fees().await?;
        let median_fee = fees.drops.median_fee;

        Ok(vec![
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(median_fee))),
            FeeRate::new(FeePriority::Fast, GasPriceType::regular(BigInt::from(median_fee * 2))),
        ])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_xrp_test_client};
    use primitives::{AccountDataType, Asset, Chain};

    #[tokio::test]
    async fn test_xrp_get_transaction_preload_activation() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Account {
                asset: Asset::from_chain(Chain::Xrp),
                account_type: AccountDataType::Activate,
            },
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: String::new(),
            references: vec![],
        };

        let metadata = client.get_transaction_preload(input).await?;

        assert!(metadata.get_sequence()? > 0);
        assert!(!metadata.get_is_destination_address_exist()?);
        Ok(())
    }
}
