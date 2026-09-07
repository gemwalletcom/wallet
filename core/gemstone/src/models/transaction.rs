use crate::address::checksum_address;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::*;
use primitives::contract_call_data::ContractCallData;
use primitives::solana_nft::SolanaNftStandard;
use primitives::solana_token_program::SolanaTokenProgramId;
use primitives::{
    AccountDataType, AssetId, EarnType, FeeOption, GasPriceType, HyperliquidOrder, PerpetualType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput,
    TransactionLoadMetadata, TransactionType, TransferDataOutputAction, TransferDataOutputType, TronStakeData,
};
use std::collections::HashMap;

pub type GemFeeOption = FeeOption;
pub type GemTransferDataOutputType = TransferDataOutputType;
pub type GemTransferDataOutputAction = TransferDataOutputAction;
pub type GemTronStakeData = TronStakeData;

#[uniffi::remote(Enum)]
pub enum FeeOption {
    TokenAccountCreation,
}

pub type GemAccountDataType = AccountDataType;

pub type GemHyperliquidOrder = HyperliquidOrder;

#[uniffi::remote(Record)]
pub struct GemHyperliquidOrder {
    pub approve_agent_required: bool,
    pub approve_referral_required: bool,
    pub approve_builder_required: bool,
    pub builder_fee_bps: u32,
    pub agent_name: String,
    pub agent_address: String,
    pub agent_private_key: String,
}

pub type GemContractCallData = ContractCallData;

pub type GemEarnType = EarnType;

