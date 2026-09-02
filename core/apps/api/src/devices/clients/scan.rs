use cacher::{AccessTokenCacherClient, CacherClient};
use gem_client::ReqwestClient;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{AssetId, ChainAddress, ScanTransaction, ScanTransactionPayload, asset_score::AssetRank};
use rocket::futures::future;
use security_provider::providers::goplus::GoPlusProvider;
use security_provider::{AddressTarget, ScanProviderConfig, ScanProviderFactory, ScanProviderRemoteConfig, ScanProviders, ScanResult};
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use storage::{AssetsRepository, Database, ScanAddressesRepository};

pub fn scan_providers(settings: &Settings, cacher: CacherClient, timeout: Duration) -> Result<ScanProviders, Box<dyn Error + Send + Sync>> {
    let config = ScanProviderConfig {
        timeout,
        goplus: ScanProviderRemoteConfig {
            url: settings.security.goplus.url.clone(),
            public_key: settings.security.goplus.key.public.clone(),
            secret_key: settings.security.goplus.key.secret.clone(),
        },
        hashdit: ScanProviderRemoteConfig {
            url: settings.security.hashdit.url.clone(),
            public_key: settings.security.hashdit.key.public.clone(),
            secret_key: settings.security.hashdit.key.secret.clone(),
        },
    };
    ScanProviderFactory::new_providers(config, Arc::new(AccessTokenCacherClient::new(cacher, GoPlusProvider::<ReqwestClient>::NAME)))
}

#[derive(Clone)]
pub struct ScanClient {
    database: Database,
    providers: ScanProviders,
    enable: bool,
}

impl ScanClient {
    pub fn new(database: Database, providers: ScanProviders, enable: bool) -> Self {
        Self { database, providers, enable }
    }

    pub async fn get_scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let local_scan = self.get_scan_transaction_local(payload.clone())?;
        if local_scan.is_malicious == Some(true) {
            return Ok(local_scan);
        }

        let target = AddressTarget {
            chain: payload.origin.asset_id.chain,
            address: payload.origin.address.clone(),
        };
        let address_scans = self.scan_address_providers(target).await;

        let malicious_addresses = address_scans
            .iter()
            .flatten()
            .any(|scan| scan.is_malicious)
            .then_some(ChainAddress::new(payload.origin.asset_id.chain, payload.origin.address))
            .into_iter()
            .collect::<Vec<_>>();
        let is_scan_complete = Self::is_scan_complete(self.enable, &address_scans);

        Ok(ScanTransaction {
            is_malicious: Some(!malicious_addresses.is_empty()),
            is_memo_required: local_scan.is_memo_required,
            is_scan_complete,
            malicious_addresses: Some(malicious_addresses),
            malicious_assets: local_scan.malicious_assets,
            malicious_website: None,
        })
    }

    fn get_scan_transaction_local(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let queries = [
            (payload.origin.asset_id.chain, payload.origin.address.as_str()),
            (payload.target.asset_id.chain, payload.target.address.as_str()),
        ];
        let addresses = self.database.scan_addresses()?.get_scan_addresses(&queries)?;
        let token_asset_ids = Self::token_asset_ids(&payload);
        let token_assets = self.database.assets()?.get_assets_basic(token_asset_ids)?;
        let malicious_addresses = addresses
            .iter()
            .filter(|address| address.is_fraudulent)
            .map(|address| ChainAddress::new(address.chain.0, address.address.clone()))
            .collect::<Vec<_>>();
        let is_memo_required = addresses.iter().any(|address| address.is_memo_required);
        let malicious_assets = token_assets
            .into_iter()
            .filter(|asset| asset.score.rank_type() == AssetRank::Fraudulent)
            .map(|asset| asset.asset.id)
            .collect::<Vec<_>>();

        Ok(ScanTransaction {
            is_malicious: Some(!malicious_addresses.is_empty() || !malicious_assets.is_empty()),
            is_memo_required: Some(is_memo_required),
            is_scan_complete: true,
            malicious_addresses: Some(malicious_addresses),
            malicious_assets: Some(malicious_assets),
            malicious_website: None,
        })
    }

    fn is_scan_complete<T>(enable: bool, scans: &[Option<T>]) -> bool {
        enable && !scans.is_empty() && scans.iter().all(Option::is_some)
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

    pub async fn scan_address_providers(&self, target: AddressTarget) -> Vec<Option<ScanResult<AddressTarget>>> {
        if !self.enable {
            return Vec::new();
        }
        future::join_all(
            self.providers
                .iter()
                .filter(|provider| provider.supports_address_chain(target.chain))
                .map(|provider| async { (provider.name(), provider.scan_address(&target).await) }),
        )
        .await
        .into_iter()
        .map(|(provider, result)| match result {
            Ok(result) => {
                info_with_fields!(
                    "security scan result",
                    kind = "address",
                    provider = result.provider.as_str(),
                    chain = result.target.chain.as_ref(),
                    malicious = result.is_malicious,
                    reason = result.reason.as_deref().unwrap_or_default()
                );
                Some(result)
            }
            Err(error) => {
                error_with_fields!("security scan failed", error.as_ref(), kind = "address", provider = provider, chain = target.chain.as_ref());
                None
            }
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, ScanAddressTarget, TransactionType};

    fn payload(origin: AssetId, target: AssetId, transaction_type: TransactionType) -> ScanTransactionPayload {
        ScanTransactionPayload {
            origin: ScanAddressTarget {
                asset_id: origin,
                address: "origin".into(),
            },
            target: ScanAddressTarget {
                asset_id: target,
                address: "target".into(),
            },
            website: None,
            transaction_type,
        }
    }

    #[test]
    fn test_scan_complete_requires_provider_results() {
        assert!(ScanClient::is_scan_complete(true, &[Some(()), Some(())]));
        assert!(!ScanClient::is_scan_complete(true, &[Some(()), None]));
        assert!(!ScanClient::is_scan_complete::<()>(true, &[]));
        assert!(!ScanClient::is_scan_complete(false, &[Some(()), Some(())]));
        assert!(!ScanClient::is_scan_complete::<()>(false, &[]));
    }

    #[test]
    fn test_native_assets_do_not_create_token_asset_ids() {
        let payload = payload(AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer);

        assert!(ScanClient::token_asset_ids(&payload).is_empty());
    }

    #[test]
    fn test_same_token_is_looked_up_once() {
        let token = AssetId::from_token(Chain::SmartChain, "0x123");
        let payload = payload(token.clone(), token, TransactionType::Transfer);

        assert_eq!(ScanClient::token_asset_ids(&payload).len(), 1);
    }

    #[test]
    fn test_swap_looks_up_both_distinct_token_assets() {
        let payload = payload(
            AssetId::from_token(Chain::Ethereum, "0x123"),
            AssetId::from_token(Chain::SmartChain, "0x456"),
            TransactionType::Swap,
        );

        assert_eq!(
            ScanClient::token_asset_ids(&payload),
            vec![AssetId::from_token(Chain::Ethereum, "0x123"), AssetId::from_token(Chain::SmartChain, "0x456"),]
        );
    }
}
