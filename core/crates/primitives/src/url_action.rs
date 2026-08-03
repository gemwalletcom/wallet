use crate::{Deeplink, Payment, PaymentLink, PaymentURLDecoder, WalletConnectLink};

#[derive(Debug, Clone, PartialEq)]
pub enum UrlAction {
    Deeplink { deeplink: Deeplink },
    Payment { link: PaymentLink },
    WalletConnect { link: WalletConnectLink },
}

impl UrlAction {
    pub fn from_url(url: &str) -> Option<Self> {
        if let Ok(Payment::Link(link)) = PaymentURLDecoder::decode(url) {
            return Some(Self::Payment { link });
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
    use crate::{AssetId, Chain, PaymentProviderName};

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
            UrlAction::from_url("https://pay.walletconnect.com/?pid=pay_123"),
            Some(UrlAction::Payment {
                link: PaymentLink::new(PaymentProviderName::WalletConnectPay, "pay_123".to_string()),
            })
        );
        assert_eq!(
            UrlAction::from_url("wc:abc@2?pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_123"),
            Some(UrlAction::Payment {
                link: PaymentLink::new(PaymentProviderName::WalletConnectPay, "pay_123".to_string()),
            })
        );
        assert_eq!(UrlAction::from_url("https://example.com/tokens/bitcoin"), None);
        assert_eq!(UrlAction::from_url("not a url"), None);
    }
}
