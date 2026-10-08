use std::collections::HashSet;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cacher::{AccessTokenCacherClient, CacherClient, SafeScanTarget, ScanSafeCacher};
use futures::future;
use gem_client::ReqwestClient;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{ScanOutcome, ScanProvider, ScanTransaction, ScanTransactionPayload, ScanType};
use security::providers::goplus::GoPlusProvider;
use security::transaction_scan::{ProviderCheck, ScanSubject, ScanTargets, TransactionScanInput, TransactionScanResult, evaluate_transaction_scan, plan_transaction_scan, scan_subjects, token_asset_ids, website_host};
use security::{ScanProviderConfig, ScanProviderFactory, ScanResult, TransactionScanProviders};
use serde_json::json;
use settings::Settings;

use super::repository::{Repository, ScanRecords};
use super::scan_config::ScanConfig;
use crate::ConfigCacher;

pub trait ScanMetrics: Send + Sync {
    fn record_scan(&self, provider: ScanProvider, scan_type: ScanType, outcome: ScanOutcome, latency: Duration);
}

pub fn scan_providers(settings: &Settings, cacher: CacherClient, timeout: Duration) -> Result<TransactionScanProviders, Box<dyn Error + Send + Sync>> {
    let config = ScanProviderConfig::new(&settings.security, timeout);
    ScanProviderFactory::new_transaction_providers(&config, Arc::new(AccessTokenCacherClient::new(cacher, GoPlusProvider::<ReqwestClient>::NAME)))
}

pub struct ScanClient {
    repository: Arc<dyn Repository>,
    config_cacher: Arc<ConfigCacher>,
    safe_targets: Arc<dyn ScanSafeCacher>,
    providers: TransactionScanProviders,
    metrics: Arc<dyn ScanMetrics>,
}

