use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cacher::{AccessTokenCacherClient, CacherClient};
use gem_client::ReqwestClient;
use gem_tracing::{DurationMs, info_with_fields};
use primitives::{AssetId, ChainAddress, ConfigKey, ConfigParamKey, ScanProvider, ScanSource, ScanTransaction, ScanTransactionPayload, TransactionType, asset_score::AssetRank};
use reqwest::Url;
use rocket::futures::future;
use security_provider::providers::goplus::GoPlusProvider;
use security_provider::{
    AddressPoisoningTarget, AddressScanProviderConfig, AddressTarget, ScanProviderFactory, ScanProviderRemoteConfig, ScanResult, TransactionScanProviders, WebsiteTarget,
};
use serde::Serialize;
use serde_json::json;
use settings::Settings;
use storage::{AssetsRepository, ConfigRepository, Database, ScanAddressesRepository};

use crate::metrics::Metrics;

pub fn scan_providers(settings: &Settings, cacher: CacherClient, timeout: Duration) -> Result<TransactionScanProviders, Box<dyn Error + Send + Sync>> {
    let config = AddressScanProviderConfig {
        timeout,
        goplus: ScanProviderRemoteConfig {
            url: settings.security.goplus.url.clone(),
            public_key: settings.security.goplus.key.public.clone(),
            secret_key: settings.security.goplus.key.secret.clone(),
        },
        hashdit: settings.security.hashdit.remote_provider_config(),
        tronscan: settings.security.tronscan.remote_provider_config(),
    };
    ScanProviderFactory::new_transaction_providers(config, Arc::new(AccessTokenCacherClient::new(cacher, GoPlusProvider::<ReqwestClient>::NAME)))
}

#[derive(Clone)]
pub struct TransactionScanConfig {
    pub providers: TransactionScanProviders,
    pub required_successes: usize,
}

#[derive(Serialize)]
struct ScanCheck {
    latency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    malicious: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ScanCheck {
    fn new<T>(result: Result<ScanResult<T>, Box<dyn Error + Send + Sync>>, duration: Duration) -> Self {
        let (malicious, reason, error) = match result {
            Ok(result) => (Some(result.is_malicious), result.reason, None),
            Err(error) => (None, None, Some(error.to_string())),
        };
        Self {
            latency: DurationMs(duration).to_string(),
            malicious,
            reason,
            error,
        }
    }
}

#[derive(Clone)]
pub struct ScanClient {
    database: Database,
    config: TransactionScanConfig,
    metrics: Arc<Metrics>,
}

impl ScanClient {
    pub fn new(database: Database, config: TransactionScanConfig, metrics: Arc<Metrics>) -> Self {
        Self { database, config, metrics }
    }

    pub async fn get_scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let is_enabled = self.database.client()?.get_config_bool(ConfigKey::ScanEnable)?;
        if !is_enabled {
            return Ok(ScanTransaction::disabled());
        }
        let dry_run = self.database.client()?.get_config_bool(ConfigKey::ScanDryRun)?;
        let (local_scan, is_target_verified) = self.get_scan_transaction_local(&payload)?;
        if local_scan.is_malicious == Some(true) || is_target_verified {
            return Ok(Self::scan_transaction_response(&payload, local_scan, ScanSource::Local, dry_run, &BTreeMap::new()));
        }

        let Some((address_target, poisoning_target, website_target)) = Self::provider_targets(&payload) else {
            return Ok(Self::scan_transaction_response(&payload, local_scan, ScanSource::Local, dry_run, &BTreeMap::new()));
        };
        let enabled = {
            let mut database = self.database.client()?;
            let mut enabled = Vec::new();
            for provider in ScanProvider::all() {
                if database.get_config_param_bool(ConfigParamKey::ScanProviderEnable(provider))? {
                    enabled.push(provider);
                }
            }
            enabled
        };
        let providers = self.config.providers.filter_enabled(&enabled);
        let (address_scans, poisoning_scans, website_scans) = future::join3(
            self.scan_address_providers(&providers, address_target.clone()),
            self.scan_address_poisoning_providers(&providers, poisoning_target),
            self.scan_website_providers(&providers, website_target.clone()),
        )
        .await;

