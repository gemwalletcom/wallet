use std::collections::{BTreeMap, HashMap};
use std::io;

use number_formatter::{BigNumberFormatter, NumberFormatterError};
use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{Asset, Chain, FiatQuoteUrlData, FiatTransactionStatus, FiatTransactionUpdate, FiatWebhook, PaymentType};

use crate::model::{FiatProviderAsset, filter_token_id};

use super::{
    client::FlashnetClient,
    model::{FlashnetOnrampRequest, FlashnetOnrampResponse, FlashnetOrder, FlashnetRoute, FlashnetWebhookPayload},
};

fn map_chain(chain: &str) -> Option<Chain> {
    match chain {
        "bitcoin" => Some(Chain::Bitcoin),
        "solana" => Some(Chain::Solana),
        "base" => Some(Chain::Base),
        "ethereum" => Some(Chain::Ethereum),
        "arbitrum" => Some(Chain::Arbitrum),
        "optimism" => Some(Chain::Optimism),
        "polygon" => Some(Chain::Polygon),
        "tron" => Some(Chain::Tron),
        "plasma" => Some(Chain::Plasma),
        _ => None,
    }
}

pub fn map_assets(routes: Vec<FlashnetRoute>) -> Vec<FiatProviderAsset> {
    routes
        .into_iter()
        .filter_map(map_asset)
        .fold(BTreeMap::new(), |mut assets, asset| {
            assets.entry(asset.id.clone()).or_insert(asset);
            assets
        })
        .into_values()
        .collect()
}

fn map_asset(route: FlashnetRoute) -> Option<FiatProviderAsset> {
    let destination = route.destination;
    let chain = map_chain(&destination.chain)?;
    let token_id = filter_token_id(Some(chain), destination.contract_address);
    let symbol = destination.asset;
    if token_id.is_none() && symbol != Asset::from_chain(chain).symbol {
        return None;
    }
    let network = destination.chain;

    Some(FiatProviderAsset {
        id: format!("{}_{}", symbol.to_ascii_lowercase(), network),
        provider: FlashnetClient::NAME,
        chain: Some(chain),
        symbol,
        token_id,
        network: Some(network),
        enabled: true,
        is_buy_enabled: true,
        is_sell_enabled: false,
        unsupported_countries: Some(HashMap::new()),
        buy_limits: vec![FiatAssetLimits {
            currency: Currency::USD,
            payment_type: PaymentType::CashApp,
            min_amount: Some(1.0),
            max_amount: Some(500.0),
        }],
        sell_limits: vec![],
    })
}

const USDB_DECIMALS: u32 = 6;

pub fn map_source_amount(fiat_amount: f64) -> Result<String, NumberFormatterError> {
    BigNumberFormatter::value_from_amount(fiat_amount, USDB_DECIMALS)
}

pub fn map_onramp_request(data: &FiatQuoteUrlData, destination_chain: String, affiliate_id: String) -> FlashnetOnrampRequest {
    FlashnetOnrampRequest {
        destination_chain,
        destination_asset: data.asset_symbol.symbol.clone(),
        recipient_address: data.wallet_address.clone(),
        amount: data.quote.value.to_string(),
        amount_mode: "exact_out".to_string(),
        affiliate_id,
    }
}

pub fn map_crypto_amount(estimated_out: &str, decimals: u32) -> Result<f64, NumberFormatterError> {
    BigNumberFormatter::value_as_f64(estimated_out, decimals)
}

pub fn map_redirect_url(response: &FlashnetOnrampResponse) -> String {
    response.payment_links.cash_app.clone()
}

pub fn map_webhook(payload: FlashnetWebhookPayload) -> Result<FiatWebhook, io::Error> {
    match payload.event.as_str() {
        "order.processing" | "order.confirming" | "order.bridging" | "order.swapping" | "order.awaiting_approval" | "order.refunding" | "order.delivering" | "order.completed" | "order.failed" | "order.refunded" => {
            Ok(FiatWebhook::Transaction(map_order(payload.data.into_order().ok_or_else(|| io::Error::other("Missing Flashnet order fields in webhook"))?)))
        }
        _ => Ok(FiatWebhook::None),
    }
}

