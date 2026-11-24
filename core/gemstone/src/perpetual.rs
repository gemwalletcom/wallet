use gem_hypercore::{
    models::{perp_dex::PerpDex, websocket::HyperliquidSubscription},
    perpetual_formatter::PerpetualFormatter,
};
use primitives::contract_constants::HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS;
use primitives::known_assets::{ARBITRUM_USDC, HYPERCORE_SPOT_USDC};
use primitives::{Asset, PerpetualAccountMode, PerpetualProvider, PerpetualType};

use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::GemBigInt;
use crate::models::perpetual::GemPerpetualSubscription;
use crate::services::error::GemServiceError;
use crate::services::transfer::model::{GemRecipient, GemTransferData};
use primitives::TransactionInputType;

const HYPERLIQUID_NAME: &str = "Hyperliquid";

#[derive(Debug)]
pub struct GemPerpetual {
    provider: PerpetualProvider,
}

impl GemPerpetual {
    pub fn new(provider: PerpetualProvider) -> Self {
        Self { provider }
    }

    pub fn format_price(&self, price: f64, decimals: u32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_price(price, decimals),
        }
    }

    pub fn price_decimals(&self, price: f64, decimals: u32) -> usize {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::price_decimals(price, decimals),
        }
    }

    pub fn display_price(&self, price: f64, decimals: u32) -> GemFormattedNumber {
        GemFormattedNumber::usd(price).with_places(self.price_decimals(price, decimals))
    }

    pub fn format_input_price(&self, price: f64, decimals: u32, decimal_separator: String) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_input_price(price, decimals, decimal_separator.chars().next().unwrap_or('.')),
        }
    }

    pub fn deposit_asset(&self) -> Asset {
        match self.provider {
            PerpetualProvider::Hypercore => ARBITRUM_USDC.clone(),
        }
    }

    pub fn recipient(&self) -> GemRecipient {
        GemRecipient {
            address: String::new(),
            name: Some(self.name().to_string()),
            memo: None,
            references: vec![],
        }
    }

    pub fn deposit_assets(&self, mode: PerpetualAccountMode) -> Vec<Asset> {
        match mode {
            PerpetualAccountMode::Standard => vec![self.deposit_asset(), HYPERCORE_SPOT_USDC.clone()],
            PerpetualAccountMode::Unified => vec![self.deposit_asset()],
        }
    }

    pub fn deposit_recipient(&self, asset: &Asset, owner: GemRecipient) -> Result<GemRecipient, GemServiceError> {
        if asset.id == self.deposit_asset().id {
            Ok(GemRecipient {
                address: HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS.to_string(),
                ..self.recipient()
            })
        } else if asset.id == HYPERCORE_SPOT_USDC.id {
            Ok(owner)
        } else {
            Err(GemServiceError::Unsupported {
                msg: format!("perpetual deposit from {}", asset.id),
            })
        }
    }

    pub fn format_size(&self, size: f64, decimals: u32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_size(size, decimals),
        }
    }

    pub fn transfer_data(&self, asset: Asset, perpetual_type: PerpetualType, value: GemBigInt, use_max_amount: bool) -> GemTransferData {
        GemTransferData {
            input_type: TransactionInputType::Perpetual { asset, perpetual_type },
            recipient: self.recipient(),
            value,
            use_max_amount,
        }
    }

    fn name(&self) -> &'static str {
        match self.provider {
            PerpetualProvider::Hypercore => HYPERLIQUID_NAME,
        }
    }
}

impl GemPerpetualSubscription {
    pub(crate) fn map(self, dexes: &[PerpDex]) -> Vec<HyperliquidSubscription> {
        match self {
            Self::Candle { symbol, interval } => vec![HyperliquidSubscription::Candle { symbol, interval }],
            Self::MarketData { symbol } => vec![HyperliquidSubscription::MarketData { symbol }],
            Self::MarketPrices => dexes.iter().map(|dex| HyperliquidSubscription::MarketPrices { dex: dex.name() }).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_subscription() {
        assert_eq!(
            serde_json::to_value(GemPerpetualSubscription::MarketPrices.map(&[PerpDex::Main, PerpDex::Xyz])).unwrap(),
            serde_json::json!([{"type": "allMids"}, {"type": "allMids", "dex": "xyz"}]),
            "market prices come from every dex the app loads"
        );
        assert_eq!(
            GemPerpetualSubscription::MarketData { symbol: "xyz:NVDA".to_string() }.map(&[PerpDex::Main, PerpDex::Xyz]),
            vec![HyperliquidSubscription::MarketData { symbol: "xyz:NVDA".to_string() }]
        );
    }
}
