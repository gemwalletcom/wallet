use chrono::{DateTime, Utc};
use primitives::{AssetId, PaymentAmount, PaymentOutcome, PaymentPrice, WalletConnectCAIP19};
use url::Url;

use crate::error::PaymentError;
use crate::wallet_connect_pay::account::account_chain;
use crate::wallet_connect_pay::model::{PaymentInfo, PaymentOption, PaymentOptionsResponse, PaymentStatusResponse};
use crate::wallet_connect_pay::quote::{QuotedOption, QuotedPayment};

const CAIP19_PREFIX: &str = "caip19";
const COLLECT_DATA_HOST: &str = "walletconnect.com";

pub fn map_quoted_payment(response: PaymentOptionsResponse) -> Result<QuotedPayment, PaymentError> {
    let payment = response.info.ok_or(PaymentError::PaymentNotFound)?;
    Ok(QuotedPayment {
        status: payment.status,
        expires_at: map_expiry(&payment)?,
        price: PaymentPrice {
            symbol: payment.amount.display.asset_symbol.clone(),
            value: payment.amount.value.clone(),
            decimals: payment.amount.display.decimals,
        },
        merchant: payment.merchant,
        options: response.options.unwrap_or_default().into_iter().filter_map(map_option).collect(),
    })
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

fn map_option(option: PaymentOption) -> Option<QuotedOption> {
    let provider_data = serde_json::to_string(&option).ok()?;
    let collect_data_url = match &option.collect_data {
        Some(collect_data) => Some(collect_data_url(&collect_data.url)?),
        None => None,
    };
    Some(QuotedOption {
        id: option.id.clone(),
        expires_at: option.expires_at.and_then(|expires_at| DateTime::from_timestamp(expires_at, 0)),
        chain: account_chain(&option.account)?,
        provider_data,
        amount: PaymentAmount {
            asset_id: asset_id(&option.amount.unit)?,
            value: option.amount.value,
            symbol: option.amount.display.asset_symbol,
            decimals: option.amount.display.decimals,
        },
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

fn asset_id(unit: &str) -> Option<AssetId> {
    match unit.split_once('/')? {
        (CAIP19_PREFIX, asset) => WalletConnectCAIP19::get_asset_id(asset),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, PaymentStatus};

    fn options_response() -> PaymentOptionsResponse {
        serde_json::from_str(include_str!("../../testdata/options_response.json")).unwrap()
    }

    #[test]
    fn test_map_quoted_payment() {
        let quoted = map_quoted_payment(options_response()).unwrap();

        assert_eq!(quoted.status, PaymentStatus::RequiresAction);
        assert_eq!(quoted.merchant.name, "Gem Wallet Test Merchant");
        assert_eq!(quoted.expires_at.timestamp(), 1785175272);

        let option = &quoted.options[0];
        assert_eq!(option.id, "opt_1");
        assert_eq!(option.chain, Chain::Ethereum);
        assert_eq!(option.collect_data_url, None);
    }

    #[test]
    fn test_asset_id_reads_the_chain_coin_and_its_tokens() {
        assert_eq!(asset_id("caip19/eip155:1/slip44:60"), Some(AssetId::from(Chain::Ethereum, None)));
        assert_eq!(
            asset_id("caip19/eip155:8453/erc20:0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
            Some(AssetId::from_token(Chain::Base, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"))
        );
        assert_eq!(asset_id("caip19/eip155:1").unwrap(), AssetId::from(Chain::Ethereum, None));
        assert_eq!(asset_id("caip19/eip155:99999/erc20:0x1"), None);
        assert_eq!(asset_id("eip155:1/erc20:0x1"), None);
        assert_eq!(asset_id("iso4217/USD"), None);
    }

    #[test]
    fn test_map_option_with_collect_data() {
        let option: PaymentOption = serde_json::from_str(include_str!("../../testdata/option_collect_data.json")).unwrap();
        let mapped = map_option(option.clone()).unwrap();

        assert_eq!(mapped.collect_data_url.unwrap(), "https://data-collection.walletconnect.com/ic/pay_123");

        for url in [
            "https://evil.com/ic/pay_123",
            "https://evil-walletconnect.com/ic/pay_123",
            "http://walletconnect.com/ic/pay_123",
            "not a url",
        ] {
            let mut off_domain = option.clone();
            off_domain.collect_data.as_mut().unwrap().url = url.to_string();

            assert!(map_option(off_domain).is_none(), "{url} must not reach the collection web view");
        }
    }

    #[test]
    fn test_map_payment_options_drops_unpayable_option() {
        let mut response = options_response();
        let options = response.options.as_mut().unwrap();
        options[0].account = "cosmos:cosmoshub-4:cosmos1".to_string();

        assert!(map_quoted_payment(response).unwrap().options.is_empty());
    }

    #[test]
    fn test_map_payment_options_without_info() {
        let response = PaymentOptionsResponse { info: None, options: None };

        assert_eq!(map_quoted_payment(response), Err(PaymentError::PaymentNotFound));
    }

    #[test]
    fn test_map_payment_outcome() {
        let response: PaymentStatusResponse = serde_json::from_str(include_str!("../../testdata/status_response_succeeded.json")).unwrap();
        let outcome = map_payment_outcome(response);

        assert_eq!(outcome.status, PaymentStatus::Succeeded);
        assert_eq!(outcome.transaction_id.unwrap(), "test:pay_b9a2ecc101KYJAYCGQZ9E0K6NY7SR7YVV4");
    }
}
