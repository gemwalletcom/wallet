use std::collections::HashMap;

use primitives::currency::Currency;
use primitives::fiat_assets::FiatAssetLimits;
use primitives::{AssetId, Chain, FiatAssetSymbol, FiatProviderCountry, FiatQuoteType, FiatTransactionStatus, FiatTransactionUpdate, FiatWebhook, PaymentType};

use crate::error::FiatQuoteError;
use crate::model::{FiatMapping, FiatProviderAsset, filter_token_id};

use super::client::FonbnkClient;
use super::model::{FonbnkCurrency, FonbnkCurrencyChannel, FonbnkOrder, FonbnkOrderLimits, FonbnkOrderWebhookData, FonbnkPaymentChannel, FonbnkQuote, FonbnkQuoteLeg, FonbnkWebhook};

const FIAT_CURRENCY_TYPE: &str = "fiat";
const CRYPTO_CURRENCY_TYPE: &str = "crypto";
const CRYPTO_CHANNEL: &str = "crypto";
const NATIVE_ASSET: &str = "NATIVE";
const ORDER_STATUS_CHANGE_EVENT: &str = "order-status-change";

const FIAT_CHANNELS: [(&str, FonbnkPaymentChannel, PaymentType); 2] = [("mobile_money", FonbnkPaymentChannel::MobileMoney, PaymentType::MobileMoney), ("bank", FonbnkPaymentChannel::Bank, PaymentType::BankTransfer)];

fn map_chain(network: &str) -> Option<Chain> {
    match network {
        "ARBITRUM" => Some(Chain::Arbitrum),
        "AVALANCHE" => Some(Chain::AvalancheC),
        "BASE" => Some(Chain::Base),
        "BNB" => Some(Chain::SmartChain),
        "CELO" => Some(Chain::Celo),
        "ETHEREUM" => Some(Chain::Ethereum),
        "OPTIMISM" => Some(Chain::Optimism),
        "POLYGON" => Some(Chain::Polygon),
        "SOLANA" => Some(Chain::Solana),
        "STELLAR" => Some(Chain::Stellar),
        "TEMPO" => Some(Chain::Tempo),
        "TON" => Some(Chain::Ton),
        "TRON" => Some(Chain::Tron),
        "XRP" => Some(Chain::Xrp),
        _ => None,
    }
}

pub fn map_payment_methods() -> Vec<PaymentType> {
    FIAT_CHANNELS.into_iter().map(|(_, _, payment_type)| payment_type).collect()
}

pub fn map_assets(currencies: Vec<FonbnkCurrency>) -> Vec<FiatProviderAsset> {
    currencies.into_iter().filter(|currency| currency.currency_type == CRYPTO_CURRENCY_TYPE).filter_map(map_asset).collect()
}

fn map_asset(currency: FonbnkCurrency) -> Option<FiatProviderAsset> {
    let network = currency.currency_details.network?;
    let symbol = currency.currency_details.asset?;
    let chain = map_chain(&network)?;
    let token_id = map_token_id(chain, &symbol, currency.currency_details.contract_address);
    if token_id.is_none() && symbol != NATIVE_ASSET {
        return None;
    }
    let channel = currency.payment_channels.iter().find(|channel| channel.channel == CRYPTO_CHANNEL)?;

    Some(FiatProviderAsset {
        id: currency.currency_code,
        provider: FonbnkClient::NAME,
        chain: Some(chain),
        symbol,
        token_id,
        network: Some(network),
        enabled: true,
        is_buy_enabled: channel.is_payout_allowed,
        is_sell_enabled: channel.is_deposit_allowed,
        unsupported_countries: Some(HashMap::new()),
        buy_limits: map_limits(),
        sell_limits: map_limits(),
    })
}

fn map_token_id(chain: Chain, symbol: &str, contract_address: Option<String>) -> Option<String> {
    if symbol == NATIVE_ASSET {
        return None;
    }
    let contract_address = contract_address.filter(|contract_address| !contract_address.is_empty())?;
    let token_id = if chain == Chain::Stellar { AssetId::sub_token_id(&[contract_address, symbol.to_string()]) } else { contract_address };
    filter_token_id(Some(chain), Some(token_id))
}

