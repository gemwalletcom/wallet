use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{AssetId, asset_score::AssetRank};
use security::{TokenScanProviders, TokenTarget};
use storage::{AssetFilter, AssetUpdate};
use streamer::consumer::MessageConsumer;

use crate::assets::repository::Repository;

pub struct FetchAssetStatusConsumer {
    pub(crate) repository: Arc<dyn Repository>,
    pub providers: TokenScanProviders,
}

#[derive(Debug, PartialEq, Eq)]
struct AssetStatusVerdict {
    is_malicious: bool,
    provider_count: usize,
    failed_providers: Vec<&'static str>,
}

impl AssetStatusVerdict {
    fn from_provider_results(results: &[(&'static str, Option<bool>)]) -> Self {
        Self {
            is_malicious: results.iter().any(|(_, result)| *result == Some(true)),
            provider_count: results.len(),
            failed_providers: results.iter().filter_map(|(provider, result)| result.is_none().then_some(*provider)).collect(),
        }
    }
}

#[async_trait]
impl MessageConsumer<AssetId, bool> for FetchAssetStatusConsumer {
    async fn should_consume(&self, asset_id: &AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(asset_id.is_token() && self.providers.iter().any(|provider| provider.supports_chain(asset_id.chain)))
    }

    async fn consume(&self, asset_id: AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let token_id = asset_id.get_token_id()?.clone();
        let target = TokenTarget { token_id, chain: asset_id.chain };
        let results = future::join_all(
            self.providers
                .iter()
                .filter(|provider| provider.supports_chain(target.chain))
                .map(|provider| async { (provider.name(), provider.scan_token(&target).await) }),
        )
        .await;
        let provider_results = results
            .into_iter()
            .map(|(provider, result)| match result {
                Ok(result) => {
                    info_with_fields!(
                        "asset status provider result",
                        provider = result.provider.as_str(),
                        chain = result.target.chain.as_ref(),
                        token_id = result.target.token_id.as_str(),
                        malicious = result.is_malicious,
                        reason = result.reason.as_deref().unwrap_or_default()
                    );
                    (provider, Some(result.is_malicious))
                }
                Err(error) => {
                    error_with_fields!("asset status fetch failed", error.as_ref(), provider = provider, chain = target.chain.as_ref(), token_id = target.token_id.as_str());
                    (provider, None)
                }
            })
            .collect::<Vec<_>>();
        let verdict = AssetStatusVerdict::from_provider_results(&provider_results);

        if verdict.is_malicious {
            self.repository
                .update_assets(vec![AssetFilter::Ids(vec![asset_id.to_string()])], vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)])
                .await?;
        }
        let failed_providers = verdict.failed_providers.join(",");
        info_with_fields!(
            "asset status result",
            chain = target.chain.as_ref(),
            token_id = target.token_id.as_str(),
            malicious = verdict.is_malicious,
            provider_count = verdict.provider_count,
            provider_failures = verdict.failed_providers.len(),
            failed_providers = failed_providers.as_str()
        );
        Ok(verdict.is_malicious)
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use primitives::{AssetBasic, Chain};
    use security::{ScanResult, TokenScanProvider};

    use super::*;
    use crate::testkit::MemoryAssetRepository;

    struct StaticTokenScan(Option<bool>);

    #[async_trait]
    impl TokenScanProvider for StaticTokenScan {
        fn name(&self) -> &'static str {
            "Static"
        }

        fn supports_chain(&self, _chain: Chain) -> bool {
            true
        }

        async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn Error + Send + Sync>> {
            let is_malicious = self.0.ok_or("scan unavailable")?;
            Ok(ScanResult {
                target: target.clone(),
                is_malicious,
                reason: None,
                provider: "Static".to_string(),
            })
        }
    }

    async fn consume(result: Option<bool>) -> (bool, Arc<MemoryAssetRepository>) {
        let repository = Arc::new(MemoryAssetRepository::new(Vec::<AssetBasic>::new()));
        let consumer = FetchAssetStatusConsumer {
            repository: repository.clone(),
            providers: vec![Arc::new(StaticTokenScan(result))],
        };
        let is_malicious = consumer.consume(AssetId::from_token(Chain::Ethereum, "0x1")).await.unwrap();
        (is_malicious, repository)
    }

    #[tokio::test]
    async fn test_malicious_token_is_disabled_as_fraudulent() {
        let (is_malicious, repository) = consume(Some(true)).await;

        assert!(is_malicious);
        let updates = repository.updates();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].0, vec![AssetFilter::Ids(vec![AssetId::from_token(Chain::Ethereum, "0x1").to_string()])]);
        assert_eq!(updates[0].1, vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)]);
    }

    #[tokio::test]
    async fn test_clean_or_failed_scan_leaves_token() {
        assert!(consume(Some(false)).await.1.updates().is_empty());
        assert!(consume(None).await.1.updates().is_empty());
    }

    #[test]
    fn test_from_provider_results() {
        assert_eq!(
            AssetStatusVerdict::from_provider_results(&[("GoPlus", Some(false)), ("Jupiter", Some(false))]),
            AssetStatusVerdict {
                is_malicious: false,
                provider_count: 2,
                failed_providers: vec![],
            }
        );
        assert_eq!(
            AssetStatusVerdict::from_provider_results(&[("GoPlus", Some(false)), ("Jupiter", None)]),
            AssetStatusVerdict {
                is_malicious: false,
                provider_count: 2,
                failed_providers: vec!["Jupiter"],
            }
        );
        assert_eq!(
            AssetStatusVerdict::from_provider_results(&[("GoPlus", Some(true)), ("Jupiter", None)]),
            AssetStatusVerdict {
                is_malicious: true,
                provider_count: 2,
                failed_providers: vec!["Jupiter"],
            }
        );
        assert_eq!(
            AssetStatusVerdict::from_provider_results(&[("GoPlus", None), ("Jupiter", None)]),
            AssetStatusVerdict {
                is_malicious: false,
                provider_count: 2,
                failed_providers: vec!["GoPlus", "Jupiter"],
            }
        );
    }
}
