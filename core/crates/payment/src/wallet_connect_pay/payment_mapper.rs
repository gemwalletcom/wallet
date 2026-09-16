use gem_evm::address::ethereum_address_checksum;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, PaymentAmount, PaymentInvoice, PaymentLink, PaymentMerchant, PaymentPrice, PaymentQuote, PaymentRequest, PaymentStatus, TransactionType, TransferDataOutputType,
    WalletConnectCAIP2, WalletConnectCAIP19, payment_decoder::is_payment_host,
};
use std::str::FromStr;
use url::Url;

use crate::PaymentTransaction;
use crate::error::PaymentError;
use crate::wallet_connect_pay::model::{Invoice, Options, PaymentAction, PaymentOption, PaymentOptionsResponse, PaymentPriceAmount, Quote};

const CAIP19_PREFIX: &str = "caip19";
const ISO4217_PREFIX: &str = "iso4217/";

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
        collect_data_url: response.collect_data.and_then(|collect_data| get_collect_data_url(&collect_data.url)),
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
        verification: None,
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
    let (transaction, transaction_type, recipient, output_type, approval) = match action {
        PaymentAction::Send(send) => {
            let transaction_type = if send.data.is_empty() {
                TransactionType::Transfer
            } else {
                TransactionType::SmartContractCall
            };
            (send.data, transaction_type, send.recipient, TransferDataOutputType::EncodedTransaction, None)
        }
        PaymentAction::Sign(sign) => (sign.typed_data, TransactionType::Transfer, sign.recipient, TransferDataOutputType::Signature, None),
        PaymentAction::ApproveAndSign { approval, sign } => (sign.typed_data, TransactionType::Transfer, sign.recipient, TransferDataOutputType::Signature, Some(approval)),
    };

    PaymentTransaction {
        invoice,
        account: quote.account.clone(),
        transaction,
        transaction_type,
        memo: None,
        request: Some(PaymentRequest {
            address: recipient,
            amount: Some(PaymentAmount::AtomicValue { value: quote.value.clone() }),
            memo: None,
            label: None,
            references: None,
            asset_id: Some(quote.asset_id.clone()),
        }),
        output_type,
        approval,
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
    let asset_id = get_asset_id(&option.amount.unit)?;
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
    is_payment_host(&Url::parse(url).ok()?).then(|| url.to_string())
}