        let is_malicious_address = address_scans.iter().chain(&poisoning_scans).any(|(_, scan)| scan.malicious == Some(true));
        let malicious_addresses = is_malicious_address
            .then_some(ChainAddress::new(address_target.chain, address_target.address))
            .into_iter()
            .collect::<Vec<_>>();
        let is_malicious_website = website_scans.iter().any(|(_, scan)| scan.malicious == Some(true));
        let malicious_website = website_target.filter(|_| is_malicious_website).map(|target| target.website);
        let completed_scans = address_scans
            .iter()
            .chain(&poisoning_scans)
            .chain(&website_scans)
            .map(|(_, scan)| scan.malicious.is_some())
            .collect::<Vec<_>>();
        let is_scan_complete = Self::is_scan_complete(self.config.required_successes, &completed_scans);

        let scan = ScanTransaction {
            is_malicious: Some(!malicious_addresses.is_empty() || malicious_website.is_some()),
            is_memo_required: local_scan.is_memo_required,
            is_scan_complete,
            malicious_addresses: Some(malicious_addresses),
            malicious_assets: local_scan.malicious_assets,
            malicious_website,
        };
        let source = if address_scans.is_empty() && poisoning_scans.is_empty() && website_scans.is_empty() {
            ScanSource::Local
        } else {
            ScanSource::Remote
        };
        let mut providers: BTreeMap<&str, BTreeMap<&str, &ScanCheck>> = BTreeMap::new();
        for (kind, checks) in [("address", &address_scans), ("address_poisoning", &poisoning_scans), ("website", &website_scans)] {
            for (provider, check) in checks {
                providers.entry(provider.as_ref()).or_default().insert(kind, check);
            }
        }
        Ok(Self::scan_transaction_response(&payload, scan, source, dry_run, &providers))
    }

    fn scan_transaction_response(
        payload: &ScanTransactionPayload,
        scan: ScanTransaction,
        source: ScanSource,
        dry_run: bool,
        providers: &BTreeMap<&str, BTreeMap<&str, &ScanCheck>>,
    ) -> ScanTransaction {
        let response = if dry_run { ScanTransaction::disabled() } else { scan.clone() };
        let scan = ScanTransaction {
            malicious_website: scan.malicious_website.as_deref().and_then(Self::website_host),
            ..scan
        };
        let message = if dry_run {
            "security transaction dry run"
        } else if scan.is_malicious == Some(true) {
            "security transaction blocked"
        } else {
            "security transaction result"
        };
        info_with_fields!(
            message,
            transaction_type = payload.transaction_type.as_ref(),
            chain = payload.target.asset_id.chain.as_ref(),
            source = source.as_ref(),
            dry_run = dry_run,
            malicious = scan.is_malicious == Some(true),
            provider_errors = providers.values().flat_map(|checks| checks.values()).filter(|check| check.error.is_some()).count(),
            origin_asset_id = payload.origin.asset_id,
            target_asset_id = payload.target.asset_id,
            address = format!("{:?}", payload.target.address),
            website_host = json!(payload.website.as_deref().and_then(Self::website_host)),
            scan = json!(scan),
            providers = json!(providers)
        );
        response
    }

    fn website_host(website: &str) -> Option<String> {
        Url::parse(website).ok()?.host_str().map(str::to_string)
    }

    fn get_scan_transaction_local(&self, payload: &ScanTransactionPayload) -> Result<(ScanTransaction, bool), Box<dyn Error + Send + Sync>> {
        let queries = [
            (payload.origin.asset_id.chain, payload.origin.address.as_str()),
            (payload.target.asset_id.chain, payload.target.address.as_str()),
        ];
        let addresses = self.database.scan_addresses()?.get_scan_addresses(&queries)?;
        let token_asset_ids = Self::token_asset_ids(payload);
        let token_assets = self.database.assets()?.get_assets_basic(token_asset_ids)?;
        let malicious_addresses = addresses
            .iter()
            .filter(|address| address.is_fraudulent)
            .map(|address| ChainAddress::new(address.chain.0, address.address.clone()))
            .collect::<Vec<_>>();
        let is_memo_required = addresses.iter().any(|address| address.is_memo_required);
        let is_target_verified = addresses
            .iter()
            .any(|address| address.is_verified_for(payload.target.asset_id.chain, &payload.target.address));
        let malicious_assets = token_assets
            .into_iter()
            .filter(|asset| Self::is_malicious_asset_rank(asset.score.rank))
            .map(|asset| asset.asset.id)
            .collect::<Vec<_>>();

        Ok((
            ScanTransaction {
                is_malicious: Some(!malicious_addresses.is_empty() || !malicious_assets.is_empty()),
                is_memo_required: Some(is_memo_required),
                is_scan_complete: true,
                malicious_addresses: Some(malicious_addresses),
                malicious_assets: Some(malicious_assets),
                malicious_website: None,
            },
            is_target_verified,
        ))
    }