pub fn map_order(order: FlashnetOrder) -> FiatTransactionUpdate {
    FiatTransactionUpdate {
        transaction_id: order.id.clone(),
        provider_transaction_id: None,
        status: map_status(&order.status),
        transaction_hash: order.destination_tx_hash().map(str::to_string),
        fiat_amount: None,
        fiat_currency: Some(Currency::USD.to_string()),
    }
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "processing" | "confirming" | "bridging" | "swapping" | "awaiting_approval" | "refunding" | "delivering" => FiatTransactionStatus::Pending,
        "completed" => FiatTransactionStatus::Complete,
        "failed" | "refunded" => FiatTransactionStatus::Failed,
        _ => FiatTransactionStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::flashnet::model::{FlashnetOrder, FlashnetRoute};
    use primitives::{AssetId, FiatProviderName, FiatQuote};

    #[test]
    fn map_status_maps_all_documented_flashnet_statuses() {
        for status in ["processing", "confirming", "bridging", "swapping", "awaiting_approval", "refunding", "delivering"] {
            assert_eq!(map_status(status), FiatTransactionStatus::Pending);
        }
        assert_eq!(map_status("completed"), FiatTransactionStatus::Complete);
        assert_eq!(map_status("failed"), FiatTransactionStatus::Failed);
        assert_eq!(map_status("refunded"), FiatTransactionStatus::Failed);
        assert_eq!(map_status("unexpected"), FiatTransactionStatus::Unknown);
    }

    #[test]
    fn map_source_amount_uses_usdb_decimals() {
        assert_eq!(map_source_amount(1.0).unwrap(), "1000000");
        assert_eq!(map_source_amount(0.5).unwrap(), "500000");
        assert_eq!(map_source_amount(500.0).unwrap(), "500000000");
    }

    #[test]
    fn map_onramp_request_orders_quoted_amount() {
        let amount = |crypto_amount: &str| {
            let quote = FiatQuote {
                asset: Asset::from_chain(Chain::Polygon),
                crypto_amount: crypto_amount.parse().unwrap(),
                value: BigNumberFormatter::value_from_amount_biguint(crypto_amount, 18).unwrap(),
                ..FiatQuote::mock(FiatProviderName::Flashnet)
            };
            map_onramp_request(&FiatQuoteUrlData::mock(quote), "polygon".to_string(), "affiliate".to_string()).amount
        };

        assert_eq!(amount("400"), "400000000000000000000");
        assert_eq!(amount("100"), "100000000000000000000");
        assert_eq!(amount("18"), "18000000000000000000");
    }

    #[test]
    fn map_crypto_amount_parses_valid_values_and_rejects_invalid_ones() {
        assert_eq!(map_crypto_amount("1000000", 6).unwrap(), 1.0);
        assert_eq!(map_crypto_amount("500000", 6).unwrap(), 0.5);
        assert!(map_crypto_amount("invalid", 6).is_err());
    }

    #[test]
    fn map_assets_keeps_one_asset_per_supported_destination() {
        let usdc_solana = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let usdc_base = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
        let routes = vec![
            FlashnetRoute::mock("solana", "SOL", None),
            FlashnetRoute::mock("solana", "HSUSD", None),
            FlashnetRoute::mock("solana", "USDC", Some(usdc_solana)),
            FlashnetRoute::mock("solana", "USDC", Some(usdc_solana)),
            FlashnetRoute::mock("base", "USDC", Some(usdc_base)),
            FlashnetRoute::mock("tempo", "USDC", Some("0x20c000000000000000000000b9537d11c60e8b50")),
        ];

        let assets = map_assets(routes);

        assert_eq!(
            assets.iter().map(|asset| (asset.id.as_str(), asset.asset_id())).collect::<Vec<_>>(),
            vec![
                ("sol_solana", Some(AssetId::from_chain(Chain::Solana))),
                ("usdc_base", Some(AssetId::from_token(Chain::Base, usdc_base))),
                ("usdc_solana", Some(AssetId::from_token(Chain::Solana, usdc_solana))),
            ]
        );
    }

    #[test]
    fn map_webhook_maps_order_events_only() {
        let payload = |json: &str| serde_json::from_str::<FlashnetWebhookPayload>(json).unwrap();

        match map_webhook(payload(r#"{"event":"quote.updated","timestamp":"2026-03-13T00:00:00Z","data":{"id":"ord_123"}}"#)).unwrap() {
            FiatWebhook::None => {}
            other => panic!("Expected ignored webhook, got {other:?}"),
        }
        assert!(map_webhook(payload(r#"{"event":"order.completed","timestamp":"2026-03-13T00:00:00Z","data":{"id":"ord_123"}}"#)).is_err());
        match map_webhook(payload(include_str!("../../../testdata/flashnet/webhook_completed.json"))).unwrap() {
            FiatWebhook::Transaction(update) => assert_eq!(update, FiatTransactionUpdate::mock("ord_test_completed", FiatTransactionStatus::Complete, Some("solana_test_signature_completed"))),
            other => panic!("Expected transaction webhook, got {other:?}"),
        }
    }

    #[test]
    fn map_order_reads_destination_hash_and_never_fiat_amount() {
        let order = |json: &str| serde_json::from_str::<FlashnetOrder>(json).unwrap();

        assert_eq!(
            map_order(order(include_str!("../../../testdata/flashnet/order_completed.json"))),
            FiatTransactionUpdate::mock("ord_123", FiatTransactionStatus::Complete, Some("solana_sig_123"))
        );
        assert_eq!(
            map_order(order(include_str!("../../../testdata/flashnet/order_completed_eth.json"))),
            FiatTransactionUpdate::mock(
                "ord_019de25e-59d8-7ca6-b5e1-39651db9717f",
                FiatTransactionStatus::Complete,
                Some("0xf3fa9ca081e1f97022352c80345b46ae5934b0fae68c76ab5ccc70773ef1443e")
            )
        );
        assert_eq!(
            map_order(FlashnetOrder::mock_webhook(include_str!("../../../testdata/flashnet/webhook_completed.json"))),
            FiatTransactionUpdate::mock("ord_test_completed", FiatTransactionStatus::Complete, Some("solana_test_signature_completed"))
        );
        assert_eq!(
            map_order(FlashnetOrder::mock_webhook(include_str!("../../../testdata/flashnet/webhook_pending.json"))),
            FiatTransactionUpdate::mock("ord_test_pending", FiatTransactionStatus::Pending, None)
        );
    }
}
