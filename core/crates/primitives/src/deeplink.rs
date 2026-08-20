use url::Url;

use crate::url_query::query_value;
use crate::{AssetId, GEM_URL_SCHEME, HTTPS_URL_SCHEME};

const DEEPLINK_HOST: &str = "gemwallet.com";

const PATH_TOKENS: &str = "tokens";
const PATH_PERPETUALS: &str = "perpetuals";
const PATH_REWARDS: &str = "rewards";
const PATH_JOIN: &str = "join";
const PATH_RECEIVE: &str = "receive";
const PATH_BUY: &str = "buy";
const PATH_SELL: &str = "sell";

const QUERY_CODE: &str = "code";
const QUERY_AMOUNT: &str = "amount";

#[derive(Debug, Clone, PartialEq)]
pub enum Deeplink {
    Asset { asset_id: AssetId },
    Perpetuals,
    Rewards { code: Option<String> },
    Receive { asset_id: Option<AssetId> },
    Buy { asset_id: AssetId, amount: Option<i32> },
    Sell { asset_id: AssetId, amount: Option<i32> },
}

impl Deeplink {
    pub fn to_url(&self) -> String {
        format!("{HTTPS_URL_SCHEME}://{DEEPLINK_HOST}{}", self.path())
    }

    pub fn to_gem_url(&self) -> String {
        format!("{GEM_URL_SCHEME}://{}", self.path().trim_start_matches('/'))
    }

    pub fn from_url(url: &str) -> Option<Self> {
        let url = Url::parse(url).ok()?;
        let segments = url_segments(&url)?;
        Self::from_segments(&url, &segments).or_else(|| segments.get(1..).and_then(|segments| Self::from_segments(&url, segments)))
    }

    fn from_segments(url: &Url, segments: &[String]) -> Option<Self> {
        let (component, params) = segments.split_first()?;

        let deeplink = match component.as_str() {
            PATH_TOKENS => Deeplink::Asset {
                asset_id: asset_id_from_segments(params)?,
            },
            PATH_PERPETUALS => Deeplink::Perpetuals,
            PATH_REWARDS | PATH_JOIN => Deeplink::Rewards {
                code: params.first().cloned().or_else(|| query_value(url, QUERY_CODE)),
            },
            PATH_RECEIVE if params.is_empty() => Deeplink::Receive { asset_id: None },
            PATH_RECEIVE => Deeplink::Receive {
                asset_id: Some(asset_id_from_segments(params)?),
            },
            PATH_BUY => Deeplink::Buy {
                asset_id: asset_id_from_segments(params)?,
                amount: amount_from_query(url),
            },
            PATH_SELL => Deeplink::Sell {
                asset_id: asset_id_from_segments(params)?,
                amount: amount_from_query(url),
            },
            _ => return None,
        };
        Some(deeplink)
    }

    fn path(&self) -> String {
        match self {
            Deeplink::Asset { asset_id } => format!("/{PATH_TOKENS}{}", asset_path(asset_id)),
            Deeplink::Perpetuals => format!("/{PATH_PERPETUALS}"),
            Deeplink::Rewards { code } => path_with_query(PATH_REWARDS, QUERY_CODE, code.clone()),
            Deeplink::Receive { asset_id } => format!("/{PATH_RECEIVE}{}", asset_id.as_ref().map(asset_path).unwrap_or_default()),
            Deeplink::Buy { asset_id, amount } => fiat_path(PATH_BUY, asset_id, *amount),
            Deeplink::Sell { asset_id, amount } => fiat_path(PATH_SELL, asset_id, *amount),
        }
    }
}

fn asset_id_from_segments(segments: &[String]) -> Option<AssetId> {
    Some(AssetId::from(segments.first()?.parse().ok()?, segments.get(1).cloned()))
}

fn amount_from_query(url: &Url) -> Option<i32> {
    query_value(url, QUERY_AMOUNT)?.parse().ok().filter(|amount: &i32| *amount > 0)
}

fn asset_path(asset_id: &AssetId) -> String {
    match &asset_id.token_id {
        Some(token_id) => format!("/{}/{token_id}", asset_id.chain.as_ref()),
        None => format!("/{}", asset_id.chain.as_ref()),
    }
}

fn fiat_path(component: &str, asset_id: &AssetId, amount: Option<i32>) -> String {
    let path = format!("/{component}{}", asset_path(asset_id));
    match amount {
        Some(amount) => format!("{path}?{QUERY_AMOUNT}={amount}"),
        None => path,
    }
}

fn path_with_query(component: &str, query_key: &str, query_value: Option<String>) -> String {
    match query_value {
        Some(value) => format!("/{component}?{query_key}={value}"),
        None => format!("/{component}"),
    }
}

