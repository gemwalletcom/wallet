use primitives::{Deeplink, Payment, UrlAction, WalletConnectLink};

#[uniffi::remote(Enum)]
pub enum UrlAction {
    Deeplink { deeplink: Deeplink },
    Payment { payment: Payment },
    WalletConnect { link: WalletConnectLink },
}

#[uniffi::export]
pub fn url_action(url: &str) -> Option<UrlAction> {
    UrlAction::from_url(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_action() {
        assert!(matches!(url_action("https://gemwallet.com/tokens/bitcoin"), Some(UrlAction::Deeplink { .. })));
        assert!(matches!(url_action("gem://wc?sessionTopic=abc"), Some(UrlAction::WalletConnect { .. })));
        assert!(matches!(
            url_action("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1"),
            Some(UrlAction::Payment { .. })
        ));
        assert_eq!(url_action("https://example.com"), None);
        assert_eq!(url_action("https://pay.walletconnect.com/?pid=pay_123"), None);
    }
}
