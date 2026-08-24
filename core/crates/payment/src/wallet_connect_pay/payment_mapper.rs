use num_bigint::BigUint;
use primitives::{AssetId, PaymentLink, PaymentOptions, PaymentOutcome, PaymentPrice, PaymentQuote, PaymentQuotes, PaymentStatus, WalletConnectCAIP19};
use std::str::FromStr;
use url::Url;

use crate::error::PaymentError;
use crate::wallet_connect_pay::account::get_chain;
use crate::wallet_connect_pay::model::{PaymentOption, PaymentOptionsResponse, PaymentStatusResponse};

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
    if get_chain(&option.account)? != asset_id.chain {
        return None;
    }
    Some(PaymentQuote {
        id: option.id.clone(),
        link: PaymentLink::WalletConnectPay(payment_id.to_string()),
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
    use crate::wallet_connect_pay::testkit::{TEST_ACCOUNT_POLYGON, TEST_PAYMENT_ID, mock_accounts};
    use primitives::Chain;

    fn quotes(response: PaymentOptionsResponse) -> Vec<PaymentQuote> {
        match map_options(response, TEST_PAYMENT_ID, &mock_accounts()).unwrap() {
            PaymentOptions::Quotes(quotes) => quotes.quotes,
            PaymentOptions::Outcome(outcome) => panic!("expected quotes, got {outcome:?}"),
        }
    }

    fn collects_data_from(url: &str) -> Option<String> {
        map_quote(PaymentOption::mock_collect_data_from(url), TEST_PAYMENT_ID, &mock_accounts())?.collect_data_url
    }

    #[test]
    fn test_map_options() {
        let PaymentOptions::Quotes(quotes) = map_options(PaymentOptionsResponse::mock(), TEST_PAYMENT_ID, &mock_accounts()).unwrap() else {
            panic!("expected quotes");
        };
        assert_eq!(quotes.merchant.name, "Gem Wallet Test Merchant");
        assert_eq!(quotes.price.unwrap().symbol, "USD");

        assert_eq!(quotes.quotes.len(), 1, "the token option must be dropped");
        let quote = &quotes.quotes[0];
        assert_eq!(quote.id, "opt_eth");
        assert_eq!(quote.link, PaymentLink::WalletConnectPay(TEST_PAYMENT_ID.to_string()));
        assert_eq!(quote.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(quote.collect_data_url, None);
    }

    #[test]
    fn test_map_options_refuses_a_payment_it_cannot_quote() {
        let map = |response| map_options(response, TEST_PAYMENT_ID, &mock_accounts());

        assert_eq!(map(PaymentOptionsResponse::mock_with_status(PaymentStatus::Expired)), Err(PaymentError::PaymentExpired));
        assert_eq!(map(PaymentOptionsResponse::mock_with_status(PaymentStatus::Cancelled)), Err(PaymentError::PaymentExpired));
        assert_eq!(map(PaymentOptionsResponse::mock_with_status(PaymentStatus::Failed)), Err(PaymentError::PaymentExpired));
        assert_eq!(map(PaymentOptionsResponse::mock_without_info()), Err(PaymentError::PaymentNotFound));

        let mut nothing_payable = PaymentOptionsResponse::mock();
        nothing_payable.options.as_mut().unwrap().clear();
        assert_eq!(map(nothing_payable), Err(PaymentError::NoPaymentOptions));
    }

    #[test]
    fn test_map_options_keeps_the_transaction_of_a_settled_payment() {
        let mut response = PaymentOptionsResponse::mock_with_status(PaymentStatus::Succeeded);
        response.result_info = Some(PaymentResultInfo { tx_id: "test:pay_1".to_string() });

        assert_eq!(
            map_options(response, TEST_PAYMENT_ID, &mock_accounts()).unwrap(),
            PaymentOptions::Outcome(PaymentOutcome {
                status: PaymentStatus::Succeeded,
                transaction_id: Some("test:pay_1".to_string()),
            })
        );
    }

    #[test]
    fn test_map_quotes_offers_quotes_asking_for_no_personal_data_first() {
        let mut response = PaymentOptionsResponse::mock();
        let options = response.options.as_mut().unwrap();
        options[1] = PaymentOption::mock_collect_data();
        options.swap(0, 1);

        let quotes = quotes(response);

        assert_eq!(quotes.len(), 2);
        assert!(quotes[0].collect_data_url.is_none());
        assert!(quotes[1].collect_data_url.is_some());
    }

    #[test]
    fn test_map_quote_refuses_an_account_the_wallet_cannot_sign_for() {
        let mut off_chain = PaymentOption::mock_collect_data();
        off_chain.account = TEST_ACCOUNT_POLYGON.to_string();
        assert_eq!(map_quote(off_chain.clone(), TEST_PAYMENT_ID, &[off_chain.account.clone()]), None);

        let stranger = PaymentOption::mock_collect_data();
        assert_eq!(map_quote(stranger, TEST_PAYMENT_ID, &["eip155:1:0xdeadbeef".to_string()]), None);
    }

    #[test]
    fn test_map_quote_only_collects_data_on_the_gateway_domain() {
        assert_eq!(
            collects_data_from("https://data-collection.walletconnect.com/ic/pay_123"),
            Some("https://data-collection.walletconnect.com/ic/pay_123".to_string())
        );

        assert_eq!(collects_data_from("https://evil.com/ic/pay_123"), None);
        assert_eq!(collects_data_from("https://evil-walletconnect.com/ic/pay_123"), None);
        assert_eq!(collects_data_from("https://walletconnect.com.evil.com/ic/pay_123"), None);
        assert_eq!(collects_data_from("http://walletconnect.com/ic/pay_123"), None);
        assert_eq!(collects_data_from("not a url"), None);
    }

    #[test]
    fn test_coin_asset_id() {
        assert_eq!(coin_asset_id("caip19/eip155:1/slip44:60"), Some(AssetId::from(Chain::Ethereum, None)));
        assert_eq!(coin_asset_id("caip19/eip155:1").unwrap(), AssetId::from(Chain::Ethereum, None));

        assert_eq!(coin_asset_id("caip19/eip155:8453/erc20:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"), None);
        assert_eq!(coin_asset_id("caip19/eip155:99999/erc20:0x1"), None);
        assert_eq!(coin_asset_id("eip155:1/erc20:0x1"), None);
        assert_eq!(coin_asset_id("iso4217/USD"), None);
    }

    #[test]
    fn test_map_payment_outcome() {
        let outcome = map_payment_outcome(PaymentStatusResponse::mock_succeeded());

        assert_eq!(outcome.status, PaymentStatus::Succeeded);
        assert_eq!(outcome.transaction_id.unwrap(), "test:pay_b9a2ecc101KYJAYCGQZ9E0K6NY7SR7YVV4");
    }
}
