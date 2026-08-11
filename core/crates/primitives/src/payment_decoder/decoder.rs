use super::error::{PaymentDecoderError, Result};
use crate::{
    Chain,
    asset::ChainAsset,
    asset_id::AssetId,
    payment::{Payment, PaymentLink, PaymentRequest},
};
use std::{collections::HashMap, str::FromStr};
use url::form_urlencoded;

use super::{
    erc681::{ETHEREUM_SCHEME, TransactionRequest, integer_value},
    solana_pay::{self, PayTransfer as SolanaPayTransfer, SOLANA_PAY_SCHEME},
    ton_pay::{self, TON_PAY_SCHEME},
    wallet_connect_pay,
};

const REQUIRED_PARAMETER_PREFIX: &str = "req-";
const XRP_SCHEMES: [&str; 2] = ["ripple", "xrpl"];
const QUERY_DESTINATION_TAG: &str = "dt";

#[derive(Debug)]
pub struct PaymentURLDecoder;

impl PaymentURLDecoder {
    pub fn decode(string: &str) -> Result<Payment> {
        let uri = string.trim();

        if let Some(payment_id) = wallet_connect_pay::parse(uri) {
            return Ok(Payment::Link(PaymentLink::WalletConnectPay(payment_id)));
        }

        let Some((scheme, path)) = uri.split_once(':') else {
            return Self::decode_path(uri, None);
        };
        let scheme = scheme.to_ascii_lowercase();

        match scheme.as_str() {
            ETHEREUM_SCHEME => Ok(Payment::Request(TransactionRequest::parse(uri)?.into())),
            SOLANA_PAY_SCHEME => match solana_pay::parse(uri)? {
                solana_pay::RequestType::Transfer(transfer) => Ok(Payment::Request(transfer.into())),
                solana_pay::RequestType::Transaction(link) => Ok(Payment::Link(PaymentLink::SolanaPay(link))),
            },
            TON_PAY_SCHEME => {
                let ton_payment = ton_pay::parse(uri)?;
                Ok(Payment::Request(PaymentRequest {
                    address: ton_payment.recipient,
                    amount: ton_payment.amount.and_then(|amount| whole_coins(&amount, Chain::Ton)),
                    memo: ton_payment.comment,
                    asset_id: Some(ton_payment.asset_id),
                }))
            }
            _ => Self::decode_path(path, Self::decode_scheme(&scheme)),
        }
    }

    fn decode_path(path: &str, asset_id: Option<AssetId>) -> Result<Payment> {
        let Some((address, query)) = path.split_once('?') else {
            return Ok(Payment::Request(PaymentRequest {
                address: path.to_string(),
                amount: None,
                memo: None,
                asset_id,
            }));
        };

        let params = Self::decode_query_string(query);
        if let Some(required) = params.keys().find(|key| key.starts_with(REQUIRED_PARAMETER_PREFIX)) {
            return Err(PaymentDecoderError::InvalidFormat(format!("Unsupported required parameter: {required}")));
        }

        let is_xrp = asset_id.as_ref().is_some_and(|asset_id| asset_id.chain == Chain::Xrp);

        Ok(Payment::Request(PaymentRequest {
            address: address.to_string(),
            amount: params.get("amount").cloned(),
            memo: params.get("memo").or_else(|| params.get(QUERY_DESTINATION_TAG).filter(|_| is_xrp)).cloned(),
            asset_id,
        }))
    }

    fn decode_query_string(query_string: &str) -> HashMap<String, String> {
        form_urlencoded::parse(query_string.as_bytes())
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect()
    }

    fn decode_scheme(scheme: &str) -> Option<AssetId> {
        let chain = match scheme {
            scheme if XRP_SCHEMES.contains(&scheme) => Chain::Xrp,
            scheme => Chain::from_str(scheme).ok()?,
        };
        Some(AssetId::from(chain, None))
    }
}

fn whole_coins(smallest_unit: &str, chain: Chain) -> Option<String> {
    let decimals = usize::try_from(ChainAsset::from_chain(chain).asset.decimals).ok()?;
    Some(with_decimal_point(smallest_unit, decimals))
}

fn with_decimal_point(digits: &str, decimals: usize) -> String {
    if digits.len() > decimals {
        let (integer, fraction) = digits.split_at(digits.len() - decimals);
        decimal(integer, fraction)
    } else {
        decimal("0", &format!("{}{digits}", "0".repeat(decimals - digits.len())))
    }
}

fn decimal(integer: &str, fraction: &str) -> String {
    match fraction.trim_end_matches('0') {
        "" => integer.to_string(),
        fraction => format!("{integer}.{fraction}"),
    }
}

