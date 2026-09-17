use primitives::{AssetBasic, AssetId, Chain};

use super::model::{GemWidgetCoin, GemWidgetSize};
use crate::formatted_number::GemFormattedNumber;
use crate::percentage::GemPercentageStyle;
use crate::precision::GemCurrencyStyle;

pub const REFRESH_INTERVAL_SECONDS: u32 = 900;

pub fn coin_ids(size: GemWidgetSize) -> Vec<AssetId> {
    let chains: &[Chain] = match size {
        GemWidgetSize::Small => &[Chain::Bitcoin],
        GemWidgetSize::Medium => &[Chain::Bitcoin, Chain::Ethereum, Chain::Solana],
        GemWidgetSize::Large => &[Chain::Bitcoin, Chain::Ethereum, Chain::Solana, Chain::Xrp, Chain::SmartChain],
    };
    chains.iter().map(|chain| AssetId::from_chain(*chain)).collect()
}

pub fn price_style(size: GemWidgetSize) -> GemCurrencyStyle {
    match size {
        GemWidgetSize::Small => GemCurrencyStyle::Abbreviated,
        GemWidgetSize::Medium | GemWidgetSize::Large => GemCurrencyStyle::Fiat,
    }
}

pub fn coins(ids: &[AssetId], assets: Vec<AssetBasic>, currency: &str, size: GemWidgetSize) -> Vec<GemWidgetCoin> {
    let style = price_style(size);
    ids.iter()
        .filter_map(|id| assets.iter().find(|asset| &asset.asset.id == id))
        .filter_map(|asset| {
            let price = asset.price.as_ref()?;
            Some(GemWidgetCoin {
                asset_id: asset.asset.id.clone(),
                name: asset.asset.name.clone(),
                symbol: asset.asset.symbol.clone(),
                price: GemFormattedNumber::currency_code(price.price, currency.to_string(), style),
                change: GemFormattedNumber::percentage(price.price_change_percentage_24h, GemPercentageStyle::Signed),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatted_number::{GemNumberDisplay, GemValueTone};
    use primitives::Asset;

    #[test]
    fn the_small_widget_shows_one_coin_and_the_large_five() {
        assert_eq!(coin_ids(GemWidgetSize::Small), vec![AssetId::from_chain(Chain::Bitcoin)]);
        assert_eq!(coin_ids(GemWidgetSize::Medium).len(), 3);
        assert_eq!(coin_ids(GemWidgetSize::Large).len(), 5);
    }

    #[test]
    fn coins_keep_the_widget_order_and_skip_an_asset_without_a_price() {
        let ids = coin_ids(GemWidgetSize::Medium);
        let assets = vec![
            AssetBasic::mock_with_price(Chain::Solana, 150.0, -3.5),
            Asset::from_chain(Chain::Ethereum).as_basic_primitive(),
            AssetBasic::mock_with_price(Chain::Bitcoin, 69_000.0, -3.5),
        ];
        let coins = coins(&ids, assets, "USD", GemWidgetSize::Medium);
        assert_eq!(coins.iter().map(|coin| coin.symbol.as_str()).collect::<Vec<_>>(), vec!["BTC", "SOL"]);
        assert_eq!(coins[0].change.tone, GemValueTone::Negative);
        assert!(matches!(coins[0].price.display, GemNumberDisplay::Number { .. }));
    }

    #[test]
    fn the_small_widget_abbreviates_its_price() {
        let ids = coin_ids(GemWidgetSize::Small);
        let coins = coins(&ids, vec![AssetBasic::mock_with_price(Chain::Bitcoin, 690_000.0, -3.5)], "USD", GemWidgetSize::Small);
        assert!(matches!(coins[0].price.display, GemNumberDisplay::Abbreviated));
    }
}
