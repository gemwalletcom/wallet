use super::error::{PaymentDecoderError, Result};
use url::Url;

use crate::payment::{Payment, PaymentLink};
use crate::url_query::query_value;
use crate::{HTTPS_URL_SCHEME, WALLET_CONNECT_URL_SCHEME, WalletConnectLink};

pub const WALLET_CONNECT_PAY_HOST: &str = "pay.walletconnect.com";
const WALLET_CONNECT_PAY_HOST_SUFFIX: &str = ".pay.walletconnect.com";
const PAYMENT_ID_PREFIX: &str = "pay_";
const PAYMENT_ID_EXTRA_CHARACTERS: &str = "-._~";

const QUERY_PAYMENT_ID: &str = "pid";
const QUERY_PAY: &str = "pay";

pub fn decode(uri: &str) -> Result<Payment> {
    let url = Url::parse(uri).map_err(|_| PaymentDecoderError::InvalidScheme)?;
    let payment_id = payment_id(&url).ok_or(PaymentDecoderError::InvalidScheme)?;
    Ok(Payment::Link {
        link: PaymentLink::WalletConnectPay { payment_id },
    })
}

fn payment_id(url: &Url) -> Option<String> {
    match url.scheme() {
        HTTPS_URL_SCHEME => from_payment_url(url),
        WALLET_CONNECT_URL_SCHEME => from_pairing_uri(url),
        _ => None,
    }
}

fn from_pairing_uri(url: &Url) -> Option<String> {
    let WalletConnectLink::Connect { uri } = WalletConnectLink::from_url(url.as_str())? else {
        return None;
    };
    let payment_url = query_value(&Url::parse(&uri).ok()?, QUERY_PAY)?;
    from_payment_url(&Url::parse(&payment_url).ok()?)
}

fn from_payment_url(url: &Url) -> Option<String> {
    if !is_payment_host(url) {
        return None;
    }
    query_value(url, QUERY_PAYMENT_ID)
        .or_else(|| Some(url.path().trim_matches('/').to_string()))
        .filter(|payment_id| is_payment_id(payment_id))
}

fn is_payment_id(payment_id: &str) -> bool {
    payment_id.starts_with(PAYMENT_ID_PREFIX)
        && payment_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || PAYMENT_ID_EXTRA_CHARACTERS.contains(character))
}

fn is_payment_host(url: &Url) -> bool {
    url.scheme() == HTTPS_URL_SCHEME
        && url
            .host_str()
            .is_some_and(|host| host == WALLET_CONNECT_PAY_HOST || host.ends_with(WALLET_CONNECT_PAY_HOST_SUFFIX))
}