    fn provider_targets(payload: &ScanTransactionPayload) -> Option<(AddressTarget, Option<AddressPoisoningTarget>, Option<WebsiteTarget>)> {
        let address = AddressTarget {
            chain: payload.target.asset_id.chain,
            address: payload.target.address.clone(),
        };
        let poisoning = match payload.transaction_type {
            TransactionType::Transfer | TransactionType::TransferNFT => Some(AddressPoisoningTarget {
                target: address.clone(),
                user_address: payload.origin.address.clone(),
            }),
            TransactionType::StakeDelegate
            | TransactionType::StakeUndelegate
            | TransactionType::StakeRewards
            | TransactionType::StakeRedelegate
            | TransactionType::StakeWithdraw
            | TransactionType::StakeFreeze
            | TransactionType::StakeUnfreeze => return None,
            TransactionType::Swap
            | TransactionType::TokenApproval
            | TransactionType::AssetActivation
            | TransactionType::SmartContractCall
            | TransactionType::PerpetualOpenPosition
            | TransactionType::PerpetualClosePosition
            | TransactionType::PerpetualModifyPosition
            | TransactionType::EarnDeposit
            | TransactionType::EarnWithdraw => None,
        };
        let website = payload.website.clone().map(|website| WebsiteTarget { website });
        Some((address, poisoning, website))
    }

    fn is_scan_complete(required_successes: usize, scans: &[bool]) -> bool {
        scans.iter().filter(|is_complete| **is_complete).count() >= required_successes
    }

    fn is_malicious_asset_rank(rank: i32) -> bool {
        rank <= AssetRank::Spam.threshold()
    }

    fn token_asset_ids(payload: &ScanTransactionPayload) -> Vec<AssetId> {
        let mut targets = Vec::new();
        for asset_id in [&payload.origin.asset_id, &payload.target.asset_id] {
            if asset_id.is_native() {
                continue;
            }
            if !targets.contains(asset_id) {
                targets.push(asset_id.clone());
            }
        }
        targets
    }

    async fn scan_address_providers(&self, providers: &TransactionScanProviders, target: AddressTarget) -> Vec<(ScanProvider, ScanCheck)> {
        future::join_all(providers.addresses.iter().filter(|provider| provider.supports_chain(target.chain)).map(|provider| async {
            let start = Instant::now();
            let result = provider.scan_address(&target).await;
            let latency = start.elapsed();
            self.metrics
                .record_scan(provider.provider(), "address", result.as_ref().ok().map(|scan| scan.is_malicious), latency);
            (provider.provider(), ScanCheck::new(result, latency))
        }))
        .await
    }

    async fn scan_address_poisoning_providers(&self, providers: &TransactionScanProviders, target: Option<AddressPoisoningTarget>) -> Vec<(ScanProvider, ScanCheck)> {
        let Some(target) = target else {
            return Vec::new();
        };
        future::join_all(
            providers
                .poisoning
                .iter()
                .filter(|provider| provider.supports_chain(target.target.chain))
                .map(|provider| async {
                    let start = Instant::now();
                    let result = provider.scan_address_poisoning(&target).await;
                    let latency = start.elapsed();
                    self.metrics
                        .record_scan(provider.provider(), "address_poisoning", result.as_ref().ok().map(|scan| scan.is_malicious), latency);
                    (provider.provider(), ScanCheck::new(result, latency))
                }),
        )
        .await
    }

    async fn scan_website_providers(&self, providers: &TransactionScanProviders, target: Option<WebsiteTarget>) -> Vec<(ScanProvider, ScanCheck)> {
        let Some(target) = target else {
            return Vec::new();
        };
        future::join_all(providers.websites.iter().map(|provider| async {
            let start = Instant::now();
            let result = provider.scan_website(&target).await;
            let latency = start.elapsed();
            self.metrics
                .record_scan(provider.provider(), "website", result.as_ref().ok().map(|scan| scan.is_malicious), latency);
            (provider.provider(), ScanCheck::new(result, latency))
        }))
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain};

