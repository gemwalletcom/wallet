use crate::models::custom_types::GemBigInt;
use crate::models::custom_types::GemBigUint;
use crate::services::transfer::{GemRecipient, GemTransferData};
use primitives::swap::{SwapData, SwapQuote, SwapQuoteData};
use primitives::{Asset, AssetId};
use swapper::{Quote, SwapperError};

use super::rules;
use primitives::TransactionInputType;

#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub struct GemSwapQuoteSummary {
    quote: SwapQuote,
    min_receive_value: GemBigUint,
    eta_minutes: Option<u32>,
}

#[uniffi::export]
impl GemSwapQuoteSummary {
    #[uniffi::constructor]
    pub fn new(quote: SwapQuote) -> Self {
        Self {
            min_receive_value: rules::min_receive_value(&quote.to_value, quote.slippage_bps),
            eta_minutes: quote.eta_in_seconds.and_then(rules::eta_minutes),
            quote,
        }
    }

    #[uniffi::constructor]
    pub fn from_quote(quote: Quote) -> Self {
        Self::new(rules::swap_quote(&quote))
    }

    pub fn quote(&self) -> SwapQuote {
        self.quote.clone()
    }

    pub fn min_receive_value(&self) -> GemBigUint {
        self.min_receive_value.clone()
    }

    pub fn eta_minutes(&self) -> Option<u32> {
        self.eta_minutes
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapTransfer {
    pub quote: SwapQuote,
    pub data: SwapQuoteData,
    pub recipient: String,
    pub value: GemBigUint,
    pub use_max_amount: bool,
}

#[uniffi::export]
impl GemSwapTransfer {
    pub fn transfer_data(&self, from_asset: Asset, to_asset: Asset) -> GemTransferData {
        GemTransferData {
            input_type: TransactionInputType::Swap {
                from_asset,
                to_asset,
                swap_data: SwapData {
                    quote: self.quote.clone(),
                    data: self.data.clone(),
                },
            },
            recipient: GemRecipient {
                address: self.recipient.clone(),
                name: None,
                memo: self.data.memo.clone(),
                references: vec![],
            },
            value: self.value.clone().into(),
            use_max_amount: self.use_max_amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPair {
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapPairSuggestion {
    pub pay_asset_id: AssetId,
    pub receive_asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapButtonInput {
    pub value: GemBigInt,
    pub available_balance: GemBigInt,
    pub quote_error: Option<SwapperError>,
    pub transfer_error: Option<SwapperError>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSwapButtonAction {
    Swap,
    RetryQuote,
    RetryTransfer,
    UseMinimumAmount { value: GemBigInt },
    InsufficientBalance,
}
