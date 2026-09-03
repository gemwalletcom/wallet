use std::{
    error::Error,
    str,
    time::{Duration, Instant},
};

use async_trait::async_trait;
pub use primitives::TransactionIdRequest;
use primitives::chart::ChartCandleStick;
use primitives::perpetual::{PerpetualAccountMode, PerpetualData, PerpetualPositionsSummary};
use primitives::portfolio::PerpetualPortfolio;
use primitives::{
    AddressStatus, Asset, AssetBalance, AssetId, BroadcastOptions, Chain, ChainRequest, ChainRequestType, ChartPeriod, DelegationBase, DelegationValidator, FeeRate,
    NodeCheckReport, NodeCheckRequest, NodeStatus, NodeSyncStatus, PerpetualPosition, SimulationInput, SimulationResult, Transaction, TransactionFee, TransactionInputType,
    TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput, TransactionStateRequest, TransactionUpdate, UTXO,
};

pub mod node_check;
mod transaction_fee;

pub use transaction_fee::{TransactionFeeEstimate, TransactionFeeEstimates, TransactionFeeOperation};

#[cfg(feature = "testkit")]
pub mod testkit;

pub enum TransactionsResult {
    Transactions(Vec<Transaction>),
    TransactionRequests(Vec<TransactionIdRequest>),
}

pub struct TransactionsRequest {
    pub address: String,
    pub asset_id: Option<AssetId>,
    pub limit: usize,
    pub from_timestamp: Option<u64>,
}

impl TransactionsRequest {
    pub fn new(address: String, limit: usize) -> Self {
        Self {
            address,
            asset_id: None,
            limit,
            from_timestamp: None,
        }
    }

    pub fn with_from_timestamp(self, from_timestamp: Option<u64>) -> Self {
        Self { from_timestamp, ..self }
    }
}

#[async_trait]
pub trait ChainTraits:
    ChainProvider
    + ChainBalances
    + ChainStaking
    + ChainTransactionBroadcast
    + ChainTransaction
    + ChainBlockTransactions
    + ChainTransactions
    + ChainTransactionState
    + ChainState
    + ChainAccount
    + ChainPerpetual
    + ChainToken
    + ChainTransactionLoad
    + ChainAddressStatus
    + ChainSimulation
{
    async fn get_transaction_fee_estimates(&self) -> Result<TransactionFeeEstimates, Box<dyn Error + Sync + Send>> {
        let chain = self.get_chain();
        let rates = self.get_transaction_fee_rates(TransactionInputType::Transfer(Asset::from_chain(chain))).await?;
        let estimate = |operation| {
            let units = self.transaction_fee_estimate_units(operation);
            rates.iter().map(|rate| TransactionFeeEstimate::new(rate, units, chain.fee_unit_type())).collect()
        };
        Ok(TransactionFeeEstimates {
            fee_asset: AssetId::from_chain(chain),
            transfer: estimate(TransactionFeeOperation::Transfer),
            token_transfer: chain.default_asset_type().is_some().then(|| estimate(TransactionFeeOperation::TokenTransfer)),
            swap: chain.is_swap_supported().then(|| estimate(TransactionFeeOperation::Swap)),
        })
    }

    async fn check_node(&self, request: &NodeCheckRequest, status: &NodeSyncStatus, status_latency: Duration) -> NodeCheckReport {
        node_check::check_node(self, request, status, status_latency).await
    }

    async fn get_nodes_status(&self) -> Result<NodeStatus, Box<dyn Error + Send + Sync>> {
        let started_at = Instant::now();
        let latest_block_number = self.get_block_latest_number().await?;

        Ok(NodeStatus {
            latest_block_number,
            latency_ms: started_at.elapsed().as_millis() as u64,
        })
    }
}

pub trait ChainProvider: Send + Sync {
    fn get_chain(&self) -> Chain;
}

pub trait ChainRequestClassifier: Send + Sync {
    fn classify_request(&self, _request: ChainRequest<'_>) -> ChainRequestType {
        ChainRequestType::Unknown
    }
}

