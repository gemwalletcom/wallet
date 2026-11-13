use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainBalances, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainRequestClassifier, ChainSimulation, ChainStaking, ChainState, ChainToken, ChainTraits, ChainTransaction,
    ChainTransactionBroadcast, ChainTransactionDecode, ChainTransactionLoad, ChainTransactionState, ChainTransactions, TransactionsRequest, TransactionsResult,
};
use chrono::DateTime;
use gem_hash::sha2::sha256;
use num_bigint::{BigInt, BigUint};
use primitives::{
    AssetBalance, AssetId, BroadcastOptions, Chain, ChainSigner, FeePriority, FeeRate, GasPriceType, SignerError, SignerInput, Transaction, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput,
    TransactionLoadMetadata, TransactionState, TransactionStateRequest, TransactionType, TransactionUpdate, hex,
};
use signer::{SignatureScheme, Signer};

use crate::error::LightningError;
use crate::invoice::decode_invoice;
use crate::model::{LightningInvoice, LightningPayment, LightningPaymentDirection, LightningPaymentStatus, LightningSendQuote};

const CHAIN: Chain = Chain::Lightning;
const PAYLOAD_SEPARATOR: &str = "|";
const AUTHORIZATION_DOMAIN: &str = "gem-lightning-transfer-v1";

#[async_trait]
pub trait LightningSessions: Send + Sync {
    async fn balance(&self, address: &str) -> Result<u64, LightningError>;
    async fn payment_status(&self, address: &str, id: &str) -> Result<LightningPaymentStatus, LightningError>;
    async fn send_fee(&self, address: &str, invoice: LightningInvoice, amount_sats: u64) -> Result<u64, LightningError>;
    async fn send(&self, address: &str, quote: LightningSendQuote, authorization: LightningAuthorization) -> Result<String, LightningError>;
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