    #[test]
    fn test_website_host_excludes_credentials_and_query_values() {
        assert_eq!(
            ScanClient::website_host("https://user:password@example.com/path?token=secret#fragment"),
            Some("example.com".into())
        );
        assert_eq!(ScanClient::website_host("invalid website"), None);
    }

    #[test]
    fn test_scan_complete_requires_configured_success_count() {
        assert!(ScanClient::is_scan_complete(1, &[true, false, false]));
        assert!(ScanClient::is_scan_complete(1, &[false, true, false]));
        assert!(ScanClient::is_scan_complete(1, &[false, false, true]));
        assert!(ScanClient::is_scan_complete(2, &[true, false, true]));
        assert!(ScanClient::is_scan_complete(2, &[true, true, true]));
        assert!(!ScanClient::is_scan_complete(2, &[true, false, false]));
        assert!(!ScanClient::is_scan_complete(3, &[true, true]));
        assert!(!ScanClient::is_scan_complete(1, &[false, false]));
        assert!(!ScanClient::is_scan_complete(1, &[]));
        assert!(ScanClient::is_scan_complete(0, &[false, false]));
        assert!(ScanClient::is_scan_complete(0, &[true]));
        assert!(ScanClient::is_scan_complete(0, &[]));
    }

    #[test]
    fn test_spam_or_lower_asset_rank_is_malicious() {
        assert!(!ScanClient::is_malicious_asset_rank(-14));
        assert!(ScanClient::is_malicious_asset_rank(-15));
        assert!(ScanClient::is_malicious_asset_rank(-20));
        assert!(ScanClient::is_malicious_asset_rank(i32::MIN));
    }

    #[test]
    fn test_native_assets_do_not_create_token_asset_ids() {
        let payload = ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Ethereum));

        assert!(ScanClient::token_asset_ids(&payload).is_empty());
    }

    #[test]
    fn test_same_token_is_looked_up_once() {
        let token = AssetId::from_token(Chain::SmartChain, "0x123");
        let payload = ScanTransactionPayload::mock_with_assets(token.clone(), token);

        assert_eq!(ScanClient::token_asset_ids(&payload).len(), 1);
    }

    #[test]
    fn test_swap_looks_up_both_distinct_token_assets() {
        let payload = ScanTransactionPayload {
            transaction_type: TransactionType::Swap,
            ..ScanTransactionPayload::mock_with_assets(AssetId::from_token(Chain::Ethereum, "0x123"), AssetId::from_token(Chain::SmartChain, "0x456"))
        };

        assert_eq!(
            ScanClient::token_asset_ids(&payload),
            vec![AssetId::from_token(Chain::Ethereum, "0x123"), AssetId::from_token(Chain::SmartChain, "0x456"),]
        );
    }

    #[test]
    fn test_provider_targets_use_recipient_context() {
        let payload = ScanTransactionPayload {
            website: Some("https://example.com".to_string()),
            ..ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::SmartChain), AssetId::from_token(Chain::SmartChain, "0x456"))
        };

        assert_eq!(
            ScanClient::provider_targets(&payload).unwrap(),
            (
                AddressTarget {
                    chain: Chain::SmartChain,
                    address: "target".to_string(),
                },
                Some(AddressPoisoningTarget {
                    target: AddressTarget {
                        chain: Chain::SmartChain,
                        address: "target".to_string(),
                    },
                    user_address: "origin".to_string(),
                }),
                Some(WebsiteTarget {
                    website: "https://example.com".to_string(),
                }),
            )
        );
    }

    #[test]
    fn test_provider_targets_skip_staking() {
        for transaction_type in [
            TransactionType::StakeDelegate,
            TransactionType::StakeUndelegate,
            TransactionType::StakeRewards,
            TransactionType::StakeRedelegate,
            TransactionType::StakeWithdraw,
            TransactionType::StakeFreeze,
            TransactionType::StakeUnfreeze,
        ] {
            let payload = ScanTransactionPayload {
                website: Some("https://example.com".to_string()),
                transaction_type,
                ..ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::Monad), AssetId::from_chain(Chain::Monad))
            };

            assert_eq!(ScanClient::provider_targets(&payload), None);
        }
    }

    #[test]
    fn test_provider_targets_skip_poisoning_for_contract_calls() {
        let payload = ScanTransactionPayload {
            transaction_type: TransactionType::Swap,
            ..ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::Ethereum), AssetId::from_token(Chain::SmartChain, "0x456"))
        };

        assert_eq!(ScanClient::provider_targets(&payload).unwrap().1, None);
    }
}