impl From<TransactionRequest> for PaymentRequest {
    fn from(val: TransactionRequest) -> Self {
        let address: String;
        let mut amount: Option<String>;
        let asset_id: Option<AssetId>;
        let memo = val.parameters.get("memo").map(|x| x.to_string());
        let mut chain = Chain::Ethereum;
        if let Some(chain_id) = val.chain_id {
            chain = Chain::from_chain_id(chain_id).unwrap_or(Chain::Ethereum);
        }

        // ERC20
        if val.function_name == Some("transfer".to_string()) {
            address = val.parameters.get("address").map(|x| x.to_string()).unwrap_or("".to_string());
            amount = None;
            asset_id = Some(AssetId::from(chain, Some(val.target_address)));
        } else {
            address = val.target_address;
            amount = val
                .parameters
                .get("value")
                .and_then(|value| integer_value(value))
                .and_then(|value| whole_coins(&value, chain));
            if amount.is_none() {
                amount = val.parameters.get("amount").cloned();
            }
            asset_id = Some(AssetId::from(chain, None));
        };
        Self { address, amount, memo, asset_id }
    }
}

impl From<SolanaPayTransfer> for PaymentRequest {
    fn from(val: SolanaPayTransfer) -> Self {
        Self {
            address: val.recipient,
            amount: val.amount,
            memo: val.memo,
            asset_id: Some(AssetId::from(Chain::Solana, val.spl_token.map(|x| x.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;

    #[test]
    fn test_address() {
        assert_eq!(
            PaymentURLDecoder::decode("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326").unwrap(),
            Payment::Request(PaymentRequest::new_address("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326"))
        );
    }

    #[test]
    fn test_uri_normalization() {
        let bitcoin = Payment::Request(PaymentRequest {
            address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
            amount: Some("0.1".to_string()),
            memo: None,
            asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
        });
        let uri = "bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.1";

        assert_eq!(PaymentURLDecoder::decode(uri).unwrap(), bitcoin);
        assert_eq!(PaymentURLDecoder::decode(&format!("  {uri}\n")).unwrap(), bitcoin);
        assert_eq!(PaymentURLDecoder::decode("BITCOIN:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.1").unwrap(), bitcoin);
        assert_eq!(
            PaymentURLDecoder::decode("bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?AMOUNT=0.1").unwrap(),
            Payment::Request(PaymentRequest {
                address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            })
        );

        assert_eq!(
            PaymentURLDecoder::decode("bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?memo=see%20http%3A%2F%2Fx.com").unwrap(),
            Payment::Request(PaymentRequest {
                address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
                amount: None,
                memo: Some("see http://x.com".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?memo=YWJjZA==").unwrap(),
            Payment::Request(PaymentRequest {
                address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
                amount: None,
                memo: Some("YWJjZA==".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            })
        );
        assert!(PaymentURLDecoder::decode("bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?req-escrow=1").is_err());
    }

    #[test]
    fn test_ton() {
        assert_eq!(
            PaymentURLDecoder::decode("ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA?amount=1000000000&text=order+7").unwrap(),
            Payment::Request(PaymentRequest {
                address: "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA".to_string(),
                amount: Some("1".to_string()),
                memo: Some("order 7".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Ton)),
            })
        );
    }

    #[test]
    fn test_xrp_destination_tag() {
        let payment = Payment::Request(PaymentRequest {
            address: "rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh".to_string(),
            amount: Some("10".to_string()),
            memo: Some("12345".to_string()),
            asset_id: Some(AssetId::from_chain(Chain::Xrp)),
        });

        assert_eq!(PaymentURLDecoder::decode("ripple:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?dt=12345&amount=10").unwrap(), payment);
        assert_eq!(PaymentURLDecoder::decode("xrpl:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?dt=12345&amount=10").unwrap(), payment);
        assert_eq!(PaymentURLDecoder::decode("xrp:rEb8TK3gBgk5auZkwc6sHnwrGVJH8DuaLh?dt=12345&amount=10").unwrap(), payment);

        assert_eq!(
            PaymentURLDecoder::decode("bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?dt=12345").unwrap(),
            Payment::Request(PaymentRequest {
                address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            })
        );
    }

    #[test]
    fn test_solana() {
        assert_eq!(
            PaymentURLDecoder::decode("HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5").unwrap(),
            Payment::Request(PaymentRequest::new_address("HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5"))
        );
        assert_eq!(
            PaymentURLDecoder::decode("solana:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5?amount=0.266232").unwrap(),
            Payment::Request(PaymentRequest {
                address: "HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5".to_string(),
                amount: Some("0.266232".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1df6564b6b4d43eaa077b732ad9b6ab9%3Fstate%3DAlabama%26country%3DUSA%26lineItems%3D%255B%257B%2522id%2522%253A%2522lineItem_82032b8ea67244e692cd322051e35689%2522%252C%2522quantity%2522%253A500%257D%255D%26solanaPayReference%3D4Vqsq8WhoTbFu8Lw2DbbtnCiHXXmBRN6afF8HkgxrXs7%26paymentReference%3DOZ_UxaOrU_F8fM5GhlrE2%26network%3Dsol%26skipPreflight%3Dfalse").unwrap(),
            Payment::Link(PaymentLink::SolanaPay("https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1df6564b6b4d43eaa077b732ad9b6ab9?state=Alabama&country=USA&lineItems=%5B%7B%22id%22%3A%22lineItem_82032b8ea67244e692cd322051e35689%22%2C%22quantity%22%3A500%7D%5D&solanaPayReference=4Vqsq8WhoTbFu8Lw2DbbtnCiHXXmBRN6afF8HkgxrXs7&paymentReference=OZ_UxaOrU_F8fM5GhlrE2&network=sol&skipPreflight=false".to_string()
            )),
        );
    }

    #[test]
    fn test_wallet_connect_pay() {
        assert_eq!(
            PaymentURLDecoder::decode("https://pay.walletconnect.com/?pid=pay_123").unwrap(),
            Payment::Link(PaymentLink::WalletConnectPay("pay_123".to_string()))
        );
        assert_eq!(
            PaymentURLDecoder::decode("wc:abc@2?pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_123").unwrap(),
            Payment::Link(PaymentLink::WalletConnectPay("pay_123".to_string()))
        );
        assert_eq!(
            PaymentURLDecoder::decode("wc:abc@2?pay=https://pay.walletconnect.com/?pid=pay_123").unwrap(),
            Payment::Link(PaymentLink::WalletConnectPay("pay_123".to_string()))
        );
    }

    #[test]
    fn test_bip21() {
        assert_eq!(
            PaymentURLDecoder::decode("bitcoin:bc1pn6pua8a566z7t822kphpd2el45ntm23354c3krfmpe3nnn33lkcskuxrdl?amount=0.00001").unwrap(),
            Payment::Request(PaymentRequest {
                address: "bc1pn6pua8a566z7t822kphpd2el45ntm23354c3krfmpe3nnn33lkcskuxrdl".to_string(),
                amount: Some("0.00001".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            })
        );

        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xA20d8935d61812b7b052E08f0768cFD6D81cB088?amount=0.01233&memo=test").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0xA20d8935d61812b7b052E08f0768cFD6D81cB088".to_string(),
                amount: Some("0.01233".to_string()),
                memo: Some("test".to_string()),
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
            })
        );

        assert_eq!(
            PaymentURLDecoder::decode("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            Payment::Request(PaymentRequest {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
            })
        );
    }

    #[test]
    fn test_erc681() {
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A@1").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A@0x38?amount=1.23").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: Some("1.23".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::SmartChain)),
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xcB3028d6120802148f03d6c884D6AD6A210Df62A?value=2.014e18").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: Some("2.014".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:my-wallet.eth").unwrap(),
            Payment::Request(PaymentRequest {
                address: "my-wallet.eth".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:pay-0xcB3028d6120802148f03d6c884D6AD6A210Df62A?value=1e6").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: Some("0.000000000001".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48@1/transfer?address=0xcB3028d6120802148f03d6c884D6AD6A210Df62A&uint256=1500000").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0xcB3028d6120802148f03d6c884D6AD6A210Df62A".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from(Chain::Ethereum, Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()))),
            })
        );
    }

    #[test]
    fn test_ton_address() {
        assert_eq!(
            PaymentURLDecoder::decode("UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA").unwrap(),
            Payment::Request(PaymentRequest {
                address: "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA".to_string(),
                amount: None,
                memo: None,
                asset_id: None,
            })
        );
        assert_eq!(
            PaymentURLDecoder::decode("ton://transfer/UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA").unwrap(),
            Payment::Request(PaymentRequest {
                address: "UQA5olhYULHkui4mTQM0LodWG0EqUaxmK6-e3mHrCZFO2diA".to_string(),
                amount: None,
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Ton)),
            })
        );
    }

    #[test]
    fn test_address_with_amount() {
        assert_eq!(
            PaymentURLDecoder::decode("0x25851Bf7D35293A89F710eBFbD4718322eF7B174?amount=50.72").unwrap(),
            Payment::Request(PaymentRequest {
                address: "0x25851Bf7D35293A89F710eBFbD4718322eF7B174".to_string(),
                amount: Some("50.72".to_string()),
                memo: None,
                asset_id: None,
            })
        );
    }
}