    async fn send(&self, _address: &str, _quote: LightningSendQuote, _authorization: LightningAuthorization) -> Result<String, LightningError> {
        Err(LightningError::Locked)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningAuthorization {
    pub digest: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningTransfer {
    pub address: String,
    pub invoice: String,
    pub amount_sats: u64,
    pub max_fee_sats: u64,
}

impl LightningTransfer {
    fn message(&self) -> String {
        [self.address.clone(), self.invoice.clone(), self.amount_sats.to_string(), self.max_fee_sats.to_string()].join(PAYLOAD_SEPARATOR)
    }

    fn digest(&self) -> [u8; 32] {
        sha256([AUTHORIZATION_DOMAIN.to_string(), self.message()].join(PAYLOAD_SEPARATOR).as_bytes())
    }

    fn sign(&self, private_key: &[u8]) -> Result<String, SignerError> {
        let signature = Signer::sign_digest(SignatureScheme::Secp256k1, &self.digest(), private_key)?;
        Ok(self.encode(&signature))
    }

    pub fn encode(&self, signature: &[u8]) -> String {
        [self.message(), hex::encode(signature)].join(PAYLOAD_SEPARATOR)
    }

    pub fn decode(value: &str) -> Result<(Self, LightningAuthorization), LightningError> {
        let mut parts = value.split(PAYLOAD_SEPARATOR);
        let (Some(address), Some(invoice), Some(amount), Some(fee), Some(signature), None) = (parts.next(), parts.next(), parts.next(), parts.next(), parts.next(), parts.next()) else {
            return Err(LightningError::InvalidTransfer);
        };
        let transfer = Self {
            address: address.to_string(),
            invoice: invoice.to_string(),
            amount_sats: amount.parse().map_err(|_| LightningError::InvalidTransfer)?,
            max_fee_sats: fee.parse().map_err(|_| LightningError::InvalidTransfer)?,
        };
        let authorization = LightningAuthorization {
            digest: transfer.digest(),
            signature: hex::decode_hex(signature).map_err(|_| LightningError::InvalidTransfer)?,
        };
        Ok((transfer, authorization))
    }

    fn quote(&self) -> Result<LightningSendQuote, LightningError> {
        Ok(LightningSendQuote {
            invoice: decode_invoice(&self.invoice)?,
            amount_sats: self.amount_sats,
            fee_sats: self.max_fee_sats,
        })
    }
}

fn payment_state(status: LightningPaymentStatus) -> TransactionState {
    match status {
        LightningPaymentStatus::Pending => TransactionState::Pending,
        LightningPaymentStatus::Completed => TransactionState::Confirmed,
        LightningPaymentStatus::Failed => TransactionState::Failed,
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

pub fn map_payment(address: &str, payment: &LightningPayment) -> Transaction {
    let asset_id = AssetId::from_chain(CHAIN);
    let (from, to) = match payment.direction {
        LightningPaymentDirection::Incoming => (String::new(), address.to_string()),
        LightningPaymentDirection::Outgoing => (address.to_string(), String::new()),
    };
    let created_at = DateTime::from_timestamp(i64::try_from(payment.created_at).unwrap_or_default(), 0).unwrap_or_default();
    Transaction::new(
        payment.id.clone(),
        asset_id.clone(),
        from,
        to,
        None,
        TransactionType::Transfer,
        payment_state(payment.status),
        BigUint::from(payment.fee_sats),
        asset_id,
        BigUint::from(payment.amount_sats),
        None,
        None,
        created_at,
    )
    .finalize(vec![address.to_string()])
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
        let amount_sats = u64::try_from(&input.value).map_err(|_| LightningError::AmountRequired)?;
        let fee_sats = self.sessions.send_fee(&input.sender_address, invoice, amount_sats).await?;
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
        let (transfer, authorization) = LightningTransfer::decode(&data)?;
        Ok(self.sessions.send(&transfer.address, transfer.quote()?, authorization).await?)
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

pub struct LightningChainSigner;

impl ChainSigner for LightningChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let invoice = decode_invoice(&input.destination_address).map_err(|error| SignerError::invalid_input(error.to_string()))?;
        let amount_sats = u64::try_from(&input.value).map_err(|_| SignerError::invalid_input("invalid amount"))?;
        let max_fee_sats = u64::try_from(&input.fee.fee).map_err(|_| SignerError::invalid_input("invalid fee"))?;
        LightningTransfer {
            address: input.sender_address.clone(),
            invoice: invoice.invoice,
            amount_sats,
            max_fee_sats,
        }
        .sign(private_key)
    }
}

pub struct BroadcastProvider;

impl ChainRequestClassifier for BroadcastProvider {}

impl ChainTransactionDecode for BroadcastProvider {
    fn decode_transaction_broadcast(&self, _request: &[u8], _response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
        Err("Lightning payments are not broadcast through a node".into())
    }
}

#[cfg(test)]
mod tests {
    use primitives::TransactionDirection;

    use super::*;
    use crate::testkit::INVOICE_2000_SATS;

    #[test]
    fn test_map_payment() {
        let incoming = map_payment("spark1address", &LightningPayment::mock(LightningPaymentDirection::Incoming, LightningPaymentStatus::Completed));
        let outgoing = map_payment("spark1address", &LightningPayment::mock(LightningPaymentDirection::Outgoing, LightningPaymentStatus::Pending));

        assert_eq!(incoming.direction, TransactionDirection::Incoming);
        assert_eq!((incoming.from.as_str(), incoming.to.as_str()), ("", "spark1address"));
        assert_eq!(incoming.state, TransactionState::Confirmed);
        assert_eq!(incoming.value, BigUint::from(1_000u64));
        assert_eq!(outgoing.direction, TransactionDirection::Outgoing);
        assert_eq!(outgoing.from, "spark1address");
        assert_eq!(outgoing.state, TransactionState::Pending);
        assert_eq!(outgoing.fee, BigUint::from(3u64));
    }

    #[test]
    fn test_lightning_transfer_round_trip() {
        let transfer = LightningTransfer {
            address: "spark1address".to_string(),
            invoice: INVOICE_2000_SATS.to_string(),
            amount_sats: 2_000,
            max_fee_sats: 7,
        };

        let (decoded, authorization) = LightningTransfer::decode(&transfer.sign(&[7u8; 32]).unwrap()).unwrap();

        assert_eq!(decoded, transfer);
        assert_eq!(authorization.digest, transfer.digest());
        assert_ne!(authorization.digest, sha256(transfer.message().as_bytes()));
        assert_eq!(authorization.signature.len(), 65);
        assert_eq!(LightningTransfer::decode("spark1address|invoice|2000|7"), Err(LightningError::InvalidTransfer));
        assert_eq!(LightningTransfer::decode("spark1address|invoice|amount|7|00"), Err(LightningError::InvalidTransfer));
    }
}
