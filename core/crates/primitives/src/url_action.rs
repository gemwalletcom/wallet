use crate::{Deeplink, Payment, PaymentURLDecoder, WalletConnectLink};

#[derive(Debug, Clone, PartialEq)]
pub enum UrlAction {
    Deeplink { deeplink: Deeplink },
    Payment { payment: Payment },
    WalletConnect { link: WalletConnectLink },
}

impl UrlAction {
    pub fn from_url(url: &str) -> Option<Self> {
        if let Ok(payment) = PaymentURLDecoder::decode(url) {
            return Some(Self::Payment { payment });
        }
        if let Some(link) = WalletConnectLink::from_url(url) {
            return Some(Self::WalletConnect { link });
        }
        Deeplink::from_url(url).map(|deeplink| Self::Deeplink { deeplink })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetId, Chain, PaymentAmount, PaymentLink, PaymentRequest};

    #[test]
    fn test_from_url() {
        assert_eq!(
            UrlAction::from_url("https://gemwallet.com/tokens/bitcoin"),
            Some(UrlAction::Deeplink {
                deeplink: Deeplink::Asset {
                    asset_id: AssetId::from_chain(Chain::Bitcoin),
                },
            })
        );
        assert_eq!(
            UrlAction::from_url("gem://wc?sessionTopic=abc123"),
            Some(UrlAction::WalletConnect {
                link: WalletConnectLink::Session { topic: "abc123".to_string() },
            })
        );
        assert_eq!(
            UrlAction::from_url("wc:topic@2?relay-protocol=irn&symKey=abc"),
            Some(UrlAction::WalletConnect {
                link: WalletConnectLink::Connect {
                    uri: "wc:topic@2?relay-protocol=irn&symKey=abc".to_string(),
                },
            })
        );
        assert_eq!(
            UrlAction::from_url("bitcoin:bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4?amount=0.1"),
            Some(UrlAction::Payment {
                payment: Payment::Request(PaymentRequest {
                    address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string(),
                    amount: Some(PaymentAmount::ExactValue("0.1".to_string())),
                    memo: None,
                    asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                }),
            })
        );
        assert_eq!(UrlAction::from_url("https://example.com/tokens/bitcoin"), None);
        assert_eq!(
            UrlAction::from_url("wc:abc@2?pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_123"),
            Some(UrlAction::Payment {
                payment: Payment::Link(PaymentLink::WalletConnectPay("pay_123".to_string())),
            })
        );
        assert_eq!(
            UrlAction::from_url("not a url"),
            Some(UrlAction::Payment {
                payment: Payment::Request(PaymentRequest {
                    address: "not a url".to_string(),
                    ..PaymentRequest::mock()
                }),
            })
        );
    }
}