pub type GemPerpetualType = PerpetualType;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: GemBigUint,
    pub gas_price: GasPriceType,
    pub memo: Option<String>,
    pub is_max_value: bool,
    pub metadata: GemTransactionLoadMetadata,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSignerInput {
    pub input: GemTransactionLoadInput,
    pub fee: GemTransactionLoadFee,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSignedTransaction {
    pub data: String,
    pub transaction_type: TransactionType,
}

#[derive(Debug, Default, Clone, uniffi::Record)]
pub struct GemFeeOptions {
    pub options: HashMap<GemFeeOption, GemBigInt>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadFee {
    pub fee: GemBigInt,
    pub gas_price_type: GasPriceType,
    pub gas_limit: GemBigInt,
    pub options: GemFeeOptions,
    pub fee_asset: AssetId,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionData {
    pub fee: GemTransactionLoadFee,
    pub metadata: GemTransactionLoadMetadata,
}

pub type GemTransactionLoadMetadata = TransactionLoadMetadata;

#[uniffi::remote(Enum)]
pub enum GemTransactionLoadMetadata {
    None,
    Solana {
        sender_token_address: Option<String>,
        recipient_token_address: Option<String>,
        token_program: Option<SolanaTokenProgramId>,
        nft: Option<SolanaNftStandard>,
        block_hash: String,
        references: Vec<String>,
    },
    Ton {
        sender_token_address: Option<String>,
        recipient_token_address: Option<String>,
        sequence: u64,
    },
    Cosmos {
        account_number: u64,
        sequence: u64,
        chain_id: String,
    },
    Bitcoin {
        utxos: Vec<GemUTXO>,
    },
    Zcash {
        utxos: Vec<GemUTXO>,
        branch_id: String,
    },
    Cardano {
        utxos: Vec<GemUTXO>,
        block_number: u64,
    },
    Evm {
        nonce: u64,
        chain_id: u64,
        contract_call: Option<GemContractCallData>,
    },
    Near {
        sequence: u64,
        block_hash: String,
    },
    Stellar {
        sequence: u64,
        is_destination_address_exist: bool,
    },
    Xrp {
        sequence: u64,
        block_number: u64,
    },
    Algorand {
        sequence: u64,
        block_hash: String,
        chain_id: String,
    },
    Aptos {
        sequence: u64,
        data: Option<String>,
    },
    Polkadot {
        sequence: u64,
        genesis_hash: String,
        block_hash: String,
        block_number: u64,
        spec_version: u64,
        transaction_version: u64,
        period: u64,
    },
    Tron {
        block_number: u64,
        block_version: u64,
        block_timestamp: u64,
        transaction_tree_root: String,
        parent_hash: String,
        witness_address: String,
        stake_data: GemTronStakeData,
    },
    Sui {
        message_bytes: String,
    },
    Hyperliquid {
        order: Option<GemHyperliquidOrder>,
    },
}

impl From<GemTransactionLoadInput> for TransactionLoadInput {
    fn from(value: GemTransactionLoadInput) -> Self {
        let input_type: TransactionInputType = value.input_type;
        let destination_address = checksum_address(&value.destination_address, input_type.get_asset().chain());
        TransactionLoadInput {
            input_type,
            sender_address: value.sender_address,
            destination_address,
            value: value.value,
            gas_price: value.gas_price,
            memo: value.memo,
            is_max_value: value.is_max_value,
            metadata: value.metadata,
        }
    }
}

impl From<GemSignerInput> for SignerInput {
    fn from(value: GemSignerInput) -> Self {
        SignerInput::new(value.input.into(), value.fee.into())
    }
}

impl From<TransactionLoadInput> for GemTransactionLoadInput {
    fn from(value: TransactionLoadInput) -> Self {
        GemTransactionLoadInput {
            input_type: value.input_type,
            sender_address: value.sender_address,
            destination_address: value.destination_address,
            value: value.value,
            gas_price: value.gas_price,
            memo: value.memo,
            is_max_value: value.is_max_value,
            metadata: value.metadata,
        }
    }
}

impl From<SignerInput> for GemSignerInput {
    fn from(value: SignerInput) -> Self {
        GemSignerInput {
            input: value.input.into(),
            fee: value.fee.into(),
        }
    }
}

pub fn transaction_metadata_block_number(metadata: &GemTransactionLoadMetadata) -> String {
    match metadata {
        GemTransactionLoadMetadata::Polkadot { block_number, .. }
        | GemTransactionLoadMetadata::Tron { block_number, .. }
        | GemTransactionLoadMetadata::Xrp { block_number, .. }
        | GemTransactionLoadMetadata::Cardano { block_number, .. } => block_number.to_string(),
        _ => "0".to_string(),
    }
}

pub fn transaction_metadata_sequence(metadata: &GemTransactionLoadMetadata) -> String {
    match metadata {
        GemTransactionLoadMetadata::Ton { sequence, .. }
        | GemTransactionLoadMetadata::Cosmos { sequence, .. }
        | GemTransactionLoadMetadata::Near { sequence, .. }
        | GemTransactionLoadMetadata::Stellar { sequence, .. }
        | GemTransactionLoadMetadata::Xrp { sequence, .. }
        | GemTransactionLoadMetadata::Algorand { sequence, .. }
        | GemTransactionLoadMetadata::Aptos { sequence, .. }
        | GemTransactionLoadMetadata::Polkadot { sequence, .. } => sequence.to_string(),
        GemTransactionLoadMetadata::Evm { nonce, .. } => nonce.to_string(),
        _ => "0".to_string(),
    }
}

impl GemFeeOptions {
    pub fn get(&self, option: &GemFeeOption) -> Option<&GemBigInt> {
        self.options.get(option)
    }

    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }
}

impl From<GemTransactionLoadFee> for TransactionFee {
    fn from(value: GemTransactionLoadFee) -> Self {
        TransactionFee {
            fee: value.fee,
            gas_price_type: value.gas_price_type,
            gas_limit: value.gas_limit,
            options: value.options.options,
            fee_asset: value.fee_asset,
        }
    }
}

impl From<TransactionFee> for GemTransactionLoadFee {
    fn from(value: TransactionFee) -> Self {
        GemTransactionLoadFee {
            fee: value.fee,
            gas_price_type: value.gas_price_type,
            gas_limit: value.gas_limit,
            options: GemFeeOptions { options: value.options },
            fee_asset: value.fee_asset,
        }
    }
}