fn map_limits() -> Vec<FiatAssetLimits> {
    map_payment_methods()
        .into_iter()
        .map(|payment_type| FiatAssetLimits {
            currency: Currency::USD,
            payment_type,
            min_amount: None,
            max_amount: None,
        })
        .collect()
}

pub fn map_countries(currencies: &[FonbnkCurrency]) -> Vec<FiatProviderCountry> {
    currencies
        .iter()
        .filter(|currency| currency.currency_type == FIAT_CURRENCY_TYPE)
        .filter_map(|currency| {
            let alpha2 = currency.currency_details.country_iso_code.clone()?;
            let is_allowed = [FiatQuoteType::Buy, FiatQuoteType::Sell]
                .into_iter()
                .any(|quote_type| map_fiat_leg(std::slice::from_ref(currency), &alpha2, quote_type).is_some());
            Some(FiatProviderCountry {
                provider: FonbnkClient::NAME,
                alpha2,
                is_allowed,
            })
        })
        .collect()
}

pub fn map_fiat_leg(currencies: &[FonbnkCurrency], country_code: &str, quote_type: FiatQuoteType) -> Option<(FonbnkQuoteLeg, PaymentType)> {
    let currency = currencies
        .iter()
        .find(|currency| currency.currency_type == FIAT_CURRENCY_TYPE && currency.currency_details.country_iso_code.as_deref() == Some(country_code))?;

    FIAT_CHANNELS.into_iter().find_map(|(name, channel, payment_type)| {
        currency
            .payment_channels
            .iter()
            .any(|entry| entry.channel == name && is_fiat_channel_allowed(entry, quote_type))
            .then(|| (FonbnkQuoteLeg::fiat(currency.currency_code.clone(), country_code.to_string(), channel), payment_type))
    })
}

fn is_fiat_channel_allowed(channel: &FonbnkCurrencyChannel, quote_type: FiatQuoteType) -> bool {
    match quote_type {
        FiatQuoteType::Buy => channel.is_deposit_allowed,
        FiatQuoteType::Sell => channel.is_payout_allowed,
    }
}

pub fn map_crypto_leg(asset_symbol: &FiatAssetSymbol) -> Result<FonbnkQuoteLeg, FiatQuoteError> {
    let network = FiatMapping::get_network(asset_symbol.network.clone())?;
    Ok(FonbnkQuoteLeg::crypto(format!("{network}_{}", asset_symbol.symbol)))
}

pub fn map_deposit_amount(limits: &FonbnkOrderLimits, usd_amount: f64) -> Result<f64, FiatQuoteError> {
    if !limits.supported {
        let reason = limits.unsupported_reason.as_ref().map(|reason| reason.message.clone()).unwrap_or_default();
        return Err(FiatQuoteError::UnsupportedState(reason));
    }
    let deposit = &limits.deposit;
    if usd_amount < deposit.min_usd {
        return Err(FiatQuoteError::MinimumAmount(deposit.min_usd));
    }
    if usd_amount > deposit.max_usd {
        return Err(FiatQuoteError::InvalidRequest(format!("Maximum amount is {}", deposit.max_usd)));
    }
    Ok(round_to_step(usd_amount * deposit.max / deposit.max_usd, deposit.step).clamp(deposit.min, deposit.max))
}

pub fn map_buy_crypto_amount(limits: &FonbnkOrderLimits, quote: &FonbnkQuote, usd_amount: f64) -> f64 {
    let crypto_amount = quote.payout.cashout.amount_after_fees * usd_amount / quote.deposit.cashout.amount_before_fees_usd;
    round_to_step(crypto_amount, limits.payout.step)
}

pub fn map_sell_crypto_amount(limits: &FonbnkOrderLimits, quote: &FonbnkQuote, usd_amount: f64) -> f64 {
    let crypto_amount = quote.deposit.cashout.amount_before_fees * usd_amount / quote.payout.cashout.amount_after_fees_usd;
    round_to_step(crypto_amount, limits.deposit.step)
}