fn get_asset_id(unit: &str) -> Option<AssetId> {
    let asset_id = match unit.split_once('/')? {
        (CAIP19_PREFIX, asset) => WalletConnectCAIP19::get_asset_id(asset)?,
        _ => return None,
    };
    match asset_id.token_id {
        Some(token_id) => Some(AssetId::from_token(asset_id.chain, &ethereum_address_checksum(&token_id).ok()?)),
        None => Some(asset_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::model::{Merchant, PaymentAmount as PaymentOptionAmount, PaymentCollectData, PaymentInfo, PaymentPriceDisplay, PaymentSend, PaymentSign};
    use primitives::swap::ApprovalData;
    use primitives::{Chain, ChainAddress};

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
    fn test_map_transaction() {
        let quote = |asset_id: AssetId| Quote {
            id: "opt_1".to_string(),
            account: ChainAddress::new(Chain::Polygon, "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string()),
            asset_id,
            value: BigUint::from(1_000_000u32),
            collect_data_url: None,
            actions: Vec::new(),
        };
        let usdt = AssetId::from_token(Chain::Polygon, "0xc2132d05d31c914a87c6611c10748aeb04b58e8f");
        let sign = || PaymentSign {
            recipient: "0x0000000000a84d1a9b0063a910315c7ffa9cd248".to_string(),
            typed_data: "{\"primaryType\":\"PermitTransferFrom\"}".to_string(),
        };

        let coin = map_transaction(
            &quote(AssetId::from_chain(Chain::Polygon)),
            PaymentAction::Send(PaymentSend {
                recipient: "0x0000000000a84d1a9b0063a910315c7ffa9cd248".to_string(),
                value: BigUint::from(1_000_000u32),
                data: String::new(),
            }),
            PaymentInvoice::mock(),
        );
        assert_eq!(coin.transaction_type, TransactionType::Transfer);
        assert_eq!(coin.output_type, TransferDataOutputType::EncodedTransaction);
        assert_eq!(coin.approval, None);
        assert_eq!(coin.request.as_ref().and_then(|request| request.asset_id.clone()), Some(AssetId::from_chain(Chain::Polygon)));

        let signature = map_transaction(&quote(usdt.clone()), PaymentAction::Sign(sign()), PaymentInvoice::mock());
        assert_eq!(signature.transaction, sign().typed_data);
        assert_eq!(signature.transaction_type, TransactionType::Transfer);
        assert_eq!(signature.output_type, TransferDataOutputType::Signature);
        assert_eq!(signature.approval, None);
        assert_eq!(signature.request.as_ref().map(|request| request.address.clone()), Some(sign().recipient));
        assert_eq!(signature.request.as_ref().and_then(|request| request.asset_id.clone()), Some(usdt.clone()));

        let approval = ApprovalData::mock();
        let approved = map_transaction(
            &quote(usdt),
            PaymentAction::ApproveAndSign {
                approval: approval.clone(),
                sign: sign(),
            },
            PaymentInvoice::mock(),
        );
        assert_eq!(approved.output_type, TransferDataOutputType::Signature);
        assert_eq!(approved.approval, Some(approval));
    }

    #[test]
    fn test_map_options() {
        let account = "eip155:56:0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB".to_string();
        let option = |id: &str, account: &str, unit: &str| PaymentOption {
            id: id.to_string(),
            account: account.to_string(),
            amount: PaymentOptionAmount {
                unit: unit.to_string(),
                value: "1000".to_string(),
            },
            actions: Vec::new(),
            collect_data: Some(PaymentCollectData {
                url: format!("https://{}/collect/?pid=pay_1&option={id}", if id == "opt_phishing" { "phishing.example" } else { "pay.walletconnect.com" }),
            }),
        };
        let response = PaymentOptionsResponse {
            collect_data: Some(PaymentCollectData {
                url: "https://pay.walletconnect.com/collect/?pid=pay_1&accounts=eip155:56:0x92ab,eip155:1:0x92ab".to_string(),
            }),
            info: Some(PaymentInfo {
                status: PaymentStatus::RequiresAction,
                merchant: Merchant {
                    name: "Gem Coffee".to_string(),
                    icon_url: None,
                },
                amount: amount("iso4217/USD", "1999"),
            }),
            options: Some(vec![
                option("opt_bnb", &account, "caip19/eip155:56/slip44:60"),
                option("opt_cake", &account.to_lowercase(), "caip19/eip155:56/erc20:0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82"),
                option("opt_eth", "eip155:1:0x92abCE21234D71EC443E679f3a1feAFD3Fc830fB", "caip19/eip155:1/slip44:60"),
                option("opt_bad_token", &account, "caip19/eip155:56/erc20:0xnot-an-address"),
                option("opt_fiat", &account, "iso4217/USD"),
                option("opt_phishing", &account, "caip19/eip155:56/slip44:60"),
            ]),
        };

        let Options::Invoice(invoice) = map_options(response, &[account]).unwrap() else {
            panic!("expected an invoice");
        };
        assert_eq!(
            invoice.quotes.iter().map(|quote| (quote.id.as_str(), quote.asset_id.clone())).collect::<Vec<_>>(),
            vec![
                ("opt_bnb", AssetId::from_chain(Chain::SmartChain)),
                ("opt_cake", AssetId::from_token(Chain::SmartChain, "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82")),
            ],
            "another account, a bad token address, a fiat unit and a form outside pay.walletconnect.com are all left out"
        );
        assert_eq!(invoice.price.amount, 19.99);
        assert_eq!(invoice.quotes[0].collect_data_url.as_deref(), Some("https://pay.walletconnect.com/collect/?pid=pay_1&option=opt_bnb"));
        assert_eq!(
            invoice.collect_data_url.as_deref(),
            Some("https://pay.walletconnect.com/collect/?pid=pay_1&accounts=eip155:56:0x92ab,eip155:1:0x92ab"),
            "the form for every account comes with the response, not with one option"
        );
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
                collect_data_url: None,
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
