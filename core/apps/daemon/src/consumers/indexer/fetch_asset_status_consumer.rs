use std::error::Error;

use async_trait::async_trait;
use futures::future;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{AssetId, asset_score::AssetRank};
use security_provider::{ScanProviders, TokenTarget};
use storage::{AssetUpdate, AssetsRepository, Database};
use streamer::consumer::MessageConsumer;

pub struct FetchAssetStatusConsumer {
    pub database: Database,
    pub providers: ScanProviders,
}

#[derive(Debug, PartialEq, Eq)]
struct AssetStatusVerdict {
    is_malicious: bool,
    is_complete: bool,
}

impl AssetStatusVerdict {
    fn from_provider_results(results: &[Option<bool>]) -> Self {
        Self {
            is_malicious: results.contains(&Some(true)),
            is_complete: !results.is_empty() && !results.contains(&None),
        }
    }
}

#[async_trait]
impl MessageConsumer<AssetId, bool> for FetchAssetStatusConsumer {
    async fn should_process(&self, asset_id: &AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(asset_id.is_token() && self.providers.iter().any(|provider| provider.supports_token_chain(asset_id.chain)))
    }

    async fn process(&self, asset_id: AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let token_id = asset_id.get_token_id()?.clone();
        let target = TokenTarget { token_id, chain: asset_id.chain };
        let results = future::join_all(
            self.providers
                .iter()
                .filter(|provider| provider.supports_token_chain(target.chain))
                .map(|provider| async { (provider.name(), provider.scan_token(&target).await) }),
        )
        .await;
        let provider_results = results
            .into_iter()
            .map(|(provider, result)| match result {
                Ok(result) => {
                    info_with_fields!(
                        "asset status result",
                        provider = result.provider.as_str(),
                        chain = result.target.chain.as_ref(),
                        token_id = result.target.token_id.as_str(),
                        malicious = result.is_malicious,
                        reason = result.reason.as_deref().unwrap_or_default()
                    );
                    Some(result.is_malicious)
                }
                Err(error) => {
                    error_with_fields!(
                        "asset status fetch failed",
                        error.as_ref(),
                        provider = provider,
                        chain = target.chain.as_ref(),
                        token_id = target.token_id.as_str()
                    );
                    None
                }
            })
            .collect::<Vec<_>>();
        let verdict = AssetStatusVerdict::from_provider_results(&provider_results);

        if verdict.is_malicious {
            self.database
                .assets()?
                .update_assets(vec![asset_id], vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)])?;
        }
        if !verdict.is_complete {
            let provider_failures = provider_results.iter().filter(|result| result.is_none()).count();
            return Err(format!("{provider_failures} asset status provider requests failed").into());
        }
        Ok(verdict.is_malicious)
    }
}

#[cfg(test)]
mod tests {
    use super::AssetStatusVerdict;

    #[test]
    fn test_provider_results_require_complete_safe_verdict() {
        assert_eq!(
            AssetStatusVerdict::from_provider_results(&[Some(false), Some(false)]),
            AssetStatusVerdict {
                is_malicious: false,
                is_complete: true,
            }
        );
        assert_eq!(
            AssetStatusVerdict::from_provider_results(&[Some(false), None]),
            AssetStatusVerdict {
                is_malicious: false,
                is_complete: false,
            }
        );
        assert_eq!(
            AssetStatusVerdict::from_provider_results(&[Some(true), None]),
            AssetStatusVerdict {
                is_malicious: true,
                is_complete: false,
            }
        );
    }
}
