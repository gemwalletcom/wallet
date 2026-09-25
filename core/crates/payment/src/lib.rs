mod decoder;
mod error;
mod model;
mod provider;
mod service;
mod solana_pay;
mod wallet_connect_pay;

use primitives::UrlAction;

pub use decoder::{PaymentDecoderError, PaymentURLDecoder};
pub use error::PaymentError;
pub use model::{PaymentLoad, PaymentTransaction, PaymentUpdate};
pub use service::PaymentService;
pub use wallet_connect_pay::{VerificationOutcome, WalletConnectPayAuth, verification_outcome};

pub fn classify_url(url: &str) -> Option<UrlAction> {
    PaymentURLDecoder::decode(url).ok().map(|payment| UrlAction::Payment { payment }).or_else(|| UrlAction::from_url(url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::payment::{Payment, PaymentLink};
    use primitives::{AssetId, Chain, Deeplink, WalletConnectLink};

    #[test]
    fn test_classify_url_reads_a_pay_link_before_a_wallet_connect_pairing() {
        let link = Payment::Link {
            link: PaymentLink::WalletConnectPay { payment_id: "pay_123".to_string() },
        };

        assert_eq!(classify_url("wc:abc@2?pay=https%3A%2F%2Fpay.walletconnect.com%2F%3Fpid%3Dpay_123"), Some(UrlAction::Payment { payment: link.clone() }));
        assert_eq!(classify_url("https://pay.walletconnect.com/?pid=pay_123"), Some(UrlAction::Payment { payment: link }));
        assert_eq!(
            classify_url("wc:topic@2?relay-protocol=irn&symKey=abc"),
            Some(UrlAction::WalletConnect {
                link: WalletConnectLink::Connect {
                    uri: "wc:topic@2?relay-protocol=irn&symKey=abc".to_string(),
                },
            })
        );
        assert_eq!(
            classify_url("https://gemwallet.com/tokens/bitcoin"),
            Some(UrlAction::Deeplink {
                deeplink: Deeplink::Asset {
                    asset_id: AssetId::from_chain(Chain::Bitcoin),
                },
            })
        );
    }
}
