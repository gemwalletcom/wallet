use primitives::{AssetId, Deeplink, UrlAction};

#[uniffi::remote(Enum)]
pub enum Deeplink {
    Asset { asset_id: AssetId },
    Perpetuals,
    Rewards { code: Option<String> },
    Receive { asset_id: AssetId },
    Buy { asset_id: AssetId, amount: Option<i32> },
    Sell { asset_id: AssetId, amount: Option<i32> },
    Swap { asset_id: AssetId },
}

#[derive(Default, uniffi::Object)]
pub struct GemDeeplinkService {}

#[uniffi::export]
impl GemDeeplinkService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_url(&self, deeplink: Deeplink) -> String {
        deeplink.to_url()
    }

    pub fn url_action(&self, url: String) -> Option<UrlAction> {
        UrlAction::from_url(&url)
    }
}

impl GemDeeplinkService {
    pub fn build_gem_url(&self, deeplink: Deeplink) -> String {
        deeplink.to_gem_url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_deeplink() {
        let service = GemDeeplinkService::new();
        let rewards = Deeplink::Rewards {
            code: Some("gemcoder".to_string()),
        };
        assert_eq!(service.build_url(rewards), "https://gemwallet.com/rewards?code=gemcoder");
        assert_eq!(service.build_gem_url(Deeplink::Perpetuals), "gem://perpetuals");
        assert_eq!(
            service.build_gem_url(Deeplink::Buy {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                amount: Some(100),
            }),
            "gem://tokens/bitcoin/buy?amount=100"
        );
    }
}
