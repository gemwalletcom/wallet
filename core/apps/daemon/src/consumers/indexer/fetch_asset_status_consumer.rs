use std::error::Error;

use async_trait::async_trait;
use futures::future;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{AssetId, asset_score::AssetRank};
use security_provider::{TokenScanProviders, TokenTarget};
use storage::{AssetUpdate, AssetsRepository, Database};
use streamer::consumer::MessageConsumer;

pub struct FetchAssetStatusConsumer {
    pub database: Database,
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
    async fn should_process(&self, asset_id: &AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(asset_id.is_token() && self.providers.iter().any(|provider| provider.supports_chain(asset_id.chain)))
    }

    async fn process(&self, asset_id: AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
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
                    error_with_fields!(
                        "asset status fetch failed",
                        error.as_ref(),
                        provider = provider,
                        chain = target.chain.as_ref(),
                        token_id = target.token_id.as_str()
                    );
                    (provider, None)
                }
            })
            .collect::<Vec<_>>();
        let verdict = AssetStatusVerdict::from_provider_results(&provider_results);

        if verdict.is_malicious {
            self.database
                .assets()?
                .update_assets(vec![asset_id], vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)])?;
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
    use super::AssetStatusVerdict;

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