fn url_segments(url: &Url) -> Option<Vec<String>> {
    let mut segments: Vec<String> = url
        .path_segments()
        .map(|parts| parts.filter(|part| !part.is_empty()).map(String::from).collect())
        .unwrap_or_default();

    match url.scheme() {
        HTTPS_URL_SCHEME => {
            if url.host_str()? != DEEPLINK_HOST {
                return None;
            }
        }
        GEM_URL_SCHEME => {
            if let Some(host) = url.host_str().filter(|host| !host.is_empty()) {
                segments.insert(0, host.to_string());
            }
        }
        _ => return None,
    }
    Some(segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chain;

    #[test]
    fn test_to_url() {
        assert_eq!(
            Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            }
            .to_url(),
            "https://gemwallet.com/tokens/bitcoin"
        );
        assert_eq!(
            Deeplink::Asset {
                asset_id: AssetId::token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            }
            .to_url(),
            "https://gemwallet.com/tokens/ethereum/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
        assert_eq!(Deeplink::Perpetuals.to_url(), "https://gemwallet.com/perpetuals");
        assert_eq!(Deeplink::Rewards { code: None }.to_url(), "https://gemwallet.com/rewards");
        assert_eq!(
            Deeplink::Rewards {
                code: Some("gemcoder".to_string()),
            }
            .to_url(),
            "https://gemwallet.com/rewards?code=gemcoder"
        );
        assert_eq!(Deeplink::Receive { asset_id: None }.to_url(), "https://gemwallet.com/receive");
        assert_eq!(
            Deeplink::Receive {
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            }
            .to_url(),
            "https://gemwallet.com/receive/bitcoin"
        );
        assert_eq!(
            Deeplink::Buy {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                amount: Some(100),
            }
            .to_url(),
            "https://gemwallet.com/buy/bitcoin?amount=100"
        );
        assert_eq!(
            Deeplink::Sell {
                asset_id: AssetId::from_chain(Chain::Solana),
                amount: None,
            }
            .to_url(),
            "https://gemwallet.com/sell/solana"
        );
    }

    #[test]
    fn test_to_gem_url() {
        assert_eq!(Deeplink::Rewards { code: None }.to_gem_url(), "gem://rewards");
        assert_eq!(Deeplink::Perpetuals.to_gem_url(), "gem://perpetuals");
        assert_eq!(
            Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            }
            .to_gem_url(),
            "gem://tokens/bitcoin"
        );
    }

    #[test]
    fn test_from_url() {
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/tokens/bitcoin"),
            Some(Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/tokens/ethereum/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            Some(Deeplink::Asset {
                asset_id: AssetId::token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/zh-cn/tokens/optimism/0x58538e6A46E07434d7E7375Bc268D3cb839C0133/"),
            Some(Deeplink::Asset {
                asset_id: AssetId::token(Chain::Optimism, "0x58538e6A46E07434d7E7375Bc268D3cb839C0133"),
            })
        );
        assert_eq!(
            Deeplink::from_url("gem://tokens/bitcoin"),
            Some(Deeplink::Asset {
                asset_id: AssetId::from_chain(Chain::Bitcoin)
            })
        );
        assert_eq!(
            Deeplink::from_url("gem://zh-cn/tokens/optimism/0x58538e6A46E07434d7E7375Bc268D3cb839C0133"),
            Some(Deeplink::Asset {
                asset_id: AssetId::token(Chain::Optimism, "0x58538e6A46E07434d7E7375Bc268D3cb839C0133"),
            })
        );
        assert_eq!(Deeplink::from_url("https://gemwallet.com/perpetuals"), Some(Deeplink::Perpetuals));
        assert_eq!(Deeplink::from_url("gem://perpetuals"), Some(Deeplink::Perpetuals));
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/rewards?code=gemcoder"),
            Some(Deeplink::Rewards {
                code: Some("gemcoder".to_string()),
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/join/gemcoder"),
            Some(Deeplink::Rewards {
                code: Some("gemcoder".to_string()),
            })
        );
        assert_eq!(
            Deeplink::from_url("https://gemwallet.com/en/join?code=test"),
            Some(Deeplink::Rewards { code: Some("test".to_string()) })
        );
        assert_eq!(Deeplink::from_url("https://gemwallet.com/join"), Some(Deeplink::Rewards { code: None }));
        assert_eq!(Deeplink::from_url("https://gemwallet.com/tokens"), None);
        assert_eq!(Deeplink::from_url("https://gemwallet.com/tokens/notachain"), None);
        assert_eq!(Deeplink::from_url("https://example.com/tokens/bitcoin"), None);
        assert_eq!(Deeplink::from_url("https://gemwallet.com/unknown"), None);
        assert_eq!(Deeplink::from_url("not a url"), None);
    }
    #[test]
    fn test_from_url_receive_buy_sell() {
        assert_eq!(Deeplink::from_url("gem://receive"), Some(Deeplink::Receive { asset_id: None }));
        assert_eq!(
            Deeplink::from_url("gem://receive/bitcoin"),
            Some(Deeplink::Receive {
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
            })
        );
        assert_eq!(
            Deeplink::from_url("gem://buy/bitcoin?amount=100"),
            Some(Deeplink::Buy {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                amount: Some(100),
            })
        );
        assert_eq!(
            Deeplink::from_url("gem://sell/ethereum/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            Some(Deeplink::Sell {
                asset_id: AssetId::token(Chain::Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
                amount: None,
            })
        );
        assert_eq!(
            Deeplink::from_url("gem://buy/bitcoin?amount=49.5"),
            Some(Deeplink::Buy {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                amount: None,
            })
        );
        assert_eq!(Deeplink::from_url("gem://receive/notachain"), None);
        assert_eq!(Deeplink::from_url("gem://buy"), None);
    }
}
