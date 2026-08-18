use std::error::Error;

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainBalances, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainState, ChainToken, ChainTraits,
    ChainTransaction, ChainTransactionBroadcast, ChainTransactionLoad, ChainTransactionState, ChainTransactions, TransactionFeeEstimate, TransactionFeeEstimates,
    TransactionFeeOperation, TransactionIdRequest, TransactionsRequest, TransactionsResult,
};
use gem_client::Client;
use gem_evm::provider::transaction_state_mapper::map_transaction_status_with_fee;
use gem_evm::rpc::mapper::EthereumMapper;
use gem_evm::rpc::{EthereumClient, EthereumProvider, EvmProviderExtensions};
use primitives::{
    Asset, AssetBalance, BroadcastOptions, Chain, FeeRate, SimulationInput, SimulationResult, Transaction, TransactionInputType, TransactionLoadData, TransactionLoadInput,
    TransactionLoadMetadata, TransactionPreloadInput, TransactionState, TransactionStateRequest, TransactionUpdate, asset_constants::TEMPO_PATHUSD_ASSET_ID, fee::FeePriority,
};

use crate::{fee::scale_fee_to_token_units, fee_calculator::TempoFeeCalculator, mapper};

pub struct TempoProvider<C: Client + Clone> {
    provider: EthereumProvider<C>,
}

impl<C: Client + Clone + 'static> TempoProvider<C> {
    pub fn new(client: EthereumClient<C>) -> Self {
        let extensions = EvmProviderExtensions {
            fee_calculator: Some(Box::new(TempoFeeCalculator::new(client.clone()))),
            ..Default::default()
        };
        Self {
            provider: EthereumProvider::new_rpc_only_with_extensions(client, extensions),
        }
    }

    pub fn new_or_else(client: EthereumClient<C>, fallback: impl FnOnce(EthereumClient<C>) -> Box<dyn ChainTraits>) -> Box<dyn ChainTraits> {
        if client.get_chain() == Chain::Tempo {
            Box::new(Self::new(client))
        } else {
            fallback(client)
        }
    }
}

impl<C: Client + Clone> ChainProvider for TempoProvider<C> {
    fn get_chain(&self) -> Chain {
        self.provider.get_chain()
    }
}

