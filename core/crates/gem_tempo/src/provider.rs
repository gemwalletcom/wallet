use std::error::Error;

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainBalances, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainToken, ChainTraits,
    ChainTransaction, ChainTransactionBroadcast, ChainTransactionLoad, ChainTransactionState, ChainTransactions, TransactionFeeOperation, TransactionIdRequest,
    TransactionsRequest, TransactionsResult,
};
use gem_client::Client;
use gem_evm::provider::transaction_state_mapper::map_transaction_status_with_fee;
use gem_evm::rpc::mapper::EthereumMapper;
use gem_evm::rpc::{EthereumClient, EthereumProvider, EvmProviderExtensions};
use primitives::{
    Asset, AssetBalance, AssetId, AssetType, BroadcastOptions, Chain, FeeRate, SimulationInput, SimulationResult, Transaction, TransactionInputType, TransactionLoadData,
    TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput, TransactionState, TransactionStateRequest, TransactionUpdate, asset_constants::TEMPO_PATHUSD_TOKEN_ID,
    fee::FeePriority,
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
}

impl<C: Client + Clone> TempoProvider<C> {
    fn client(&self) -> &EthereumClient<C> {
        &self.provider
    }
}

impl<C: Client + Clone> ChainProvider for TempoProvider<C> {
    fn get_chain(&self) -> Chain {
        self.provider.get_chain()
    }
}

#[async_trait]
impl<C: Client + Clone> ChainBalances for TempoProvider<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self
            .client()
            .batch_token_balance_calls(&address, &[TEMPO_PATHUSD_TOKEN_ID.to_string()])
            .await?
            .into_iter()
            .next()
            .ok_or("Missing pathUSD balance result")?;
        gem_evm::provider::balances_mapper::map_balance_coin(balance, Chain::Tempo)
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
        self.provider.transaction_fee_estimate_units(operation)
    }

    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        self.provider.get_transaction_preload(input).await
    }

    async fn get_transaction_fee_rates(&self, input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        // Tempo has a single fee lane: priority fees are always 0, so only the Normal rate is meaningful.
        Ok(self
            .provider
            .get_transaction_fee_rates(input_type)
            .await?
            .into_iter()
            .filter(|rate| rate.priority == FeePriority::Normal)
            .collect())
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        self.provider.map_transaction_load(map_pathusd_transfer_input(input)).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactionState for TempoProvider<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let Some(receipt) = self.client().get_transaction_receipt(&request.id).await? else {
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
        let Some(receipt) = self.client().get_transaction_receipt(&hash).await? else {
            return Ok(None);
        };
        Ok(Some(mapper::map_transaction(transaction, &receipt)))
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
        let block = self.client().get_block(block_number).await?;
        if block.transactions.is_empty() {
            return Ok(Vec::new());
        }

        let receipts = self.client().get_block_receipts(block_number).await?;
        Ok(block
            .transactions
            .into_iter()
            .zip(receipts)
            .filter_map(|(transaction, receipt)| {
                EthereumMapper::map_transaction(Chain::Tempo, &transaction, &receipt, &block.timestamp, &[]).map(|mapped| mapper::map_transaction(mapped, &receipt))
            })
            .collect())
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
impl<C: Client + Clone> chain_traits::ChainState for TempoProvider<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        chain_traits::ChainState::get_chain_id(&self.provider).await
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
impl<C: Client + Clone> ChainTraits for TempoProvider<C> {}

fn map_pathusd_transfer_input(input: TransactionLoadInput) -> TransactionLoadInput {
    let asset = match &input.input_type {
        TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) if asset.id.is_native() => asset,
        _ => return input,
    };
    let pathusd = Asset::new(
        AssetId::from_token(Chain::Tempo, TEMPO_PATHUSD_TOKEN_ID),
        asset.name.clone(),
        asset.symbol.clone(),
        asset.decimals,
        AssetType::TIP20,
    );
    let input_type = match input.input_type {
        TransactionInputType::Deposit(_) => TransactionInputType::Deposit(pathusd),
        _ => TransactionInputType::Transfer(pathusd),
    };
    TransactionLoadInput { input_type, ..input }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::provider::preload_mapper::map_evm_transaction_params;
    use gem_evm::rpc::model::TransactionReceipt;
    use num_bigint::{BigInt, BigUint};
    use primitives::{EVMChain, TransactionChange, asset_constants::TEMPO_USDC_TOKEN_ID, hex, testkit::signer_mock::TEST_EVM_RECIPIENT};

    #[test]
    fn maps_chain_asset_transfer_to_pathusd_contract() {
        let input = map_pathusd_transfer_input(TransactionLoadInput::mock_evm(TransactionInputType::Transfer(Asset::from_chain(Chain::Tempo)), "1000000"));
        let params = map_evm_transaction_params(EVMChain::Tempo, &input).unwrap();

        assert_eq!(params.to, TEMPO_PATHUSD_TOKEN_ID);
        assert_eq!(params.value, BigInt::ZERO);
        assert_eq!(hex::encode(&params.data[..4]), "a9059cbb");
        assert!(hex::encode(&params.data).contains(&TEST_EVM_RECIPIENT[2..].to_lowercase()));

        let token = Asset::mock_tempo_usdc();
        let unchanged = map_pathusd_transfer_input(TransactionLoadInput::mock_evm(TransactionInputType::Transfer(token.clone()), "1000000"));
        assert_eq!(unchanged.input_type.get_asset(), &token);
        assert_eq!(unchanged.input_type.get_asset().token_id.as_deref(), Some(TEMPO_USDC_TOKEN_ID));
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
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::testkit::{TEMPO_TEST_ADDRESS, create_tempo_test_client};
    use num_bigint::BigInt;
    use primitives::{AssetId, TransactionPreloadInput};

    #[tokio::test]
    async fn test_get_transaction_load_native_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = TempoProvider::new(create_tempo_test_client());
        let input_type = TransactionInputType::Transfer(Asset::from_chain(Chain::Tempo));

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

        println!("Tempo native transfer load: {:#?}", load_data.fee);

        // Native transfers execute as pathUSD ERC-20 calls, so the estimate exceeds the plain 21k transfer.
        assert!(load_data.fee.gas_limit > BigInt::from(21_000u64));
        // The fee is scaled to pathUSD 6-decimal units.
        assert_eq!(load_data.fee.fee_asset, Asset::from_chain(Chain::Tempo));
        assert!(load_data.fee.fee > BigInt::ZERO);

        Ok(())
    }
}
