use crate::{Deeplink, Payment, PaymentURLDecoder, WalletConnectLink};

#[derive(Debug, Clone, PartialEq)]
pub enum UrlAction {
    Deeplink { deeplink: Deeplink },
    Payment { payment: Payment },
    WalletConnect { link: WalletConnectLink },
}

impl UrlAction {
    pub fn from_url(url: &str) -> Option<Self> {
        if let Some(link) = WalletConnectLink::from_url(url) {
            return Some(Self::WalletConnect { link });
        }
        if let Some(deeplink) = Deeplink::from_url(url) {
            return Some(Self::Deeplink { deeplink });
        }
        PaymentURLDecoder::decode(url).ok().map(|payment| Self::Payment { payment })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetId, Chain, PaymentAmount, PaymentRequest};

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
    }
}