impl ScanClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, config_cacher: Arc<ConfigCacher>, safe_targets: Arc<dyn ScanSafeCacher>, providers: TransactionScanProviders, metrics: Arc<dyn ScanMetrics>) -> Self {
        Self {
            repository,
            config_cacher,
            safe_targets,
            providers,
            metrics,
        }
    }

    pub async fn get_scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let config = ScanConfig::from_config(&self.config_cacher).await?;
        let subjects = scan_subjects(&payload);
        let safe_targets = Self::safe_targets(&config, &subjects);
        let safe = self.get_cached_safe(&safe_targets).await;
        let input = self.get_scan_input(&config, payload, &subjects, safe).await?;
        let plan = plan_transaction_scan(&input, &config.enabled_providers);
        let checks = match &plan.targets {
            Some(targets) => self.run_checks(&config.enabled_providers, targets).await?,
            None => Vec::new(),
        };
        let result = evaluate_transaction_scan(&input, plan, checks);
        if let Err(error) = self.save(&safe_targets, &result).await {
            error_with_fields!("security scan save failed", &*error, chain = input.payload.target.asset_id.chain.as_ref());
        }
        Self::log(&input.payload, &result);
        Ok(result.scan)
    }

    async fn get_scan_input(&self, config: &ScanConfig, payload: ScanTransactionPayload, subjects: &[ScanSubject], safe: HashSet<ScanType>) -> Result<TransactionScanInput, Box<dyn Error + Send + Sync>> {
        let addresses = vec![(payload.origin.asset_id.chain, payload.origin.address.clone()), (payload.target.asset_id.chain, payload.target.address.clone())];
        let asset_ids = token_asset_ids(&payload);
        let mut targets = subjects.iter().map(|subject| subject.target.clone()).collect::<Vec<_>>();
        targets.dedup();
        let detection_max_age = if targets.is_empty() { None } else { Some(config.detection_max_age) };
        let ScanRecords { addresses, assets, verdicts } = self.repository.scan_records(addresses, asset_ids, targets, detection_max_age).await?;
        Ok(TransactionScanInput {
            payload,
            enforced: config.enforced.clone(),
            addresses,
            assets,
            verdicts,
            safe,
            required_successes: config.required_successes,
        })
    }

    fn safe_targets(config: &ScanConfig, subjects: &[ScanSubject]) -> Vec<SafeScanTarget> {
        subjects
            .iter()
            .filter(|subject| subject.scan_type.is_safe_cacheable())
            .filter_map(|subject| {
                let ttl = config.safe_cache_ttl(subject.scan_type);
                (ttl > 0).then(|| SafeScanTarget {
                    scan_type: subject.scan_type,
                    target: subject.cache_key(),
                    ttl,
                })
            })
            .collect()
    }

    async fn get_cached_safe(&self, targets: &[SafeScanTarget]) -> HashSet<ScanType> {
        let mut safe = HashSet::new();
        for target in targets {
            match self.safe_targets.is_safe(target).await {
                Ok(true) => {
                    safe.insert(target.scan_type);
                }
                Ok(false) => {}
                Err(error) => error_with_fields!("security scan safe cache read failed", &*error, scan_type = target.scan_type.as_ref()),
            }
        }
        safe
    }

    async fn save(&self, safe_targets: &[SafeScanTarget], result: &TransactionScanResult) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.repository.add_scan_detections(result.new_verdicts.clone()).await?;
        let new_safe = safe_targets.iter().filter(|target| result.new_safe.contains(&target.scan_type)).collect::<Vec<_>>();
        self.safe_targets.add_safe(&new_safe).await
    }

    async fn run_checks(&self, enabled_providers: &HashSet<ScanProvider>, targets: &ScanTargets) -> Result<Vec<ProviderCheck>, Box<dyn Error + Send + Sync>> {
        let providers = self.providers.filter_enabled(enabled_providers);
        let (addresses, poisoning, websites) = future::join3(
            future::join_all(targets.address.iter().flat_map(|target| {
                providers
                    .addresses
                    .iter()
                    .filter(|provider| provider.supports_chain(target.chain))
                    .map(move |provider| self.run_check(provider.provider(), ScanType::Address, provider.scan_address(target)))
            })),
            future::join_all(targets.poisoning.iter().flat_map(|target| {
                providers
                    .poisoning
                    .iter()
                    .filter(|provider| provider.supports_chain(target.target.chain))
                    .map(move |provider| self.run_check(provider.provider(), ScanType::AddressPoisoning, provider.scan_address_poisoning(target)))
            })),
            future::join_all(
                targets
                    .website
                    .iter()
                    .flat_map(|target| providers.websites.iter().map(move |provider| self.run_check(provider.provider(), ScanType::Website, provider.scan_website(target)))),
            ),
        )
        .await;
        Ok([addresses, poisoning, websites].concat())
    }

    async fn run_check<T>(&self, provider: ScanProvider, scan_type: ScanType, request: impl Future<Output = Result<ScanResult<T>, Box<dyn Error + Send + Sync>>>) -> ProviderCheck {
        let start = Instant::now();
        let result = request.await;
        let latency = start.elapsed();
        let check = ProviderCheck::new(provider, scan_type, result, latency);
        self.metrics.record_scan(provider, scan_type, check.outcome, latency);
        check
    }

    fn log(payload: &ScanTransactionPayload, result: &TransactionScanResult) {
        let transaction_type = payload.transaction_type.as_ref();
        let chain = payload.target.asset_id.chain.as_ref();
        let has_token_assets = !token_asset_ids(payload).is_empty();
        let scan_types = ScanType::all()
            .into_iter()
            .filter(|scan_type| match scan_type {
                ScanType::Address | ScanType::AddressPoisoning | ScanType::Website => result.subjects.iter().any(|subject| subject.scan_type == *scan_type),
                ScanType::Asset => has_token_assets,
            })
            .collect::<Vec<_>>();
        let scan_types = format!("|{}|", scan_types.iter().map(AsRef::as_ref).collect::<Vec<_>>().join("|"));
        let providers = ScanProvider::all()
            .into_iter()
            .filter(|provider| result.checks.iter().any(|check| check.provider == *provider) || result.detections.iter().any(|detection| detection.provider == *provider))
            .collect::<Vec<_>>();
        let scan_providers = providers.iter().map(AsRef::as_ref).collect::<Vec<_>>().join("|");
        let scan_providers = format!("|{scan_providers}|");
        let website_host = website_host(payload);
        let target = if payload.target.address.is_empty() {
            website_host.clone().unwrap_or_default()
        } else {
            payload.target.address.clone()
        };
        let message = if result.scan.is_malicious == Some(true) { "security transaction blocked" } else { "security transaction result" };
        info_with_fields!(
            message,
            transaction_type = transaction_type,
            chain = chain,
            source = result.source.as_ref(),
            scan_types = scan_types,
            scan_providers = scan_providers,
            malicious = result.scan.is_malicious == Some(true),
            target = format!("{target:?}"),
            origin_asset_id = payload.origin.asset_id,
            target_asset_id = payload.target.asset_id,
            website_host = json!(website_host),
            cached_safe = json!(result.safe)
        );
        for detection in &result.detections {
            info_with_fields!(
                "security finding",
                transaction_type = transaction_type,
                chain = chain,
                scan_type = detection.scan_type.as_ref(),
                target = format!("{:?}", detection.target),
                provider = detection.provider.as_ref(),
                reason = format!("{:?}", detection.reason.as_deref().unwrap_or_default()),
                enforced = detection.is_enforced,
                cached = detection.is_cached
            );
        }
        for check in result.checks.iter().filter(|check| check.outcome == ScanOutcome::Error) {
            info_with_fields!(
                "security provider error",
                transaction_type = transaction_type,
                chain = chain,
                scan_type = check.scan_type.as_ref(),
                target = format!("{:?}", result.subject_target(check.scan_type)),
                provider = check.provider.as_ref(),
                error = format!("{:?}", check.error.as_deref().unwrap_or_default())
            );
        }
    }
}
