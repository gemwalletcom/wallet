use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainBalances, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainRequestClassifier, ChainSimulation, ChainStaking, ChainState, ChainToken, ChainTraits, ChainTransaction,
    ChainTransactionBroadcast, ChainTransactionDecode, ChainTransactionLoad, ChainTransactionState, ChainTransactions, TransactionsRequest, TransactionsResult,
};
use num_bigint::{BigInt, BigUint};
use primitives::{
    AssetBalance, AssetId, BroadcastOptions, Chain, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionStateRequest, TransactionUpdate,
};

use crate::chain_signer::SignedLightningTransfer;
use crate::error::LightningError;
use crate::invoice::decode_invoice;
use crate::model::{LightningInvoice, LightningPaymentStatus};

pub mod transactions_mapper;

use transactions_mapper::payment_state;

const CHAIN: Chain = Chain::Lightning;

#[async_trait]
pub trait LightningSessions: Send + Sync {
    async fn balance(&self, address: &str) -> Result<u64, LightningError>;
    async fn payment_status(&self, address: &str, id: &str) -> Result<LightningPaymentStatus, LightningError>;
    async fn send_fee(&self, address: &str, invoice: LightningInvoice, amount_sats: u64) -> Result<u64, LightningError>;
    async fn send(&self, signed: SignedLightningTransfer) -> Result<String, LightningError>;
}

pub struct LockedSessions;

#[async_trait]
impl LightningSessions for LockedSessions {
    async fn balance(&self, _address: &str) -> Result<u64, LightningError> {
        Err(LightningError::Locked)
    }

    async fn payment_status(&self, _address: &str, _id: &str) -> Result<LightningPaymentStatus, LightningError> {
        Err(LightningError::Locked)
    }

    async fn send_fee(&self, _address: &str, _invoice: LightningInvoice, _amount_sats: u64) -> Result<u64, LightningError> {
        Err(LightningError::Locked)
    }

    async fn send(&self, _signed: SignedLightningTransfer) -> Result<String, LightningError> {
        Err(LightningError::Locked)
    }
}

pub struct LightningProvider {
    sessions: Arc<dyn LightningSessions>,
}

impl LightningProvider {
    pub fn new(sessions: Arc<dyn LightningSessions>) -> Self {
        Self { sessions }
    }
}

#[async_trait]
impl ChainBalances for LightningProvider {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        let balance = self.sessions.balance(&address).await?;
        Ok(AssetBalance::new(AssetId::from_chain(CHAIN), BigUint::from(balance)))
    }
}

#[async_trait]
impl ChainTransactions for LightningProvider {
    async fn get_transactions_by_address(&self, _request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        Ok(TransactionsResult::Transactions(Vec::new()))
    }
}

#[async_trait]
impl ChainState for LightningProvider {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        Ok(Some(CHAIN.network_id().to_string()))
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(0)
    }
}

impl ChainProvider for LightningProvider {
    fn get_chain(&self) -> Chain {
        CHAIN
    }
}

#[async_trait]
impl ChainTransactionLoad for LightningProvider {
    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let invoice = decode_invoice(&input.destination_address)?;
        let fee_sats = self.sessions.send_fee(&input.sender_address, invoice, input.value_as_u64()?).await?;
        Ok(TransactionLoadData {
            fee: TransactionFee::new_from_fee(BigInt::from(fee_sats), AssetId::from_chain(CHAIN)),
            metadata: TransactionLoadMetadata::None,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(0))])
    }
}

#[async_trait]
impl ChainTransactionBroadcast for LightningProvider {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.sessions.send(SignedLightningTransfer::decode(&data)?).await?)
    }
}

#[async_trait]
impl ChainTransactionState for LightningProvider {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let status = self.sessions.payment_status(&request.sender_address, &request.id).await?;
        Ok(TransactionUpdate::new_state(payment_state(status)))
    }
}

impl ChainStaking for LightningProvider {}
impl ChainTransaction for LightningProvider {}
impl ChainBlockTransactions for LightningProvider {}
impl ChainAccount for LightningProvider {}
impl ChainPerpetual for LightningProvider {}
impl ChainToken for LightningProvider {}
impl ChainAddressStatus for LightningProvider {}
impl ChainSimulation for LightningProvider {}
impl ChainTraits for LightningProvider {}

pub struct BroadcastProvider;

impl ChainRequestClassifier for BroadcastProvider {}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, _request: &[u8], _response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        Err("Lightning payments are not broadcast through a node".into())
    }
}
