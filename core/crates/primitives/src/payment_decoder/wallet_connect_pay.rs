use super::error::{PaymentDecoderError, Result};
use url::Url;

use crate::url_query::query_value;
use crate::{HTTP_URL_SCHEME, HTTPS_URL_SCHEME, WalletConnectLink};

pub const WALLET_CONNECT_PAY_HOST: &str = "pay.walletconnect.com";
const WALLET_CONNECT_PAY_HOST_SUFFIX: &str = ".pay.walletconnect.com";
const PAYMENT_ID_PREFIX: &str = "pay_";
const PAYMENT_ID_EXTRA_CHARACTERS: &str = "-._~";

const QUERY_PAYMENT_ID: &str = "pid";
const QUERY_PAY: &str = "pay";

#[derive(Debug, Clone, PartialEq)]
pub struct WalletConnectPayLink {
    pub payment_id: String,
}

pub fn parse(uri: &str) -> Result<WalletConnectPayLink> {
    let url = Url::parse(uri)?;
    let payment_id = payment_id(&url).ok_or(PaymentDecoderError::InvalidScheme)?;
    Ok(WalletConnectPayLink { payment_id })
}

fn payment_id(url: &Url) -> Option<String> {
    match url.scheme() {
        HTTP_URL_SCHEME | HTTPS_URL_SCHEME => from_payment_url(url),
        _ => from_pairing_uri(url),
    }
}

fn from_payment_url(url: &Url) -> Option<String> {
    if !is_payment_host(url) {
        return None;
    }
    query_value(url, QUERY_PAYMENT_ID)
        .or_else(|| Some(url.path().trim_matches('/').to_string()))
        .filter(|payment_id| is_payment_id(payment_id))
}

pub fn is_payment_id(payment_id: &str) -> bool {
    payment_id.starts_with(PAYMENT_ID_PREFIX)
        && payment_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || PAYMENT_ID_EXTRA_CHARACTERS.contains(character))
}

fn from_pairing_uri(url: &Url) -> Option<String> {
    let WalletConnectLink::Connect { uri } = WalletConnectLink::from_url(url.as_str())? else {
        return None;
    };
    let pairing_uri = Url::parse(&uri).ok()?;
    let payment_url = Url::parse(&query_value(&pairing_uri, QUERY_PAY)?).ok()?;
    from_payment_url(&payment_url)
}

fn is_payment_host(url: &Url) -> bool {
    url.scheme() == "https"
        && url
            .host_str()
            .is_some_and(|host| host == WALLET_CONNECT_PAY_HOST || host.ends_with(WALLET_CONNECT_PAY_HOST_SUFFIX))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payment_id(uri: &str) -> Option<String> {
        parse(uri).ok().map(|payment| payment.payment_id)
    }

    #[test]
    fn test_parse_payment_url() {
        assert_eq!(payment_id("https://pay.walletconnect.com/?pid=pay_1"), Some("pay_1".to_string()));
        assert_eq!(payment_id("http://pay.walletconnect.com/?pid=pay_1"), None);
        assert_eq!(payment_id("https://pay.walletconnect.com/pay_1"), Some("pay_1".to_string()));
        assert_eq!(payment_id("https://staging.pay.walletconnect.com/?pid=pay_1"), Some("pay_1".to_string()));

        assert_eq!(payment_id("https://pay.walletconnect.com"), None);
        assert_eq!(payment_id("https://pay.walletconnect.com/terms"), None);
        assert_eq!(payment_id("https://pay.walletconnect.com/?pid=checkout"), None);
        assert_eq!(payment_id("https://notpay.walletconnect.com/pay_1"), None);
        assert_eq!(payment_id("https://pay.walletconnect.com.attacker.io/pay_1"), None);
        assert_eq!(payment_id("https://gemwallet.com/tokens/bitcoin"), None);
    }

    #[test]
    fn test_is_payment_id() {
        assert!(is_payment_id("pay_b9a2ecc101KYJAYCGQZ9E0K6NY7SR7YVV4"));
        assert!(is_payment_id("pay_1-2.3~4"));

        assert!(!is_payment_id("checkout"));
        assert!(!is_payment_id(""));
        assert!(!is_payment_id("pay_../../admin"));
        assert!(!is_payment_id("pay_1?maxPollMs=0"));
        assert!(!is_payment_id("pay_1/status"));
        assert!(!is_payment_id("pay_1 2"));
    }

    #[test]
    fn test_parse_pairing_uri() {
        assert_eq!(payment_id("wc:abc@2?pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_1"), Some("pay_1".to_string()));
        assert_eq!(payment_id("wc:abc@2?pay=https://pay.walletconnect.com/?pid=pay_1"), Some("pay_1".to_string()));
        assert_eq!(payment_id("wc:abc@2?pay=https%3A%2F%2Fmerchant.example.com%2Fcheckout"), None);
        assert_eq!(payment_id("wc:abc@2?relay-protocol=irn&symKey=123"), None);
    }

    #[test]
    fn test_parse_deep_link() {
        assert_eq!(
            payment_id("gem://wc?uri=wc%3Aabc%402%3Fpay%3Dhttps%253A%252F%252Fpay.walletconnect.com%252F%253Fpid%253Dpay_1"),
            Some("pay_1".to_string())
        );
        assert_eq!(payment_id("gem://wc?uri=wc%3Atopic%402%3Frelay-protocol%3Dirn%26symKey%3Dabc"), None);
        assert_eq!(payment_id("gem://wc?sessionTopic=abc"), None);
    }
}