#[async_trait]
impl<C: Client + Clone> ChainBalances for TempoProvider<C> {
    async fn get_balance_coin(&self, _address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        Ok(AssetBalance::new_zero_balance(Chain::Tempo.as_asset_id()))
    }

    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        self.provider.get_balance_tokens(address, token_ids).await
    }

    async fn get_balance_assets(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.provider.get_balance_assets(address).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for TempoProvider<C> {
    fn transaction_fee_estimate_units(&self, operation: TransactionFeeOperation) -> Option<u64> {
        self.provider.transaction_fee_estimate_units(match operation {
            TransactionFeeOperation::Transfer => TransactionFeeOperation::TokenTransfer,
            operation => operation,
        })
    }

    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        self.provider.get_transaction_preload(input).await
    }

    async fn get_transaction_fee_rates(&self, input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(self
            .provider
            .get_transaction_fee_rates(input_type)
            .await?
            .into_iter()
            .filter(|rate| rate.priority == FeePriority::Normal)
            .collect())
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        self.provider.map_transaction_load(input).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactionState for TempoProvider<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let Some(receipt) = self.provider.get_transaction_receipt(&request.id).await? else {
            return Ok(TransactionUpdate::new_state(TransactionState::Pending));
        };
        Ok(map_transaction_status_with_fee(&receipt, scale_fee_to_token_units(receipt.get_fee().into())))
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransaction for TempoProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let hash = request.hash.clone();
        let Some(transaction) = self.provider.get_transaction_by_hash(request).await? else {
            return Ok(None);
        };
        let Some(receipt) = self.provider.get_transaction_receipt(&hash).await? else {
            return Ok(None);
        };
        Ok(Some(mapper::map_transaction(transaction, &receipt)?))
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for TempoProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.provider.get_transactions_by_address(request).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainBlockTransactions for TempoProvider<C> {
    async fn get_transactions_by_block(&self, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block = self.provider.get_block(block_number).await?;
        if block.transactions.is_empty() {
            return Ok(Vec::new());
        }

        let receipts = self.provider.get_block_receipts(block_number).await?;
        block
            .transactions
            .into_iter()
            .zip(receipts)
            .filter_map(|(transaction, receipt)| EthereumMapper::map_transaction(Chain::Tempo, &transaction, &receipt, &block.timestamp, &[]).map(|mapped| (mapped, receipt)))
            .map(|(transaction, receipt)| mapper::map_transaction(transaction, &receipt))
            .collect()
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactionBroadcast for TempoProvider<C> {
    async fn transaction_broadcast(&self, data: String, options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.provider.transaction_broadcast(data, options).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainToken for TempoProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        self.provider.get_token_data(token_id).await
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        self.provider.get_is_token_address(token_id)
    }
}

#[async_trait]
impl<C: Client + Clone> ChainState for TempoProvider<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        ChainState::get_chain_id(&self.provider).await
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.provider.get_block_latest_number().await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainSimulation for TempoProvider<C> {
    async fn simulate_transaction(&self, input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        self.provider.simulate_transaction(input).await
    }
}

impl<C: Client + Clone> ChainStaking for TempoProvider<C> {}
impl<C: Client + Clone> ChainPerpetual for TempoProvider<C> {}
impl<C: Client + Clone> ChainAccount for TempoProvider<C> {}
impl<C: Client + Clone> ChainAddressStatus for TempoProvider<C> {}
#[async_trait]
impl<C: Client + Clone> ChainTraits for TempoProvider<C> {
    async fn get_transaction_fee_estimates(&self) -> Result<TransactionFeeEstimates, Box<dyn Error + Sync + Send>> {
        let mut estimates = self.provider.get_transaction_fee_estimates().await?;
        let transfer_units = self
            .transaction_fee_estimate_units(TransactionFeeOperation::Transfer)
            .ok_or("Missing transfer fee estimate units")?;
        for estimate in &mut estimates.transfer {
            estimate.fee.gas_limit = transfer_units.into();
            estimate.fee.fee = estimate.fee.gas_price_type.total_fee() * &estimate.fee.gas_limit;
        }

        let scale = |estimates: &mut Vec<TransactionFeeEstimate>| {
            estimates.retain(|estimate| estimate.priority == FeePriority::Normal);
            for estimate in estimates {
                estimate.fee.fee = scale_fee_to_token_units(estimate.fee.fee.clone());
                estimate.fee.fee_asset = TEMPO_PATHUSD_ASSET_ID.clone();
            }
        };
        scale(&mut estimates.transfer);
        if let Some(estimates) = &mut estimates.token_transfer {
            scale(estimates);
        }
        if let Some(estimates) = &mut estimates.swap {
            scale(estimates);
        }
        Ok(estimates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::encode::encode_erc20_transfer;
    use gem_evm::provider::preload_mapper::map_evm_transaction_params;
    use gem_evm::rpc::model::TransactionReceipt;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use num_bigint::{BigInt, BigUint};
    use primitives::{EVMChain, TransactionChange, asset_constants::TEMPO_PATHUSD_TOKEN_ID, known_assets::TEMPO_PATHUSD, testkit::signer_mock::TEST_EVM_RECIPIENT};

    #[test]
    fn selects_tempo_provider() {
        let tempo = EthereumClient::new(mock_jsonrpc_client(|_, _| unreachable!()), EVMChain::Tempo);
        let provider = TempoProvider::new_or_else(tempo, |_| unreachable!());
        assert_eq!(provider.get_chain(), Chain::Tempo);

        let ethereum = EthereumClient::new(mock_jsonrpc_client(|_, _| unreachable!()), EVMChain::Ethereum);
        let provider = TempoProvider::new_or_else(ethereum, |client| Box::new(EthereumProvider::new_rpc_only(client)));
        assert_eq!(provider.get_chain(), Chain::Ethereum);
    }

    #[test]
    fn maps_pathusd_transfer_as_tip20() {
        let input = TransactionLoadInput::mock_evm(TransactionInputType::Transfer(TEMPO_PATHUSD.clone()), "1000000");
        let params = map_evm_transaction_params(EVMChain::Tempo, &input).unwrap();

        assert_eq!(params.to, TEMPO_PATHUSD_TOKEN_ID);
        assert_eq!(params.value, BigInt::ZERO);
        assert_eq!(params.data, encode_erc20_transfer(TEST_EVM_RECIPIENT, &BigInt::from(1_000_000u64)).unwrap());
    }

    #[test]
    fn scales_transaction_status_fee_to_tip20_units() {
        let receipt = TransactionReceipt {
            gas_used: BigUint::from(471_789u64),
            effective_gas_price: BigUint::from(1_260_212_000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x1".to_string(),
            block_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            block_number: 291,
            fee_token: None,
        };

        let result = map_transaction_status_with_fee(&receipt, scale_fee_to_token_units(receipt.get_fee().into()));
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(
            result.changes,
            vec![TransactionChange::BlockNumber("291".to_string()), TransactionChange::NetworkFee(BigInt::from(595u64))]
        );
    }

    #[tokio::test]
    async fn scales_fee_estimates_to_tip20_units() {
        let client = EthereumClient::new(
            mock_jsonrpc_client(|_, _| {
                Ok(serde_json::json!({
                    "reward": [["0x0", "0x0"]],
                    "baseFeePerGas": ["0x4a817c800", "0x4a817c800"],
                    "gasUsedRatio": [0.5],
                    "oldestBlock": "0x1"
                }))
            }),
            EVMChain::Tempo,
        );

        let estimates = TempoProvider::new(client).get_transaction_fee_estimates().await.unwrap();
        assert_eq!(estimates.transfer.len(), 1);
        assert_eq!(estimates.transfer[0].priority, FeePriority::Normal);
        assert_eq!(estimates.transfer[0].fee.gas_limit, BigInt::from(65_000u64));
        assert_eq!(estimates.transfer[0].fee.fee, BigInt::from(1_300u64));
        assert_eq!(estimates.transfer[0].fee.fee_asset, TEMPO_PATHUSD_ASSET_ID.clone());
        assert_eq!(estimates.token_transfer.unwrap()[0].fee.fee, BigInt::from(1_300u64));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::testkit::{TEMPO_TEST_ADDRESS, create_tempo_test_client};
    use num_bigint::BigInt;
    use primitives::TransactionPreloadInput;

    #[tokio::test]
    async fn test_get_transaction_load_pathusd_transfer() -> Result<(), Box<dyn Error + Send + Sync>> {
        let provider = TempoProvider::new(create_tempo_test_client());
        let input_type = TransactionInputType::Transfer(primitives::known_assets::TEMPO_PATHUSD.clone());

        let metadata = provider
            .get_transaction_preload(TransactionPreloadInput {
                input_type: input_type.clone(),
                sender_address: TEMPO_TEST_ADDRESS.to_string(),
                destination_address: TEMPO_TEST_ADDRESS.to_string(),
            })
            .await?;

        let fee_rates = provider.get_transaction_fee_rates(input_type.clone()).await?;
        assert_eq!(fee_rates.len(), 1);
        assert_eq!(fee_rates[0].priority, FeePriority::Normal);

        let load_data = provider
            .get_transaction_load(TransactionLoadInput {
                input_type,
                sender_address: TEMPO_TEST_ADDRESS.to_string(),
                destination_address: TEMPO_TEST_ADDRESS.to_string(),
                value: "1000".to_string(),
                gas_price: fee_rates[0].gas_price_type.clone(),
                memo: None,
                is_max_value: false,
                metadata,
            })
            .await?;

        assert!(load_data.fee.gas_limit > BigInt::from(21_000u64));
        assert_eq!(load_data.fee.fee_asset, TEMPO_PATHUSD_ASSET_ID.clone());
        assert!(load_data.fee.fee > BigInt::ZERO);

        Ok(())
    }
}
