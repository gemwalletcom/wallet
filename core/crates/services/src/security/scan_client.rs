use std::collections::HashSet;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use cacher::{AccessTokenCacherClient, CacheKey, CacherClient};
use config_keys::{ConfigKey, ConfigParamKey};
use futures::future;
use gem_client::ReqwestClient;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{ScanOutcome, ScanProvider, ScanSource, ScanTransaction, ScanTransactionPayload, ScanType};
use security::providers::goplus::GoPlusProvider;
use security::transaction_scan::{ProviderCheck, ScanSubject, ScanTargets, TransactionScanInput, TransactionScanResult, evaluate_transaction_scan, plan_transaction_scan, scan_subjects, token_asset_ids, website_host};
use security::{ScanProviderConfig, ScanProviderFactory, ScanResult, TransactionScanProviders};
use serde_json::json;
use settings::Settings;
use storage::{AssetsRepository, Database, DatabaseError, ScanAddressesRepository, ScanDetectionsRepository};

use crate::ConfigCacher;

pub trait ScanMetrics: Send + Sync {
    fn record_scan(&self, provider: ScanProvider, scan_type: ScanType, outcome: ScanOutcome, latency: Duration);
}

pub fn scan_providers(settings: &Settings, cacher: CacherClient, timeout: Duration) -> Result<TransactionScanProviders, Box<dyn Error + Send + Sync>> {
    let config = ScanProviderConfig::new(&settings.security, timeout);
    ScanProviderFactory::new_transaction_providers(&config, Arc::new(AccessTokenCacherClient::new(cacher, GoPlusProvider::<ReqwestClient>::NAME)))
}

struct SafeCacheKey {
    scan_type: ScanType,
    target: String,
    ttl: u64,
}

impl SafeCacheKey {
    fn key(&self) -> CacheKey<'_> {
        CacheKey::ScanSafe(self.scan_type.as_ref(), &self.target, self.ttl)
    }
}

pub struct ScanClient {
    database: Database,
    config_cacher: Arc<ConfigCacher>,
    cacher: CacherClient,
    providers: TransactionScanProviders,
    metrics: Arc<dyn ScanMetrics>,
}

impl ScanClient {
    pub fn new(database: Database, config_cacher: Arc<ConfigCacher>, cacher: CacherClient, providers: TransactionScanProviders, metrics: Arc<dyn ScanMetrics>) -> Self {
        Self {
            database,
            config_cacher,
            cacher,
            providers,
            metrics,
        }
    }