fn round_to_step(value: f64, step: f64) -> f64 {
    let scale = 10f64.powi(-step.log10().round() as i32);
    (value * scale).round() / scale
}

pub fn map_webhook(payload: FonbnkWebhook) -> Result<FiatWebhook, serde_json::Error> {
    if payload.event != ORDER_STATUS_CHANGE_EVENT {
        return Ok(FiatWebhook::None);
    }
    let order = serde_json::from_value::<FonbnkOrderWebhookData>(payload.data)?.order;
    Ok(map_order(order).map_or(FiatWebhook::None, FiatWebhook::Transaction))
}

fn map_order(order: FonbnkOrder) -> Option<FiatTransactionUpdate> {
    let transaction_id = order.merchant_order_params?;
    let (fiat_leg, crypto_leg, fiat_amount) = match order.order_type.as_str() {
        "on_ramp" => (&order.deposit, &order.payout, order.deposit.cashout.amount_before_fees),
        "off_ramp" => (&order.payout, &order.deposit, order.payout.cashout.amount_after_fees),
        _ => return None,
    };

    Some(FiatTransactionUpdate {
        transaction_id,
        provider_transaction_id: Some(order.id),
        status: map_status(&order.status),
        transaction_hash: crypto_leg.transaction_hash(),
        fiat_amount: Some(fiat_amount),
        fiat_currency: Some(fiat_leg.currency_code.clone()),
    })
}

