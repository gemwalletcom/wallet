use primitives::OptionStringExt;
use url::Url;

use crate::url_query::query_value;
use crate::{GEM_HOST, GEM_URL_SCHEME, HTTPS_URL_SCHEME, WALLET_CONNECT_URL_SCHEME};

const WALLET_CONNECT_HOST: &str = "wc";

const QUERY_URI: &str = "uri";
const QUERY_SESSION_TOPIC: &str = "sessionTopic";
const QUERY_REQUEST_ID: &str = "requestId";

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectLink {
    Connect { uri: String },
    Request,
    Session { topic: String },
}

impl WalletConnectLink {
    pub fn from_url(url: &str) -> Option<Self> {
        let parsed = Url::parse(url).ok()?;
        match parsed.scheme() {
            WALLET_CONNECT_URL_SCHEME => Some(Self::session_or_request(&parsed).unwrap_or_else(|| WalletConnectLink::Connect { uri: url.to_string() })),
            GEM_URL_SCHEME if parsed.host_str() == Some(WALLET_CONNECT_HOST) => Some(Self::wallet_link(&parsed)),
            HTTPS_URL_SCHEME if parsed.host_str() == Some(GEM_HOST) && parsed.path().strip_prefix('/').map(|path| path.trim_end_matches('/')) == Some(WALLET_CONNECT_HOST) => Some(Self::wallet_link(&parsed)),
            _ => None,
        }
    }

    pub fn app_link() -> String {
        format!("{GEM_URL_SCHEME}://{WALLET_CONNECT_HOST}")
    }

    pub fn universal_link() -> String {
        format!("{HTTPS_URL_SCHEME}://{GEM_HOST}/{WALLET_CONNECT_HOST}")
    }

    fn wallet_link(url: &Url) -> Self {
        Self::connect(url).or_else(|| Self::session_or_request(url)).unwrap_or(WalletConnectLink::Request)
    }

    fn connect(url: &Url) -> Option<Self> {
        query_value(url, QUERY_URI).non_empty().map(|uri| WalletConnectLink::Connect { uri })
    }

    fn session_or_request(url: &Url) -> Option<Self> {
        if let Some(topic) = query_value(url, QUERY_SESSION_TOPIC).non_empty() {
            Some(WalletConnectLink::Session { topic })
        } else if query_value(url, QUERY_REQUEST_ID).is_some() {
            Some(WalletConnectLink::Request)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_connect_link_from_url() {
        assert_eq!(
            WalletConnectLink::from_url("wc:abc@2?relay-protocol=irn&symKey=123"),
            Some(WalletConnectLink::Connect {
                uri: "wc:abc@2?relay-protocol=irn&symKey=123".to_string(),
            })
        );
        assert_eq!(WalletConnectLink::from_url("wc:abc@2?requestId"), Some(WalletConnectLink::Request));
        assert_eq!(WalletConnectLink::from_url("wc:abc@2?requestId=123"), Some(WalletConnectLink::Request));
        assert_eq!(WalletConnectLink::from_url("gem://wc?uri=wc:topic@2"), Some(WalletConnectLink::Connect { uri: "wc:topic@2".to_string() }));
        assert_eq!(
            WalletConnectLink::from_url("gem://wc?uri=wc%3Atopic%402%3Frelay-protocol%3Dirn%26symKey%3Dabc"),
            Some(WalletConnectLink::Connect {
                uri: "wc:topic@2?relay-protocol=irn&symKey=abc".to_string(),
            })
        );
        assert_eq!(WalletConnectLink::from_url("gem://wc?requestId=1"), Some(WalletConnectLink::Request));
        assert_eq!(WalletConnectLink::from_url("gem://wc?sessionTopic=abc123"), Some(WalletConnectLink::Session { topic: "abc123".to_string() }));
        assert_eq!(WalletConnectLink::from_url("gem://wc?sessionTopic="), Some(WalletConnectLink::Request));
        assert_eq!((WalletConnectLink::app_link().as_str(), WalletConnectLink::universal_link().as_str()), ("gem://wc", "https://gemwallet.com/wc"));
        for url in ["gem://wc", "https://gemwallet.com/wc", "https://gemwallet.com/wc/"] {
            assert_eq!(WalletConnectLink::from_url(url), Some(WalletConnectLink::Request), "a dapp opens the wallet to answer its request: {url}");
        }
        assert_eq!(WalletConnectLink::from_url("gem://asset/solana"), None);
        for url in [
            "https://gemwallet.com/wc?uri=wc%3Atopic%402%3Frelay-protocol%3Dirn%26symKey%3Dabc",
            "https://gemwallet.com/wc/?uri=wc%3Atopic%402%3Frelay-protocol%3Dirn%26symKey%3Dabc",
        ] {
            assert_eq!(
                WalletConnectLink::from_url(url),
                Some(WalletConnectLink::Connect {
                    uri: "wc:topic@2?relay-protocol=irn&symKey=abc".to_string(),
                })
            );
        }
        assert_eq!(WalletConnectLink::from_url("https://gemwallet.com/wc/?requestId=1"), Some(WalletConnectLink::Request));
        assert_eq!(WalletConnectLink::from_url("https://gemwallet.com/wc?sessionTopic=abc123"), Some(WalletConnectLink::Session { topic: "abc123".to_string() }));
        for url in [
            "http://gemwallet.com/wc?uri=wc:topic@2",
            "https://gemwallet.com.evil.example/wc?uri=wc:topic@2",
            "https://gemwallet.com/wc/other?uri=wc:topic@2",
        ] {
            assert_eq!(WalletConnectLink::from_url(url), None, "{url}");
        }
        assert_eq!(WalletConnectLink::from_url("https://gemwallet.com/tokens/bitcoin"), None);
    }
}