    pub async fn get_scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let subjects = scan_subjects(&payload);
        let safe_keys = self.get_safe_cache_keys(&subjects).await?;
        let safe = self.get_cached_safe(&safe_keys).await;
        let input = self.get_scan_input(payload, &subjects, safe).await?;
        let plan = plan_transaction_scan(&input);
        let checks = match &plan.targets {
            Some(targets) => self.run_checks(targets).await?,
            None => Vec::new(),
        };
        let result = evaluate_transaction_scan(&input, plan, checks);
        if let Err(error) = self.save(&safe_keys, &result).await {
            error_with_fields!("security scan save failed", &*error, chain = input.payload.target.asset_id.chain.as_ref());
        }
        Self::log(&input.payload, &result);
        Ok(result.scan)
    }

    async fn get_scan_input(&self, payload: ScanTransactionPayload, subjects: &[ScanSubject], safe: HashSet<ScanType>) -> Result<TransactionScanInput, Box<dyn Error + Send + Sync>> {
        let mut enforced = HashSet::new();
        for scan_type in ScanType::all() {
            if self.config_cacher.get_param_bool(&ConfigParamKey::ScanTypeEnable(scan_type)).await? {
                enforced.insert(scan_type);
            }
        }
        let queries = [(payload.origin.asset_id.chain, payload.origin.address.clone()), (payload.target.asset_id.chain, payload.target.address.clone())];
        let asset_ids = token_asset_ids(&payload);
        let mut targets = subjects.iter().map(|subject| subject.target.clone()).collect::<Vec<_>>();
        targets.dedup();
        let detection_max_age = if targets.is_empty() { None } else { Some(self.config_cacher.get_duration(ConfigKey::ScanDetectionMaxAge).await?) };
        let (addresses, assets, verdicts) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let queries = queries.iter().map(|(chain, address)| (*chain, address.as_str())).collect::<Vec<_>>();
                let addresses = client.get_scan_addresses(&queries)?;
                let assets = client.get_assets_basic(asset_ids)?;
                let verdicts = match detection_max_age {
                    Some(max_age) => client.get_scan_detections(targets, max_age)?,
                    None => Vec::new(),
                };
                Ok((addresses, assets, verdicts))
            })
            .await?;
        Ok(TransactionScanInput {
            payload,
            enforced,
            addresses,
            assets,
            verdicts,
            safe,
            required_successes: self.config_cacher.get_usize(ConfigKey::ScanRequiredSuccesses).await?,
        })
    }

    async fn get_safe_cache_keys(&self, subjects: &[ScanSubject]) -> Result<Vec<SafeCacheKey>, Box<dyn Error + Send + Sync>> {
        let mut keys = Vec::new();
        for subject in subjects.iter().filter(|subject| subject.scan_type.is_safe_cacheable()) {
            let ttl = self.config_cacher.get_param_duration(&ConfigParamKey::ScanSafeCacheDuration(subject.scan_type)).await?.as_secs();
            if ttl > 0 {
                keys.push(SafeCacheKey {
                    scan_type: subject.scan_type,
                    target: subject.cache_key(),
                    ttl,
                });
            }
        }
        Ok(keys)
    }

    async fn get_cached_safe(&self, keys: &[SafeCacheKey]) -> HashSet<ScanType> {
        let mut safe = HashSet::new();
        for key in keys {
            match self.cacher.get_cached_optional::<bool>(key.key()).await {
                Ok(Some(_)) => {
                    safe.insert(key.scan_type);
                }
                Ok(None) => {}
                Err(error) => error_with_fields!("security scan safe cache read failed", &*error, scan_type = key.scan_type.as_ref()),
            }
        }
        safe
    }

    async fn save(&self, safe_keys: &[SafeCacheKey], result: &TransactionScanResult) -> Result<(), Box<dyn Error + Send + Sync>> {
        let verdicts = result.new_verdicts.clone();
        self.database.run(move |client| client.add_scan_detections(verdicts)).await?;
        let entries = safe_keys.iter().filter(|key| result.new_safe.contains(&key.scan_type)).map(|key| (key.key(), &true)).collect::<Vec<_>>();
        self.cacher.set_values_cached(&entries).await?;
        Ok(())
    }

    async fn run_checks(&self, targets: &ScanTargets) -> Result<Vec<ProviderCheck>, Box<dyn Error + Send + Sync>> {
        let mut enabled = Vec::new();
        for provider in ScanProvider::all() {
            if self.config_cacher.get_param_bool(&ConfigParamKey::ScanProviderEnable(provider)).await? {
                enabled.push(provider);
            }
        }
        let providers = self.providers.filter_enabled(&enabled);
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
        let scan_types = format!("|{}|", scan_types.iter().map(|scan_type| scan_type.as_ref()).collect::<Vec<_>>().join("|"));
        let providers = ScanProvider::all()
            .into_iter()
            .filter(|provider| result.checks.iter().any(|check| check.provider == *provider) || result.detections.iter().any(|detection| detection.provider == Some(*provider)))
            .collect::<Vec<_>>();
        let scan_providers = (result.source == ScanSource::Local)
            .then_some("internal")
            .into_iter()
            .chain(providers.iter().map(|provider| provider.as_ref()))
            .collect::<Vec<_>>()
            .join("|");
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
                provider = detection.provider.as_ref().map_or("internal", |provider| provider.as_ref()),
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
