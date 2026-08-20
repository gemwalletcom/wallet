use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::{AssetId, PaymentLink, PaymentOptions, PaymentOutcome, PaymentPrice, PaymentQuote, PaymentQuotes, PaymentStatus, WalletConnectCAIP19};
use std::str::FromStr;
use url::Url;

use crate::error::PaymentError;
use crate::wallet_connect_pay::account::account_chain;
use crate::wallet_connect_pay::model::{PaymentInfo, PaymentOption, PaymentOptionsResponse, PaymentStatusResponse};

const CAIP19_PREFIX: &str = "caip19";
const COLLECT_DATA_HOST: &str = "walletconnect.com";

pub fn map_options(response: PaymentOptionsResponse, payment_id: &str, accounts: &[String]) -> Result<PaymentOptions, PaymentError> {
    let payment = response.info.ok_or(PaymentError::PaymentNotFound)?;
    match payment.status {
        PaymentStatus::RequiresAction => {}
        PaymentStatus::Succeeded | PaymentStatus::Processing => {
            return Ok(PaymentOptions::Outcome(PaymentOutcome {
                status: payment.status,
                transaction_id: response.result_info.map(|info| info.tx_id),
            }));
        }
        PaymentStatus::Failed | PaymentStatus::Expired | PaymentStatus::Cancelled => return Err(PaymentError::PaymentExpired),
    }

    let expires_at = map_expiry(&payment)?;
    if expires_at <= Utc::now() {
        return Err(PaymentError::PaymentExpired);
    }

    let quotes = map_quotes(response.options.unwrap_or_default(), payment_id, accounts);
    if quotes.is_empty() {
        return Err(PaymentError::NoPaymentOptions);
    }

    Ok(PaymentOptions::Quotes(PaymentQuotes {
        price: BigUint::from_str(&payment.amount.value).ok().map(|value| PaymentPrice {
            symbol: payment.amount.display.asset_symbol,
            value,
            decimals: payment.amount.display.decimals,
        }),
        merchant: payment.merchant,
        expires_at: Some(expires_at),
        quotes,
    }))
}

fn map_quotes(options: Vec<PaymentOption>, payment_id: &str, accounts: &[String]) -> Vec<PaymentQuote> {
    let quotes = options.into_iter().filter_map(|option| map_quote(option, payment_id, accounts));
    let (ready, collecting): (Vec<PaymentQuote>, Vec<PaymentQuote>) = quotes.partition(|quote| quote.collect_data_url.is_none());
    ready.into_iter().chain(collecting).collect()
}

pub fn map_payment_outcome(response: PaymentStatusResponse) -> PaymentOutcome {
    PaymentOutcome {
        status: response.status,
        transaction_id: response.info.map(|info| info.tx_id),
    }
}

fn map_expiry(payment: &PaymentInfo) -> Result<DateTime<Utc>, PaymentError> {
    DateTime::from_timestamp(payment.expires_at, 0).ok_or_else(|| PaymentError::InvalidRequest("Invalid payment expiry".to_string()))
}

fn map_quote(option: PaymentOption, payment_id: &str, accounts: &[String]) -> Option<PaymentQuote> {
    if !accounts.iter().any(|account| account.eq_ignore_ascii_case(&option.account)) {
        return None;
    }
    let provider_data = serde_json::to_string(&option).ok()?;
    let collect_data_url = match &option.collect_data {
        Some(collect_data) => Some(collect_data_url(&collect_data.url)?),
        None => None,
    };
    let asset_id = coin_asset_id(&option.amount.unit)?;
    let value = BigUint::from_str(&option.amount.value).ok()?;
    if account_chain(&option.account)? != asset_id.chain {
        return None;
    }
    Some(PaymentQuote {
        id: option.id.clone(),
        link: PaymentLink::WalletConnectPay(payment_id.to_string()),
        expires_at: option.expires_at.and_then(|expires_at| DateTime::from_timestamp(expires_at, 0)),
        provider_data,
        asset_id,
        value,
        collect_data_url,
    })
}

