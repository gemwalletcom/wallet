use gem_evm::address::ethereum_address_checksum;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, PaymentAmount, PaymentInvoice, PaymentLink, PaymentMerchant, PaymentPrice, PaymentQuote, PaymentRequest, PaymentStatus, TransactionType,
    TransferDataOutputType, WalletConnectCAIP2, WalletConnectCAIP19, payment_decoder::is_payment_host,
};
use url::Url;

use crate::PaymentTransaction;
use crate::error::PaymentError;
use crate::wallet_connect_pay::model::{Invoice, Options, PaymentAction, PaymentOption, PaymentOptionsResponse, PaymentPriceAmount, Quote};

const CAIP19_PREFIX: &str = "caip19/";
const ISO4217_PREFIX: &str = "iso4217/";

pub(super) fn map_options(response: PaymentOptionsResponse, accounts: &[String]) -> Result<Options, PaymentError> {
    let payment = response.info;
    if payment.status != PaymentStatus::RequiresAction {
        return Ok(Options::Status { status: payment.status });
    }
    let quotes: Vec<Quote> = response.options.unwrap_or_default().into_iter().filter_map(|option| map_quote(option, accounts)).collect();
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
    Ok(PaymentPrice {
        currency: amount.unit.strip_prefix(ISO4217_PREFIX).unwrap_or(&amount.unit).to_string(),
        amount: BigNumberFormatter::value_as_f64(&amount.value.to_string(), amount.display.decimals)
            .map_err(|_| PaymentError::invalid_request(format!("Invalid payment amount {}", amount.value)))?,
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
        PaymentAction::ApproveAndSign { approval, sign } => {
            (sign.typed_data, TransactionType::Transfer, sign.recipient, TransferDataOutputType::Signature, Some(approval))
        }
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

fn map_quote(option: PaymentOption, accounts: &[String]) -> Option<Quote> {
    let account = accounts.iter().find(|account| account.eq_ignore_ascii_case(&option.account))?;
    let collect_data_url = match &option.collect_data {
        Some(collect_data) => Some(get_collect_data_url(&collect_data.url)?),
        None => None,
    };
    Some(Quote {
        id: option.id,
        account: WalletConnectCAIP2::parse_account(account.clone())?,
        asset_id: get_asset_id(&option.amount.unit)?,
        value: option.amount.value,
        collect_data_url,
        actions: option.actions,
    })
}

fn get_collect_data_url(url: &str) -> Option<String> {
    is_payment_host(&Url::parse(url).ok()?).then(|| url.to_string())
}

fn get_asset_id(unit: &str) -> Option<AssetId> {
    let asset_id = WalletConnectCAIP19::get_asset_id(unit.strip_prefix(CAIP19_PREFIX)?)?;
    match asset_id.token_id {
        Some(token_id) => Some(AssetId::from_token(asset_id.chain, &ethereum_address_checksum(&token_id).ok()?)),
        None => Some(asset_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect_pay::model::{Merchant, PaymentSend, TypedDataTransfer};
    use crate::wallet_connect_pay::testkit::{
        OPTIONS, OPTIONS_FAILED, OPTIONS_IDENTITY_REQUIRED, TEST_ACCOUNT, TEST_PERMIT_SPENDER, TEST_ROUTER, accounts, invoice, options, quote, quotes,
    };
    use primitives::asset_constants::{ETHEREUM_USDT_ASSET_ID, ETHEREUM_USDT_TOKEN_ID, SMARTCHAIN_USDC_TOKEN_ID, SMARTCHAIN_USDT_ASSET_ID};
    use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
    use primitives::swap::ApprovalData;
    use num_bigint::BigUint;
    use primitives::{Chain, ChainAddress};

    #[test]
    fn test_map_options() {
        let accounts = accounts(TEST_ACCOUNT);
        let response = options(OPTIONS);
        let quote = |index: usize, id: &str, asset_id: AssetId, value: u64| Quote {
            id: id.to_string(),
            account: ChainAddress::new(asset_id.chain, TEST_ACCOUNT.to_string()),
            asset_id,
            value: BigUint::from(value),
            collect_data_url: None,
            actions: response.options.as_ref().unwrap()[index].actions.clone(),
        };

        assert_eq!(
            map_options(options(OPTIONS), &accounts),
            Ok(Options::Invoice(Invoice {
                merchant: Merchant::mock(),
                price: PaymentPrice::mock(),
                quotes: vec![
                    quote(0, "e2e85aac-8275-4ae2-b264-fd513f7d4fb6", AssetId::from_chain(Chain::Optimism), 41_877_035_785_636),
                    quote(1, "c498802c-f496-4edf-ab43-aa391f42ab6e", ETHEREUM_USDT_ASSET_ID.clone(), 100_000),
                    quote(2, "bb125849-23a2-431d-bf0a-6e38405659de", AssetId::from_chain(Chain::Ethereum), 41_782_394_099_655),
                    quote(3, "99bb2908-daed-4dd4-b40c-3d70343b6a75", AssetId::from_chain(Chain::SmartChain), 140_151_888_385_232),
                    quote(4, "57965ba0-dc0f-4ca9-a553-9b8041ea669c", SMARTCHAIN_USDT_ASSET_ID.clone(), 100_000_000_000_000_000),
                    quote(5, "2e473787-74fe-4559-a3fc-6b62736497c2", AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDC_TOKEN_ID), 100_000_000_000_000_000),
                ],
                collect_data_url: None,
            }))
        );
        assert_eq!(invoice(options(OPTIONS), &accounts[1..2]).quotes.len(), 1, "only the options of the given accounts are quotes");
        assert_eq!(map_options(options(OPTIONS_FAILED), &accounts), Ok(Options::Status { status: PaymentStatus::Failed }));
        assert_eq!(map_options(options(OPTIONS), &[]), Err(PaymentError::NoPaymentOptions));

        let mut lowercase = options(OPTIONS);
        lowercase.options.as_mut().unwrap()[1].amount.unit = format!("caip19/eip155:1/erc20:{}", ETHEREUM_USDT_TOKEN_ID.to_lowercase());
        assert_eq!(invoice(lowercase, &accounts).quotes[1].asset_id, *ETHEREUM_USDT_ASSET_ID, "a lowercase token id is the wallet's checksummed one");
    }

    #[test]
    fn test_map_options_identity_required() {
        let accounts = accounts(TEST_ACCOUNT);
        let form = |accounts: &str| format!("https://pay.walletconnect.com/collect/?pid=pay_dfa2ecc101M2NV3FG4QSGGGZDQYW8DX45A&accounts={accounts}");
        let every_account = form(&["42161", "10", "137", "8453", "1", "56"].map(|chain| format!("eip155%3A{chain}%3A{TEST_ACCOUNT}")).join("%2C"));

        let verified = invoice(options(OPTIONS_IDENTITY_REQUIRED), &accounts);
        assert_eq!(verified.collect_data_url, Some(every_account), "the form for every account comes with the response");
        assert_eq!(verified.quotes[0].collect_data_url, Some(form(&format!("eip155%3A10%3A{TEST_ACCOUNT}"))), "the one for one account with its option");

        let mut phishing = options(OPTIONS_IDENTITY_REQUIRED);
        phishing.collect_data.as_mut().unwrap().url = "https://pay.walletconnect.com.example/collect".to_string();
        phishing.options.as_mut().unwrap()[0].collect_data.as_mut().unwrap().url = "http://pay.walletconnect.com/collect".to_string();
        let refused = invoice(phishing, &accounts);
        assert_eq!(refused.collect_data_url, None, "a form outside the payment host is not opened");
        assert_eq!(refused.quotes[0].asset_id, *ETHEREUM_USDT_ASSET_ID, "and an option asking for one is not offered");
    }

    #[test]
    fn test_map_transaction() {
        let invoice = PaymentInvoice::mock();
        let coin = quote(OPTIONS, TEST_ACCOUNT, &AssetId::from_chain(Chain::Optimism));
        let request = |quote: &Quote, address: &str| {
            Some(PaymentRequest {
                address: address.to_string(),
                amount: Some(PaymentAmount::AtomicValue { value: quote.value.clone() }),
                asset_id: Some(quote.asset_id.clone()),
                ..PaymentRequest::mock()
            })
        };
        let send = |data: &str| {
            PaymentAction::Send(PaymentSend {
                recipient: TEST_ROUTER.to_string(),
                data: data.to_string(),
            })
        };

        assert_eq!(
            map_transaction(&coin, send("0xd3906488"), invoice.clone()),
            PaymentTransaction {
                invoice: invoice.clone(),
                account: coin.account.clone(),
                transaction: "0xd3906488".to_string(),
                transaction_type: TransactionType::SmartContractCall,
                memo: None,
                request: request(&coin, TEST_ROUTER),
                output_type: TransferDataOutputType::EncodedTransaction,
                approval: None,
            }
        );
        assert_eq!(map_transaction(&coin, send(""), invoice.clone()).transaction_type, TransactionType::Transfer, "a value transfer has no calldata");

        let usdt = quote(OPTIONS, TEST_ACCOUNT, &ETHEREUM_USDT_ASSET_ID);
        let sign = TypedDataTransfer {
            token: ETHEREUM_USDT_TOKEN_ID.to_string(),
            amount: usdt.value.clone(),
            from: None,
            recipient: TEST_PERMIT_SPENDER.to_string(),
            verifying_contract: UNISWAP_PERMIT2_CONTRACT.to_string(),
            typed_data: "{}".to_string(),
        };
        let signature = PaymentTransaction {
            invoice: invoice.clone(),
            account: usdt.account.clone(),
            transaction: "{}".to_string(),
            transaction_type: TransactionType::Transfer,
            memo: None,
            request: request(&usdt, TEST_PERMIT_SPENDER),
            output_type: TransferDataOutputType::Signature,
            approval: None,
        };
        assert_eq!(map_transaction(&usdt, PaymentAction::Sign(sign.clone()), invoice.clone()), signature);
        assert_eq!(
            map_transaction(
                &usdt,
                PaymentAction::ApproveAndSign {
                    approval: ApprovalData::mock(),
                    sign,
                },
                invoice,
            ),
            PaymentTransaction {
                approval: Some(ApprovalData::mock()),
                ..signature
            }
        );
    }

    #[test]
    fn test_map_invoice() {
        let quotes = quotes(OPTIONS, TEST_ACCOUNT);
        let invoice = Invoice {
            merchant: Merchant::mock(),
            price: PaymentPrice::mock(),
            quotes: quotes[..1].to_vec(),
            collect_data_url: None,
        };

        assert_eq!(
            map_invoice(&invoice, "pay_1"),
            PaymentInvoice {
                link: PaymentLink::WalletConnectPay { payment_id: "pay_1".to_string() },
                merchant: PaymentMerchant::mock(),
                price: Some(PaymentPrice::mock()),
                quotes: vec![PaymentQuote {
                    id: quotes[0].id.clone(),
                    asset_id: AssetId::from_chain(Chain::Optimism),
                    value: BigUint::from(41_877_035_785_636u64),
                }],
                verification: None,
            }
        );
    }
}
