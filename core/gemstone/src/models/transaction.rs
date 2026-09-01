use crate::address::checksum_address;
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::*;
use primitives::ApplicationMetadata;
use primitives::contract_call_data::ContractCallData;
use primitives::nft::NFTAsset;
use primitives::solana_nft::SolanaNftStandard;
use primitives::solana_token_program::SolanaTokenProgramId;
use primitives::{
    AccountDataType, AssetId, EarnType, FeeOption, GasPriceType, HyperliquidOrder, PerpetualConfirmData, PerpetualDirection, PerpetualProvider, PerpetualType, SignerInput,
    StakeType, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransactionType, TransferDataExtra, TransferDataOutputAction,
    TransferDataOutputType, TronStakeData, perpetual::PerpetualReduceData,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use swap::{GemApprovalData, GemSwapData};

pub type GemPerpetualDirection = PerpetualDirection;
pub type GemPerpetualProvider = PerpetualProvider;
pub type GemPerpetualConfirmData = PerpetualConfirmData;
pub type GemPerpetualReduceData = PerpetualReduceData;
pub type GemFeeOption = FeeOption;
pub type GemTransferDataOutputType = TransferDataOutputType;
pub type GemTransferDataOutputAction = TransferDataOutputAction;
#[uniffi::remote(Enum)]
pub enum PerpetualProvider {
    Hypercore,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemTransferDataExtra {
    pub to: String,
    pub gas_limit: Option<String>,
    pub gas_price: Option<GemGasPriceType>,
    pub data: Option<Vec<u8>>,
    pub output_type: GemTransferDataOutputType,
    pub output_action: GemTransferDataOutputAction,
    pub transaction_type: TransactionType,
    pub approval: Option<GemApprovalData>,
}

pub type GemPerpetualType = PerpetualType;

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemTransactionInputType {
    Transfer {
        asset: GemAsset,
    },
    Deposit {
        asset: GemAsset,
    },
    Swap {
        from_asset: GemAsset,
        to_asset: GemAsset,
        swap_data: GemSwapData,
    },
    Stake {
        asset: GemAsset,
        stake_type: StakeType,
    },
    TokenApprove {
        asset: GemAsset,
        approval_data: GemApprovalData,
    },
    Generic {
        asset: GemAsset,
        metadata: ApplicationMetadata,
        extra: GemTransferDataExtra,
    },
    TransferNft {
        asset: GemAsset,
        nft_asset: NFTAsset,
    },
    Account {
        asset: GemAsset,
        account_type: GemAccountDataType,
    },
    Perpetual {
        asset: GemAsset,
        perpetual_type: GemPerpetualType,
    },
    Earn {
        asset: GemAsset,
        earn_type: GemEarnType,
        data: GemContractCallData,
    },
    // Gemstone-only: primitives has no withdrawal variant, so it routes to sign_withdrawal but lowers to Transfer.
    Withdrawal {
        asset: GemAsset,
    },
}

impl GemTransactionInputType {
    pub fn asset(&self) -> &GemAsset {
        match self {
            Self::Transfer { asset }
            | Self::Deposit { asset }
            | Self::Stake { asset, .. }
            | Self::TokenApprove { asset, .. }
            | Self::Generic { asset, .. }
            | Self::TransferNft { asset, .. }
            | Self::Account { asset, .. }
            | Self::Perpetual { asset, .. }
            | Self::Earn { asset, .. }
            | Self::Withdrawal { asset } => asset,
            Self::Swap { from_asset, .. } => from_asset,
        }
    }

    pub fn swap_data(&self) -> Result<&GemSwapData, String> {
        match self {
            Self::Swap { swap_data, .. } => Ok(swap_data),
            _ => Err("Expected Swap".to_string()),
        }
    }

    pub fn earn_data(&self) -> Result<&GemContractCallData, String> {
        match self {
            Self::Earn { data, .. } => Ok(data),
            _ => Err("Expected Earn".to_string()),
        }
    }

    pub fn stake_type(&self) -> Result<&StakeType, String> {
        match self {
            Self::Stake { stake_type, .. } => Ok(stake_type),
            _ => Err("Expected Stake".to_string()),
        }
    }

    pub fn perpetual_type(&self) -> Result<&GemPerpetualType, String> {
        match self {
            Self::Perpetual { perpetual_type, .. } => Ok(perpetual_type),
            _ => Err("Expected Perpetual".to_string()),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionLoadInput {
    pub input_type: GemTransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: GemBigUint,
    pub gas_price: GemGasPriceType,
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
    pub gas_price_type: GemGasPriceType,
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
        let input_type: TransactionInputType = value.input_type.into();
        let destination_address = checksum_address(&value.destination_address, input_type.get_asset().chain());
        TransactionLoadInput {
            input_type,
            sender_address: value.sender_address,
            destination_address,
            value: value.value,
            gas_price: value.gas_price.into(),
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
            input_type: value.input_type.into(),
            sender_address: value.sender_address,
            destination_address: value.destination_address,
            value: value.value,
            gas_price: value.gas_price.into(),
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

impl From<TransactionInputType> for GemTransactionInputType {
    fn from(value: TransactionInputType) -> Self {
        match value {
            TransactionInputType::Transfer(asset) => GemTransactionInputType::Transfer { asset },
            TransactionInputType::Deposit(asset) => GemTransactionInputType::Deposit { asset },
            TransactionInputType::Swap(from_asset, to_asset, swap_data) => GemTransactionInputType::Swap { from_asset, to_asset, swap_data },
            TransactionInputType::Stake(asset, stake_type) => GemTransactionInputType::Stake { asset, stake_type },
            TransactionInputType::TokenApprove(asset, approval_data) => GemTransactionInputType::TokenApprove { asset, approval_data },
            TransactionInputType::Generic(asset, metadata, extra) => GemTransactionInputType::Generic {
                asset,
                metadata,
                extra: extra.into(),
            },
            TransactionInputType::TransferNft(asset, nft_asset) => GemTransactionInputType::TransferNft { asset, nft_asset },
            TransactionInputType::Account(asset, account_type) => GemTransactionInputType::Account { asset, account_type },
            TransactionInputType::Perpetual(asset, perpetual_type) => GemTransactionInputType::Perpetual { asset, perpetual_type },
            TransactionInputType::Earn(asset, earn_type, data) => GemTransactionInputType::Earn { asset, earn_type, data },
        }
    }
}

impl From<GemTransferDataExtra> for TransferDataExtra {
    fn from(value: GemTransferDataExtra) -> Self {
        TransferDataExtra {
            to: value.to,
            gas_limit: value.gas_limit.map(|s| s.parse().unwrap_or_default()),
            gas_price: value.gas_price.map(|gp| gp.into()),
            data: value.data,
            output_type: value.output_type,
            output_action: value.output_action,
            transaction_type: value.transaction_type,
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

impl From<TransferDataExtra> for GemTransferDataExtra {
    fn from(value: TransferDataExtra) -> Self {
        GemTransferDataExtra {
            to: value.to,
            gas_limit: value.gas_limit.map(|x| x.to_string()),
            gas_price: value.gas_price.map(|x| x.into()),
            data: value.data,
            output_type: value.output_type,
            output_action: value.output_action,
            transaction_type: value.transaction_type,
            approval: None,
        }
    }
}

impl From<GemGasPriceType> for GasPriceType {
    fn from(value: GemGasPriceType) -> Self {
        match value {
            GemGasPriceType::Regular { gas_price } => GasPriceType::Regular { gas_price },
            GemGasPriceType::Eip1559 { gas_price, priority_fee } => GasPriceType::Eip1559 { gas_price, priority_fee },
            GemGasPriceType::Solana {
                gas_price,
                priority_fee,
                unit_price,
            } => GasPriceType::Solana {
                gas_price,
                priority_fee,
                unit_price,
            },
        }
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
            gas_price_type: value.gas_price_type.into(),
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
            gas_price_type: value.gas_price_type.into(),
            gas_limit: value.gas_limit,
            options: GemFeeOptions { options: value.options },
            fee_asset: value.fee_asset,
        }
    }
}

impl From<GemTransactionInputType> for TransactionInputType {
    fn from(value: GemTransactionInputType) -> Self {
        match value {
            GemTransactionInputType::Transfer { asset } => TransactionInputType::Transfer(asset),
            GemTransactionInputType::Deposit { asset } => TransactionInputType::Deposit(asset),
            GemTransactionInputType::Swap { from_asset, to_asset, swap_data } => TransactionInputType::Swap(
                from_asset,
                to_asset,
                GemSwapData {
                    quote: swap_data.quote,
                    data: swap_data.data,
                },
            ),
            GemTransactionInputType::Stake { asset, stake_type } => TransactionInputType::Stake(asset, stake_type),
            GemTransactionInputType::TokenApprove { asset, approval_data } => TransactionInputType::TokenApprove(
                asset,
                GemApprovalData {
                    token: approval_data.token,
                    spender: approval_data.spender,
                    value: approval_data.value,
                    is_unlimited: approval_data.is_unlimited,
                },
            ),
            GemTransactionInputType::Generic { asset, metadata, extra } => TransactionInputType::Generic(asset, metadata, extra.into()),
            GemTransactionInputType::TransferNft { asset, nft_asset } => TransactionInputType::TransferNft(asset, nft_asset),
            GemTransactionInputType::Account { asset, account_type } => TransactionInputType::Account(asset, account_type),
            GemTransactionInputType::Perpetual { asset, perpetual_type } => TransactionInputType::Perpetual(asset, perpetual_type),
            GemTransactionInputType::Earn { asset, earn_type, data } => TransactionInputType::Earn(asset, earn_type, data),
            GemTransactionInputType::Withdrawal { asset } => TransactionInputType::Transfer(asset),
        }
    }
}