fn collect_data_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    if parsed.scheme() != "https" {
        return None;
    }
    let host = parsed.host_str()?.to_lowercase();
    if host == COLLECT_DATA_HOST || host.ends_with(&format!(".{COLLECT_DATA_HOST}")) {
        Some(url.to_string())
    } else {
        None
    }
}

fn coin_asset_id(unit: &str) -> Option<AssetId> {
    let asset_id = match unit.split_once('/')? {
        (CAIP19_PREFIX, asset) => WalletConnectCAIP19::get_asset_id(asset)?,
        _ => return None,
    };
    asset_id.token_id.is_none().then_some(asset_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::model::PaymentResultInfo;
    use primitives::Chain;

    const ACCOUNT: &str = "eip155:1:0x1085c5f70F7F7591D97da281A64688385455c2bD";

    fn options_response() -> PaymentOptionsResponse {
        let mut response: PaymentOptionsResponse = serde_json::from_str(include_str!("../../testdata/options_response_coin.json")).unwrap();
        response.info.as_mut().unwrap().expires_at = Utc::now().timestamp() + 600;
        response
    }

    fn accounts() -> Vec<String> {
        vec![ACCOUNT.to_string()]
    }

    fn quotes(response: PaymentOptionsResponse) -> Vec<PaymentQuote> {
        match map_options(response, "pay_123", &accounts()).unwrap() {
            PaymentOptions::Quotes(quotes) => quotes.quotes,
            PaymentOptions::Outcome(outcome) => panic!("expected quotes, got {outcome:?}"),
        }
    }

    #[test]
    fn test_map_options_quotes_a_payment_in_the_coins_it_can_pay_with() {
        let PaymentOptions::Quotes(quotes) = map_options(options_response(), "pay_123", &accounts()).unwrap() else {
            panic!("expected quotes");
        };

        assert_eq!(quotes.merchant.name, "Gem Wallet Test Merchant");
        assert_eq!(quotes.price.unwrap().symbol, "USD");

        let quote = &quotes.quotes[0];
        assert_eq!(quote.id, "opt_eth");
        assert_eq!(quote.link, PaymentLink::WalletConnectPay("pay_123".to_string()));
        assert_eq!(quote.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(quote.collect_data_url, None);
    }

    #[test]
    fn test_map_options_refuses_a_payment_it_can_no_longer_pay() {
        let mut expired = options_response();
        expired.info.as_mut().unwrap().expires_at = Utc::now().timestamp() - 1;
        assert_eq!(map_options(expired, "pay_123", &accounts()), Err(PaymentError::PaymentExpired));

        let mut cancelled = options_response();
        cancelled.info.as_mut().unwrap().status = PaymentStatus::Cancelled;
        assert_eq!(map_options(cancelled, "pay_123", &accounts()), Err(PaymentError::PaymentExpired));
    }

    #[test]
    fn test_map_options_offers_quotes_asking_for_no_personal_data_first() {
        let mut response = options_response();
        let options = response.options.as_mut().unwrap();
        options[1] = serde_json::from_str(include_str!("../../testdata/option_collect_data.json")).unwrap();
        options.swap(0, 1);

        let quotes = quotes(response);

        assert_eq!(quotes.len(), 2);
        assert!(quotes[0].collect_data_url.is_none());
        assert!(quotes[1].collect_data_url.is_some());
    }

    #[test]
    fn test_asset_id_reads_the_chain_coin_and_refuses_a_token() {
        assert_eq!(coin_asset_id("caip19/eip155:1/slip44:60"), Some(AssetId::from(Chain::Ethereum, None)));
        assert_eq!(coin_asset_id("caip19/eip155:1").unwrap(), AssetId::from(Chain::Ethereum, None));

        assert_eq!(coin_asset_id("caip19/eip155:8453/erc20:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"), None);
        assert_eq!(coin_asset_id("caip19/eip155:99999/erc20:0x1"), None);
        assert_eq!(coin_asset_id("eip155:1/erc20:0x1"), None);
        assert_eq!(coin_asset_id("iso4217/USD"), None);
    }

    #[test]
    fn test_map_option_refuses_an_account_off_the_quoted_chain() {
        let mut option: PaymentOption = serde_json::from_str(include_str!("../../testdata/option_collect_data.json")).unwrap();
        option.account = "eip155:137:0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string();

        assert_eq!(map_quote(option.clone(), "pay_123", &[option.account.clone()]), None);
    }

    #[test]
    fn test_map_option_refuses_an_account_the_wallet_did_not_offer() {
        let option: PaymentOption = serde_json::from_str(include_str!("../../testdata/option_collect_data.json")).unwrap();

        assert_eq!(map_quote(option, "pay_123", &["eip155:1:0xdeadbeef".to_string()]), None);
    }

    #[test]
    fn test_map_option_with_collect_data() {
        let option: PaymentOption = serde_json::from_str(include_str!("../../testdata/option_collect_data.json")).unwrap();
        let mapped = map_quote(option.clone(), "pay_123", &accounts()).unwrap();

        assert_eq!(mapped.collect_data_url.unwrap(), "https://data-collection.walletconnect.com/ic/pay_123");

        for url in [
            "https://evil.com/ic/pay_123",
            "https://evil-walletconnect.com/ic/pay_123",
            "https://walletconnect.com.evil.com/ic/pay_123",
            "http://walletconnect.com/ic/pay_123",
            "not a url",
        ] {
            let mut off_domain = option.clone();
            off_domain.collect_data.as_mut().unwrap().url = url.to_string();

            assert!(map_quote(off_domain, "pay_123", &accounts()).is_none(), "{url} must not reach the collection web view");
        }
    }

    #[test]
    fn test_map_options_refuses_a_payment_with_nothing_payable() {
        let mut response = options_response();
        response.options.as_mut().unwrap().clear();

        assert_eq!(map_options(response, "pay_123", &accounts()), Err(PaymentError::NoPaymentOptions));
    }

    #[test]
    fn test_map_options_drops_a_quote_priced_in_a_token() {
        let quotes = quotes(options_response());

        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].asset_id, AssetId::from_chain(Chain::Ethereum));
    }

    #[test]
    fn test_map_options_drops_an_option_quoted_against_another_wallet() {
        let mut response = options_response();
        let options = response.options.as_mut().unwrap();
        options[1] = options[0].clone();
        options[1].id = "opt_stranger".to_string();
        options[1].account = "eip155:1:0x1234567890123456789012345678901234567890".to_string();

        let quotes = quotes(response);

        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].id, "opt_eth");
    }

    #[test]
    fn test_map_payment_options_without_info() {
        let response = PaymentOptionsResponse {
            info: None,
            options: None,
            result_info: None,
        };

        assert_eq!(map_options(response, "pay_123", &accounts()), Err(PaymentError::PaymentNotFound));
    }

    #[test]
    fn test_map_options_keeps_the_transaction_of_a_settled_payment() {
        let mut response = options_response();
        response.info.as_mut().unwrap().status = PaymentStatus::Succeeded;
        response.result_info = Some(PaymentResultInfo { tx_id: "test:pay_1".to_string() });

        assert_eq!(
            map_options(response, "pay_123", &accounts()).unwrap(),
            PaymentOptions::Outcome(PaymentOutcome {
                status: PaymentStatus::Succeeded,
                transaction_id: Some("test:pay_1".to_string()),
            })
        );
    }

    #[test]
    fn test_map_payment_outcome() {
        let response: PaymentStatusResponse = serde_json::from_str(include_str!("../../testdata/status_response_succeeded.json")).unwrap();
        let outcome = map_payment_outcome(response);

        assert_eq!(outcome.status, PaymentStatus::Succeeded);
        assert_eq!(outcome.transaction_id.unwrap(), "test:pay_b9a2ecc101KYJAYCGQZ9E0K6NY7SR7YVV4");
    }
}
