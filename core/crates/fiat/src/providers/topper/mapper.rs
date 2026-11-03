use std::collections::HashMap;

use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{Asset, AssetId, Chain, FiatAssetSymbol, FiatProviderCountry, FiatTransactionStatus, FiatTransactionUpdate, FiatWebhook, PaymentType};

use crate::error::FiatQuoteError;
use crate::model::{FiatMapping, FiatProviderAsset, filter_token_id};

use super::client::TopperClient;
use super::model::{
    TopperAsset, TopperAssetNetwork, TopperAssetType, TopperAssetsResponse, TopperCountriesResponse, TopperCountryHint, TopperEvent, TopperOnRamp, TopperOrder, TopperPaymentMethodType, TopperPaymentMethodsResponse, TopperRecipient,
    TopperRecipientEditMode, TopperSource, TopperWebhook,
};

fn map_chain(network: &str) -> Option<Chain> {
    match network {
        "algorand" => Some(Chain::Algorand),
        "arbitrum" => Some(Chain::Arbitrum),
        "arc" => Some(Chain::Arc),
        "asset-hub" => Some(Chain::Polkadot),
        "avalanche" => Some(Chain::AvalancheC),
        "base" => Some(Chain::Base),
        "bitcoin" => Some(Chain::Bitcoin),
        "bitcoin-cash" => Some(Chain::BitcoinCash),
        "bnb-smart-chain" => Some(Chain::SmartChain),
        "cardano" => Some(Chain::Cardano),
        "celo" => Some(Chain::Celo),
        "cosmos" => Some(Chain::Cosmos),
        "dash" => Some(Chain::Dash),
        "dogecoin" => Some(Chain::Doge),
        "ethereum" => Some(Chain::Ethereum),
        "hyperliquid-hyperevm" => Some(Chain::Hyperliquid),
        "ink" => Some(Chain::Ink),
        "litecoin" => Some(Chain::Litecoin),
        "mantle" => Some(Chain::Mantle),
        "monad" => Some(Chain::Monad),
        "near" => Some(Chain::Near),
        "optimism" => Some(Chain::Optimism),
        "plasma" => Some(Chain::Plasma),
        "polygon" => Some(Chain::Polygon),
        "robinhood-chain" => Some(Chain::Robinhood),
        "solana" => Some(Chain::Solana),
        "sonic" => Some(Chain::Sonic),
        "stellar" => Some(Chain::Stellar),
        "sui" => Some(Chain::Sui),
        "tempo" => Some(Chain::Tempo),
        "ton" => Some(Chain::Ton),
        "tron" => Some(Chain::Tron),
        "unichain" => Some(Chain::Unichain),
        "xrp-ledger" => Some(Chain::Xrp),
        _ => None,
    }
}

fn map_payment_type(method_type: TopperPaymentMethodType) -> Option<PaymentType> {
    match method_type {
        TopperPaymentMethodType::CreditCard | TopperPaymentMethodType::DebitCard => Some(PaymentType::Card),
        TopperPaymentMethodType::ApplePay => Some(PaymentType::ApplePay),
        TopperPaymentMethodType::GooglePay => Some(PaymentType::GooglePay),
        TopperPaymentMethodType::Sepa => Some(PaymentType::Sepa),
        TopperPaymentMethodType::Venmo => Some(PaymentType::Venmo),
        TopperPaymentMethodType::Paypal | TopperPaymentMethodType::Unknown => None,
    }
}

pub fn map_limits(response: &TopperPaymentMethodsResponse) -> Vec<FiatAssetLimits> {
    let mut limits: Vec<FiatAssetLimits> = Vec::new();
    for method in &response.payment_methods {
        let Some(payment_type) = map_payment_type(method.method_type) else {
            continue;
        };
        for limit in method.limits.iter().flatten().filter(|limit| limit.asset == Currency::USD.as_ref()) {
            match limits.iter_mut().find(|existing| existing.payment_type == payment_type) {
                Some(existing) => {
                    existing.min_amount = existing.min_amount.map(|minimum| minimum.min(limit.minimum));
                    existing.max_amount = existing.max_amount.map(|maximum| maximum.max(limit.maximum));
                }
                None => limits.push(FiatAssetLimits {
                    currency: Currency::USD,
                    payment_type: payment_type.clone(),
                    min_amount: Some(limit.minimum),
                    max_amount: Some(limit.maximum),
                }),
            }
        }
    }
    limits
}

