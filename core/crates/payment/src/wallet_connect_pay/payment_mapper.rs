use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, PaymentAmount, PaymentInvoice, PaymentLink, PaymentMerchant, PaymentPrice, PaymentQuote, PaymentRequest, PaymentStatus, TransactionType,
    WalletConnectCAIP2, WalletConnectCAIP19,
};
use std::str::FromStr;
use url::Url;

use crate::PaymentTransaction;
use crate::error::PaymentError;
use crate::wallet_connect_pay::model::{Invoice, Options, PaymentAction, PaymentOption, PaymentOptionsResponse, PaymentPriceAmount, Quote};

const CAIP19_PREFIX: &str = "caip19";
const ISO4217_PREFIX: &str = "iso4217/";
const COLLECT_DATA_HOST: &str = "walletconnect.com";

pub(super) fn map_options(response: PaymentOptionsResponse, accounts: &[String]) -> Result<Options, PaymentError> {
    let payment = response.info.ok_or(PaymentError::InvalidRequest {
        reason: "Payment not found".to_string(),
    })?;
    if payment.status != PaymentStatus::RequiresAction {
        return Ok(Options::Status { status: payment.status });
    }

    let quotes = map_quotes(response.options.unwrap_or_default(), accounts);
    if quotes.is_empty() {
        return Err(PaymentError::NoPaymentOptions);
    }

    Ok(Options::Invoice(Invoice {
        merchant: payment.merchant,
        price: map_price(&payment.amount)?,
        quotes,
    }))
}

fn map_price(amount: &PaymentPriceAmount) -> Result<PaymentPrice, PaymentError> {
    let invalid = || PaymentError::InvalidRequest {
        reason: format!("Invalid payment amount {}", amount.value),
    };
    let value = BigUint::from_str(&amount.value).map_err(|_| invalid())?;
    Ok(PaymentPrice {
        currency: amount.unit.strip_prefix(ISO4217_PREFIX).unwrap_or(&amount.unit).to_string(),
        amount: BigNumberFormatter::value_as_f64(&value.to_string(), amount.display.decimals).map_err(|_| invalid())?,
    })
}

pub(super) fn map_invoice(invoice: &Invoice, payment_id: &str) -> PaymentInvoice {
    PaymentInvoice {
        link: PaymentLink::WalletConnectPay {
            payment_id: payment_id.to_string(),
        },
        merchant: PaymentMerchant {
            name: invoice.merchant.name.clone(),
            icon: invoice.merchant.icon_url.clone().unwrap_or_default(),
        },
        price: Some(invoice.price.clone()),
        quotes: invoice.quotes.iter().map(map_payment_quote).collect(),
    }
}

pub(super) fn status_reason(status: PaymentStatus) -> String {
    match status {
        PaymentStatus::RequiresAction | PaymentStatus::Processing => "Payment is already in progress",
        PaymentStatus::Succeeded => "Payment is already paid",
        PaymentStatus::Failed => "Payment has failed",
        PaymentStatus::Expired => "Payment has expired",
        PaymentStatus::Cancelled => "Payment was cancelled",
    }
    .to_string()
}

pub(super) fn map_transaction(quote: &Quote, action: PaymentAction, invoice: PaymentInvoice) -> PaymentTransaction {
    let transaction_type = if action.data.is_empty() {
        TransactionType::Transfer
    } else {
        TransactionType::SmartContractCall
    };

    PaymentTransaction {
        invoice,
        account: action.account,
        transaction: action.data,
        transaction_type,
        memo: None,
        request: Some(PaymentRequest {
            address: action.recipient,
            amount: Some(PaymentAmount::AtomicValue { value: action.value }),
            memo: None,
            label: None,
            references: None,
            asset_id: Some(quote.asset_id.clone()),
        }),
    }
}

fn map_payment_quote(quote: &Quote) -> PaymentQuote {
    PaymentQuote {
        id: quote.id.clone(),
        asset_id: quote.asset_id.clone(),
        value: quote.value.clone(),
    }
}

fn map_quotes(options: Vec<PaymentOption>, accounts: &[String]) -> Vec<Quote> {
    options.into_iter().filter_map(|option| map_quote(option, accounts)).collect()
}

fn map_quote(option: PaymentOption, accounts: &[String]) -> Option<Quote> {
    let account = accounts.iter().find(|account| account.eq_ignore_ascii_case(&option.account))?;
    let account = WalletConnectCAIP2::parse_account(account.clone())?;
    let collect_data_url = match &option.collect_data {
        Some(collect_data) => Some(get_collect_data_url(&collect_data.url)?),
        None => None,
    };
    let asset_id = get_coin_asset_id(&option.amount.unit)?;
    let value = BigUint::from_str(&option.amount.value).ok()?;
    if account.chain != asset_id.chain {
        return None;
    }
    Some(Quote {
        id: option.id,
        account,
        asset_id,
        value,
        collect_data_url,
        actions: option.actions,
    })
}

fn get_collect_data_url(url: &str) -> Option<String> {
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

fn get_coin_asset_id(unit: &str) -> Option<AssetId> {
    let asset_id = match unit.split_once('/')? {
        (CAIP19_PREFIX, asset) => WalletConnectCAIP19::get_asset_id(asset)?,
        _ => return None,
    };
    asset_id.token_id.is_none().then_some(asset_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::model::{Merchant, PaymentPriceDisplay};

    fn amount(unit: &str, value: &str) -> PaymentPriceAmount {
        PaymentPriceAmount {
            unit: unit.to_string(),
            value: value.to_string(),
            display: PaymentPriceDisplay { decimals: 2 },
        }
    }

    #[test]
    fn test_map_price() {
        assert_eq!(
            map_price(&amount("iso4217/USD", "30")).unwrap(),
            PaymentPrice {
                currency: "USD".to_string(),
                amount: 0.30,
            }
        );
        assert_eq!(map_price(&amount("EUR", "5")).unwrap().currency, "EUR");
        assert!(map_price(&amount("iso4217/USD", "0.30")).is_err());
    }

    #[test]
    fn test_map_invoice() {
        let invoice = map_invoice(
            &Invoice {
                merchant: Merchant {
                    name: "Gem Wallet Test Merchant".to_string(),
                    icon_url: None,
                },
                price: map_price(&amount("iso4217/USD", "30")).unwrap(),
                quotes: vec![],
            },
            "pay_123",
        );

        assert_eq!(invoice.link, PaymentLink::WalletConnectPay { payment_id: "pay_123".to_string() });
        assert_eq!(invoice.link.url(), "https://pay.walletconnect.com/?pid=pay_123");
        assert_eq!(invoice.merchant, PaymentMerchant { name: "Gem Wallet Test Merchant".to_string(), icon: String::new() });
        assert_eq!(invoice.price.as_ref().map(|price| price.amount), Some(0.30));
        assert!(invoice.quotes.is_empty());
    }
}