#[async_trait]
pub trait ChainBalances: Send + Sync {
    async fn get_balance_coin(&self, _address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support balance operations".into())
    }
    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support balance operations".into())
    }
    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support balance operations".into())
    }
    async fn get_balance_assets(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Err("Chain does not support balance operations".into())
    }
}

#[async_trait]
pub trait ChainStaking: Send + Sync {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_staking_delegation_validators(&self, _address: String) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_staking_delegations(&self, _address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainTransactionBroadcast: Send + Sync {
    async fn transaction_broadcast(&self, _data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction broadcasting".into())
    }
}

pub trait ChainTransactionDecode: Send + Sync {
    fn decode_transaction_broadcast(&self, _response: &str) -> Option<String> {
        None
    }

    fn decode_transaction_broadcast_bytes(&self, response: &[u8]) -> Option<String> {
        str::from_utf8(response).ok().and_then(|response| self.decode_transaction_broadcast(response))
    }
}

#[async_trait]
pub trait ChainTransaction: Send + Sync {
    async fn get_transaction_by_hash(&self, _request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

#[async_trait]
pub trait ChainBlockTransactions: Send + Sync {
    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_transactions_in_blocks(&self, blocks: Vec<u64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let futures = blocks.into_iter().map(|block| self.get_transactions_by_block(block));
        let results = futures::future::try_join_all(futures).await?;
        Ok(results.into_iter().flatten().collect())
    }
}

#[async_trait]
pub trait ChainTransactions: Send + Sync {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>>;
}

pub struct EmptyTransactionsProvider;

#[async_trait]
impl ChainTransactions for EmptyTransactionsProvider {
    async fn get_transactions_by_address(&self, _request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        Ok(TransactionsResult::Transactions(Vec::new()))
    }
}

#[async_trait]
impl ChainTransaction for EmptyTransactionsProvider {}

#[async_trait]
impl ChainBlockTransactions for EmptyTransactionsProvider {}

#[async_trait]
pub trait ChainTransactionState: Send + Sync {
    async fn get_transaction_status(&self, _request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction status".into())
    }
}

#[async_trait]
pub trait ChainState: Send + Sync {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>>;
    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        Ok(NodeSyncStatus::in_sync())
    }
    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainAccount: Send + Sync {}

#[async_trait]
pub trait ChainPerpetual: Send + Sync {
    async fn get_positions(&self, _address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual trading".into())
    }

    async fn get_positions_for_classification(&self, address: String) -> Result<Vec<PerpetualPosition>, Box<dyn Error + Sync + Send>> {
        Ok(self.get_positions(address).await?.positions)
    }

    async fn get_perpetual_account_mode(&self, _address: String) -> Result<PerpetualAccountMode, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual trading".into())
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual trading".into())
    }

    async fn get_perpetual_candlesticks(&self, _symbol: String, _period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual trading".into())
    }

    async fn get_perpetual_portfolio(&self, _address: String) -> Result<PerpetualPortfolio, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual portfolio".into())
    }

    async fn get_perpetual_referred_addresses(&self) -> Result<Vec<String>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainToken: Send + Sync {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support tokens".into())
    }

    fn get_is_token_address(&self, _token_id: &str) -> bool {
        false
    }
}

#[async_trait]
pub trait ChainTransactionLoad: Send + Sync {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, _input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction loading".into())
    }

    async fn get_transaction_fee_from_data(&self, _data: String) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction fee".into())
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support fee rates".into())
    }

    fn transaction_fee_estimate_units(&self, _operation: TransactionFeeOperation) -> Option<u64> {
        None
    }

    async fn get_utxos(&self, _address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainAddressStatus: Send + Sync {
    async fn get_address_status(&self, _address: String) -> Result<Vec<AddressStatus>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainSimulation: Send + Sync {
    async fn simulate_transaction(&self, _input: SimulationInput) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        Err("Chain does not support transaction simulation".into())
    }
}