pub fn map_assets(response: TopperAssetsResponse, buy_limits: &[FiatAssetLimits]) -> Vec<FiatProviderAsset> {
    response.assets.target.iter().flat_map(|asset| asset.networks.iter().filter_map(move |network| map_asset(asset, network, buy_limits))).collect()
}

fn map_asset(asset: &TopperAsset, network: &TopperAssetNetwork, buy_limits: &[FiatAssetLimits]) -> Option<FiatProviderAsset> {
    let chain = map_chain(&network.code)?;
    let token_id = match network.asset_type.as_ref()? {
        TopperAssetType::Coin if Asset::from_chain(chain).symbol == asset.code => None,
        TopperAssetType::Token => Some(map_token_id(chain, &asset.code, network.asset_contract_address.clone()?)?),
        TopperAssetType::Coin | TopperAssetType::Unknown => return None,
    };

    Some(FiatProviderAsset {
        id: format!("{}_{}", network.code, asset.code),
        provider: TopperClient::NAME,
        chain: Some(chain),
        symbol: asset.code.clone(),
        token_id,
        network: Some(network.code.clone()),
        enabled: true,
        is_buy_enabled: true,
        is_sell_enabled: false,
        unsupported_countries: Some(HashMap::new()),
        buy_limits: buy_limits.to_vec(),
        sell_limits: vec![],
    })
}

fn map_token_id(chain: Chain, symbol: &str, contract_address: String) -> Option<String> {
    let token_id = if chain == Chain::Stellar {
        AssetId::sub_token_id(&[contract_address, symbol.to_string()])
    } else if chain == Chain::Xrp {
        contract_address.split_once(':').map_or(contract_address.clone(), |(issuer, _)| issuer.to_string())
    } else {
        contract_address
    };
    filter_token_id(Some(chain), Some(token_id))
}

pub fn map_countries(response: TopperCountriesResponse) -> Vec<FiatProviderCountry> {
    response
        .countries
        .into_iter()
        .map(|country| FiatProviderCountry {
            provider: TopperClient::NAME,
            alpha2: country.alpha2,
            is_allowed: true,
        })
        .collect()
}

pub fn map_on_ramp(fiat_amount: f64, fiat_currency: &str, asset_symbol: &FiatAssetSymbol, address: Option<&str>, country_code: &str) -> Result<TopperOnRamp, FiatQuoteError> {
    let recipient_edit_mode = match address {
        Some(_) => TopperRecipientEditMode::NotEditable,
        None => TopperRecipientEditMode::OnlyAddressAndTag,
    };
    Ok(TopperOnRamp {
        source: TopperSource {
            amount: fiat_amount.to_string(),
            asset: fiat_currency.to_string(),
        },
        target: TopperRecipient {
            asset: asset_symbol.symbol.clone(),
            network: FiatMapping::get_network(asset_symbol.network.clone())?,
            address: address.map(str::to_string),
            recipient_edit_mode,
        },
        simulation: TopperCountryHint { country: country_code.to_string() },
    })
}

pub fn map_webhook(payload: TopperWebhook) -> Result<FiatWebhook, FiatQuoteError> {
    let Some(status) = map_status(&payload.name) else {
        return Ok(FiatWebhook::None);
    };
    let transaction_id = payload.bootstrap_token_id.ok_or(FiatQuoteError::InvalidWebhook)?;
    let order = serde_json::from_value::<TopperOrder>(payload.data).map_err(|_| FiatQuoteError::InvalidWebhook)?;

    Ok(FiatWebhook::Transaction(FiatTransactionUpdate {
        transaction_id,
        provider_transaction_id: Some(order.id),
        status,
        transaction_hash: order.destination.ledger.map(|ledger| ledger.txid),
        fiat_amount: Some(order.origin.amount),
        fiat_currency: Some(order.origin.asset),
    }))
}

