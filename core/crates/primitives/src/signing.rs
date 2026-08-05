use crate::{Chain, TransactionType, TransferDataOutputType, UInt64, WCEthereumTransaction};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum SignDigestType {
    Eip191,
    Eip712,
    Base58,
    SuiPersonal,
    Siwe,
    TonPersonal,
    TronPersonal,
}

#[derive(Debug)]
pub struct SignMessage {
    pub chain: Chain,
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SignableTransaction {
    Ethereum {
        data: EthereumTransactionData,
        transaction_type: TransactionType,
    },
    Solana {
        data: SolanaTransactionData,
        output_type: TransferDataOutputType,
    },
    Sui {
        data: SuiTransactionData,
        output_type: TransferDataOutputType,
    },
    Ton {
        data: String,
        output_type: TransferDataOutputType,
    },
    Tron {
        data: String,
        output_type: TransferDataOutputType,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignableTransactionType {
    Ethereum,
    Solana { output_type: TransferDataOutputType },
    Sui { output_type: TransferDataOutputType },
    Ton { output_type: TransferDataOutputType },
    Tron { output_type: TransferDataOutputType },
}

impl SignableTransactionType {
    pub fn get_output_type(&self) -> Option<TransferDataOutputType> {
        match self {
            Self::Ethereum => None,
            Self::Solana { output_type } | Self::Sui { output_type } | Self::Ton { output_type } | Self::Tron { output_type } => Some(output_type.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EthereumTransactionData {
    pub chain_id: Option<UInt64>,
    pub from: String,
    pub to: String,
    pub value: Option<String>,
    pub gas: Option<String>,
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SolanaTransactionData {
    pub transaction: String,
}

#[derive(Debug, Clone)]
pub struct SuiTransactionData {
    pub transaction: String,
    pub wallet_address: String,
}

impl From<WCEthereumTransaction> for EthereumTransactionData {
    fn from(transaction: WCEthereumTransaction) -> Self {
        Self {
            chain_id: transaction.chain_id,
            from: transaction.from,
            to: transaction.to,
            value: transaction.value,
            gas: transaction.gas,
            gas_limit: transaction.gas_limit,
            gas_price: transaction.gas_price,
            max_fee_per_gas: transaction.max_fee_per_gas,
            max_priority_fee_per_gas: transaction.max_priority_fee_per_gas,
            nonce: transaction.nonce,
            data: transaction.data,
        }
    }
}