fn map_status(status: &str) -> FiatTransactionStatus {
    match status {
        "deposit_awaiting" | "deposit_validating" | "deposit_successful" | "payout_pending" | "payout_failed" | "refund_initiated" | "refund_pending" | "refund_failed" => FiatTransactionStatus::Pending,
        "payout_successful" => FiatTransactionStatus::Complete,
        "deposit_invalid" | "deposit_canceled" | "deposit_expired" | "refund_successful" => FiatTransactionStatus::Failed,
        _ => FiatTransactionStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::BASE_USDC_ASSET_ID;

    fn currencies() -> Vec<FonbnkCurrency> {
        serde_json::from_str(include_str!("../../../testdata/fonbnk/currencies.json")).unwrap()
    }

    fn buy_limits() -> FonbnkOrderLimits {
        serde_json::from_str(include_str!("../../../testdata/fonbnk/order_limits_buy.json")).unwrap()
    }

    #[test]
    fn test_map_assets() {
        let assets = map_assets(currencies());
        let asset = |id: &str| assets.iter().find(|asset| asset.id == id);

        assert_eq!(asset("BASE_USDC").unwrap().asset_id(), Some(BASE_USDC_ASSET_ID.clone()));
        assert_eq!(asset("ETHEREUM_NATIVE").unwrap().asset_id(), Some(AssetId::from_chain(Chain::Ethereum)));
        assert_eq!(asset("POLYGON_NATIVE").unwrap().asset_id(), Some(AssetId::from_chain(Chain::Polygon)));
        assert!(asset("TON_NATIVE").is_some_and(|asset| !asset.is_buy_enabled && asset.is_sell_enabled));
        assert!(asset("LISK_USDT").is_none());
        assert!(asset("XRP_RLUSD").is_none());
    }

    #[test]
    fn test_map_countries() {
        let countries = map_countries(&currencies());
        let is_allowed = |alpha2: &str| countries.iter().find(|country| country.alpha2 == alpha2).map(|country| country.is_allowed);

        assert_eq!(is_allowed("NG"), Some(true));
        assert_eq!(is_allowed("PH"), Some(true));
        assert_eq!(is_allowed("CO"), Some(false));
        assert_eq!(is_allowed("US"), None);
    }

    #[test]
    fn test_map_fiat_leg() {
        let currencies = currencies();
        let leg = |country_code: &str, quote_type: FiatQuoteType| map_fiat_leg(&currencies, country_code, quote_type).map(|(leg, payment_type)| (leg.currency_code, leg.payment_channel, payment_type));

        assert_eq!(leg("NG", FiatQuoteType::Buy), Some(("NGN".to_string(), FonbnkPaymentChannel::Bank, PaymentType::BankTransfer)));
        assert_eq!(leg("KE", FiatQuoteType::Sell), Some(("KES".to_string(), FonbnkPaymentChannel::MobileMoney, PaymentType::MobileMoney)));
        assert_eq!(leg("SN", FiatQuoteType::Buy), Some(("XOF".to_string(), FonbnkPaymentChannel::MobileMoney, PaymentType::MobileMoney)));
        assert_eq!(leg("PH", FiatQuoteType::Buy), None);
        assert_eq!(leg("PH", FiatQuoteType::Sell), Some(("PHP".to_string(), FonbnkPaymentChannel::Bank, PaymentType::BankTransfer)));
    }

    #[test]
    fn test_map_deposit_amount() {
        let limits = buy_limits();

        assert_eq!(map_deposit_amount(&limits, 50.0).unwrap(), 71705.0);
        assert_eq!(map_deposit_amount(&limits, 500.0).unwrap(), 717051.0);
        assert_eq!(map_deposit_amount(&limits, 0.5).unwrap_err().to_string(), "Minimum Amount is 1");
        assert!(map_deposit_amount(&limits, 501.0).is_err());
        assert!(map_deposit_amount(&FonbnkOrderLimits { supported: false, ..limits }, 50.0).is_err());
    }

    #[test]
    fn test_map_buy_crypto_amount() {
        let quote: FonbnkQuote = serde_json::from_str(include_str!("../../../testdata/fonbnk/quote_buy.json")).unwrap();

        assert_eq!(map_buy_crypto_amount(&buy_limits(), &quote, 50.0), 48.748907);
    }

    #[test]
    fn test_map_sell_crypto_amount() {
        let limits: FonbnkOrderLimits = serde_json::from_str(include_str!("../../../testdata/fonbnk/order_limits_sell.json")).unwrap();
        let quote: FonbnkQuote = serde_json::from_str(include_str!("../../../testdata/fonbnk/quote_sell.json")).unwrap();

        assert_eq!(map_sell_crypto_amount(&limits, &quote, 50.0), 0.02262942);
    }

    #[test]
    fn test_map_webhook_on_ramp() {
        let payload: FonbnkWebhook = serde_json::from_str(include_str!("../../../testdata/fonbnk/webhook_on_ramp.json")).unwrap();

        assert_eq!(
            map_webhook(payload).unwrap(),
            FiatWebhook::Transaction(FiatTransactionUpdate {
                transaction_id: "4f6b2c1e-8d3a-4b5e-9f7a-2c1d0e9b8a76".to_string(),
                provider_transaction_id: Some("68df90a2372f378356ef75c1".to_string()),
                status: FiatTransactionStatus::Complete,
                transaction_hash: Some("0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060".to_string()),
                fiat_amount: Some(71705.0),
                fiat_currency: Some("NGN".to_string()),
            })
        );
    }

    #[test]
    fn test_map_webhook_off_ramp() {
        let payload: FonbnkWebhook = serde_json::from_str(include_str!("../../../testdata/fonbnk/webhook_off_ramp.json")).unwrap();

        assert_eq!(
            map_webhook(payload).unwrap(),
            FiatWebhook::Transaction(FiatTransactionUpdate {
                transaction_id: "9a1c7e5b-2f4d-4c8a-b6e3-1d0f9a8b7c65".to_string(),
                provider_transaction_id: Some("68e0a1b2c3d4e5f60718293a".to_string()),
                status: FiatTransactionStatus::Pending,
                transaction_hash: Some("4f1e5c0b2d9a8e7f6c5b4a39281706f5e4d3c2b1a09f8e7d6c5b4a3928170605".to_string()),
                fiat_amount: Some(6084.0),
                fiat_currency: Some("KES".to_string()),
            })
        );
    }

    #[test]
    fn test_map_webhook_ignores_other_events() {
        let payload = FonbnkWebhook {
            event: "kyc".to_string(),
            data: serde_json::json!({}),
        };

        assert_eq!(map_webhook(payload).unwrap(), FiatWebhook::None);
    }
}
