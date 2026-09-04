use crate::models::custom_types::GemBigUint;
use primitives::{Asset, AssetId, PerpetualDirection, Resource, Transaction};

use super::rules;
use crate::block_explorer::GemBlockExplorerLink;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionTitle {
    Received,
    Sent,
    Transfer,
    SmartContract,
    Swap,
    Approve,
    Stake,
    Unstake,
    Redelegate,
    Rewards,
    Withdraw,
    ActivateAsset,
    Freeze,
    Unfreeze,
    Earn,
    PerpetualOpen { direction: Option<PerpetualDirection> },
    PerpetualClose { direction: Option<PerpetualDirection> },
    PerpetualModify,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionSubtitle {
    None,
    ToAddress { address: String },
    FromAddress { address: String },
    ToResource { resource: Resource },
    FromResource { resource: Resource },
    Price { value: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAmountSign {
    None,
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionValue {
    None,
    AssetSymbol,
    Amount { sign: GemAmountSign },
    SwapReceived,
    SwapSpent,
    PerpetualNotional,
    PerpetualPnl { value: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemTransactionParticipantRole {
    Sender,
    Recipient,
    Contract,
    Validator,
    Provider,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionParticipant {
    pub role: GemTransactionParticipantRole,
    pub address: String,
    pub link: GemBlockExplorerLink,
}

#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub struct GemTransactionSummary {
    title: GemTransactionTitle,
    subtitle: GemTransactionSubtitle,
    value: GemTransactionValue,
    equivalent_value: GemTransactionValue,
}

#[uniffi::export]
impl GemTransactionSummary {
    #[uniffi::constructor]
    pub fn new(transaction: Transaction) -> Self {
        Self {
            title: rules::transaction_title(&transaction),
            subtitle: rules::transaction_subtitle(&transaction),
            value: rules::transaction_value(&transaction),
            equivalent_value: rules::transaction_equivalent_value(&transaction),
        }
    }

    pub fn title(&self) -> GemTransactionTitle {
        self.title.clone()
    }

    pub fn subtitle(&self) -> GemTransactionSubtitle {
        self.subtitle.clone()
    }

    pub fn value(&self) -> GemTransactionValue {
        self.value.clone()
    }

    pub fn equivalent_value(&self) -> GemTransactionValue {
        self.equivalent_value.clone()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionHeaderKind {
    Amount { shows_fiat: bool },
    Swap,
    Nft,
    Symbol,
    AssetImage,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionDetails {
    pub swap_progress: Option<GemSwapProgress>,
    pub swap_again: Option<GemSwapAgain>,
    pub provider_name: Option<String>,
    pub estimated_confirmation_seconds: Option<u32>,
    pub pnl: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapProgress {
    pub from_asset: Asset,
    pub from_value: GemBigUint,
    pub provider_name: String,
    pub transfer: GemSwapProgressStep,
    pub swap: GemSwapProgressStep,
    pub eta_seconds: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSwapProgressStep {
    Pending,
    Waiting,
    Completed,
    Failed,
    Reverted,
    Refunded,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapAgain {
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}