fn map_status(event: &TopperEvent) -> Option<FiatTransactionStatus> {
    match event {
        TopperEvent::Committed | TopperEvent::Charged | TopperEvent::CryptoSent => Some(FiatTransactionStatus::Pending),
        TopperEvent::Completed => Some(FiatTransactionStatus::Complete),
        TopperEvent::Expired | TopperEvent::Failed | TopperEvent::Refunded => Some(FiatTransactionStatus::Failed),
        TopperEvent::Other => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSETS: &str = include_str!("../../../testdata/topper/assets_crypto_onramp.json");
    const PAYMENT_METHODS: &str = include_str!("../../../testdata/topper/payment_methods_crypto_onramp.json");
    const WEBHOOK_COMPLETED: &str = include_str!("../../../testdata/topper/webhook_order_completed.json");

    #[test]
    fn test_map_assets() {
        let assets = map_assets(serde_json::from_str(ASSETS).unwrap(), &[]);

        assert_eq!(
            assets.iter().map(|asset| (asset.id.as_str(), asset.chain.unwrap(), asset.token_id.as_deref())).collect::<Vec<_>>(),
            vec![
                ("bitcoin_BTC", Chain::Bitcoin, None),
                ("asset-hub_DOT", Chain::Polkadot, None),
                ("arbitrum_ETH", Chain::Arbitrum, None),
                ("base_ETH", Chain::Base, None),
                ("ethereum_ETH", Chain::Ethereum, None),
                ("ton_GRAM", Chain::Ton, None),
                ("base_USDC", Chain::Base, Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")),
                ("solana_USDC", Chain::Solana, Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")),
                ("stellar_USDC", Chain::Stellar, Some("GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC")),
                ("xrp-ledger_USDC", Chain::Xrp, Some("rGm7WCVp9gb4jZHWTEtGUr4dd74z2XuWhE")),
            ]
        );
        assert_eq!(assets.iter().map(|asset| (asset.is_buy_enabled, asset.is_sell_enabled)).collect::<Vec<_>>(), vec![(true, false); 10]);
    }

    #[test]
    fn test_map_limits() {
        let limits = map_limits(&serde_json::from_str(PAYMENT_METHODS).unwrap());

        assert_eq!(
            limits.iter().map(|limit| (limit.payment_type.clone(), limit.currency.clone(), limit.min_amount, limit.max_amount)).collect::<Vec<_>>(),
            vec![
                (PaymentType::ApplePay, Currency::USD, Some(10.0), Some(67014.71)),
                (PaymentType::Card, Currency::USD, Some(10.0), Some(50000.0)),
                (PaymentType::Venmo, Currency::USD, Some(10.0), Some(50000.0)),
            ]
        );
    }

    #[test]
    fn test_map_on_ramp() {
        let asset_symbol = FiatAssetSymbol {
            symbol: "USDC".to_string(),
            network: Some("base".to_string()),
        };

        assert_eq!(
            map_on_ramp(100.0, "USD", &asset_symbol, None, "GB").unwrap(),
            TopperOnRamp {
                source: TopperSource {
                    amount: "100".to_string(),
                    asset: "USD".to_string(),
                },
                target: TopperRecipient {
                    asset: "USDC".to_string(),
                    network: "base".to_string(),
                    address: None,
                    recipient_edit_mode: TopperRecipientEditMode::OnlyAddressAndTag,
                },
                simulation: TopperCountryHint { country: "GB".to_string() },
            }
        );
        assert_eq!(
            map_on_ramp(99.5, "USD", &asset_symbol, Some("0x41018795fA95783117242244303fd7e26e964eE8"), "US").unwrap().target,
            TopperRecipient {
                asset: "USDC".to_string(),
                network: "base".to_string(),
                address: Some("0x41018795fA95783117242244303fd7e26e964eE8".to_string()),
                recipient_edit_mode: TopperRecipientEditMode::NotEditable,
            }
        );
        assert!(map_on_ramp(100.0, "USD", &FiatAssetSymbol { network: None, ..asset_symbol }, None, "US").is_err());
    }

    #[test]
    fn test_map_webhook() {
        assert_eq!(
            map_webhook(serde_json::from_str(WEBHOOK_COMPLETED).unwrap()).unwrap(),
            FiatWebhook::Transaction(FiatTransactionUpdate {
                transaction_id: "3f2c1b9e-7a4d-4c8e-9b1f-5d6e7a8b9c0d".to_string(),
                provider_transaction_id: Some("966b8e24-6a65-442a-942e-577f16288789".to_string()),
                status: FiatTransactionStatus::Complete,
                transaction_hash: Some("0xdfa1ea4ddb841af466a5ca78c6e1f0edfaef5e54e79a28238d2a5bb2da4f1911".to_string()),
                fiat_amount: Some(100.0),
                fiat_currency: Some("USD".to_string()),
            })
        );
        assert_eq!(
            map_webhook(serde_json::from_str(r#"{"name":"user:created","id":"00a991cf-a870-43af-88b8-43c51532831d","data":{}}"#).unwrap()).unwrap(),
            FiatWebhook::None
        );
    }
}
