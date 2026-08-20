use primitives::{AssetId, Deeplink};

#[uniffi::remote(Enum)]
pub enum Deeplink {
    Asset { asset_id: AssetId },
    Perpetuals,
    Rewards { code: Option<String> },
    Receive { asset_id: Option<AssetId> },
    Buy { asset_id: AssetId, amount: Option<i32> },
    Sell { asset_id: AssetId, amount: Option<i32> },
}

#[uniffi::export]
pub fn deeplink_build_url(deeplink: Deeplink) -> String {
    deeplink.to_url()
}

#[uniffi::export]
pub fn deeplink_build_gem_url(deeplink: Deeplink) -> String {
    deeplink.to_gem_url()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_deeplink() {
        let rewards = Deeplink::Rewards {
            code: Some("gemcoder".to_string()),
        };
        assert_eq!(deeplink_build_url(rewards), "https://gemwallet.com/rewards?code=gemcoder");
        assert_eq!(deeplink_build_gem_url(Deeplink::Perpetuals), "gem://perpetuals");
        assert_eq!(deeplink_build_gem_url(Deeplink::Receive { asset_id: None }), "gem://receive");
        assert_eq!(
            deeplink_build_gem_url(Deeplink::Buy {
                asset_id: AssetId::from_chain(Chain::Bitcoin),
                amount: Some(100),
            }),
            "gem://buy/bitcoin?amount=100"
        );
    }
}
