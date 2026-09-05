use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::{AccessTokenCacherClient, CacherClient};
use gem_client::ReqwestClient;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{AssetId, ChainAddress, ScanTransaction, ScanTransactionPayload, TransactionType, asset_score::AssetRank};
use rocket::futures::future;
use security_provider::providers::goplus::GoPlusProvider;
use security_provider::{
    AddressPoisoningTarget, AddressScanProviderConfig, AddressTarget, ScanProviderFactory, ScanProviderRemoteConfig, ScanResult, TransactionScanProviders, WebsiteTarget,
};
use settings::Settings;
use storage::{AssetsRepository, Database, ScanAddressesRepository};

pub fn scan_providers(settings: &Settings, cacher: CacherClient, timeout: Duration) -> Result<TransactionScanProviders, Box<dyn Error + Send + Sync>> {
    let config = AddressScanProviderConfig {
        timeout,
        goplus: ScanProviderRemoteConfig {
            url: settings.security.goplus.url.clone(),
            public_key: settings.security.goplus.key.public.clone(),
            secret_key: settings.security.goplus.key.secret.clone(),
        },
        hashdit: settings.security.hashdit.remote_provider_config(),
    };
    ScanProviderFactory::new_transaction_providers(config, Arc::new(AccessTokenCacherClient::new(cacher, GoPlusProvider::<ReqwestClient>::NAME)))
}

#[derive(Clone)]
pub struct ScanClient {
    database: Database,
    providers: TransactionScanProviders,
    enable: bool,
}

impl ScanClient {
    pub fn new(database: Database, providers: TransactionScanProviders, enable: bool) -> Self {
        Self { database, providers, enable }
    }

    pub async fn get_scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let local_scan = self.get_scan_transaction_local(payload.clone())?;
        if local_scan.is_malicious == Some(true) {
            return Ok(local_scan);
        }

        let (address_target, poisoning_target, website_target) = Self::provider_targets(&payload);
        let (address_scans, poisoning_scans, website_scans) = future::join3(
            self.scan_address_providers(address_target.clone()),
            self.scan_address_poisoning_providers(poisoning_target),
            self.scan_website_providers(website_target.clone()),
        )
        .await;

        let is_malicious_address = address_scans.iter().flatten().any(|scan| scan.is_malicious) || poisoning_scans.iter().flatten().any(|scan| scan.is_malicious);
        let malicious_addresses = is_malicious_address
            .then_some(ChainAddress::new(address_target.chain, address_target.address))
            .into_iter()
            .collect::<Vec<_>>();
        let is_malicious_website = website_scans.iter().flatten().any(|scan| scan.is_malicious);
        let malicious_website = website_target.filter(|_| is_malicious_website).map(|target| target.website);
        let completed_scans = address_scans
            .iter()
            .map(Option::is_some)
            .chain(poisoning_scans.iter().map(Option::is_some))
            .chain(website_scans.iter().map(Option::is_some))
            .collect::<Vec<_>>();
        let is_scan_complete = Self::is_scan_complete(self.enable, !address_scans.is_empty(), &completed_scans);

        Ok(ScanTransaction {
            is_malicious: Some(!malicious_addresses.is_empty() || malicious_website.is_some()),
            is_memo_required: local_scan.is_memo_required,
            is_scan_complete,
            malicious_addresses: Some(malicious_addresses),
            malicious_assets: local_scan.malicious_assets,
            malicious_website,
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
            .filter(|asset| Self::is_malicious_asset_rank(asset.score.rank))
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

    fn provider_targets(payload: &ScanTransactionPayload) -> (AddressTarget, Option<AddressPoisoningTarget>, Option<WebsiteTarget>) {
        let address = AddressTarget {
            chain: payload.target.asset_id.chain,
            address: payload.target.address.clone(),
        };
        let poisoning = match payload.transaction_type {
            TransactionType::Transfer | TransactionType::TransferNFT => Some(AddressPoisoningTarget {
                target: address.clone(),
                user_address: payload.origin.address.clone(),
            }),
            TransactionType::Swap
            | TransactionType::TokenApproval
            | TransactionType::StakeDelegate
            | TransactionType::StakeUndelegate
            | TransactionType::StakeRewards
            | TransactionType::StakeRedelegate
            | TransactionType::StakeWithdraw
            | TransactionType::StakeFreeze
            | TransactionType::StakeUnfreeze
            | TransactionType::AssetActivation
            | TransactionType::SmartContractCall
            | TransactionType::PerpetualOpenPosition
            | TransactionType::PerpetualClosePosition
            | TransactionType::PerpetualModifyPosition
            | TransactionType::EarnDeposit
            | TransactionType::EarnWithdraw => None,
        };
        let website = payload.website.clone().map(|website| WebsiteTarget { website });
        (address, poisoning, website)
    }

    fn is_scan_complete(enable: bool, has_address_scan: bool, scans: &[bool]) -> bool {
        enable && has_address_scan && scans.iter().all(|is_complete| *is_complete)
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

    pub async fn scan_address_providers(&self, target: AddressTarget) -> Vec<Option<ScanResult<AddressTarget>>> {
        if !self.enable {
            return Vec::new();
        }
        future::join_all(
            self.providers
                .addresses
                .iter()
                .filter(|provider| provider.supports_chain(target.chain))
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

    async fn scan_address_poisoning_providers(&self, target: Option<AddressPoisoningTarget>) -> Vec<Option<ScanResult<AddressPoisoningTarget>>> {
        if !self.enable {
            return Vec::new();
        }
        let Some(target) = target else {
            return Vec::new();
        };
        future::join_all(
            self.providers
                .poisoning
                .iter()
                .filter(|provider| provider.supports_chain(target.target.chain))
                .map(|provider| async { (provider.name(), provider.scan_address_poisoning(&target).await) }),
        )
        .await
        .into_iter()
        .map(|(provider, result)| match result {
            Ok(result) => {
                info_with_fields!(
                    "security scan result",
                    kind = "address_poisoning",
                    provider = result.provider.as_str(),
                    chain = result.target.target.chain.as_ref(),
                    malicious = result.is_malicious,
                    reason = result.reason.as_deref().unwrap_or_default()
                );
                Some(result)
            }
            Err(error) => {
                error_with_fields!(
                    "security scan failed",
                    error.as_ref(),
                    kind = "address_poisoning",
                    provider = provider,
                    chain = target.target.chain.as_ref()
                );
                None
            }
        })
        .collect()
    }

    async fn scan_website_providers(&self, target: Option<WebsiteTarget>) -> Vec<Option<ScanResult<WebsiteTarget>>> {
        if !self.enable {
            return Vec::new();
        }
        let Some(target) = target else {
            return Vec::new();
        };
        future::join_all(
            self.providers
                .websites
                .iter()
                .map(|provider| async { (provider.name(), provider.scan_website(&target).await) }),
        )
        .await
        .into_iter()
        .map(|(provider, result)| match result {
            Ok(result) => {
                info_with_fields!(
                    "security scan result",
                    kind = "website",
                    provider = result.provider.as_str(),
                    malicious = result.is_malicious,
                    reason = result.reason.as_deref().unwrap_or_default()
                );
                Some(result)
            }
            Err(error) => {
                error_with_fields!("security scan failed", error.as_ref(), kind = "website", provider = provider);
                None
            }
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, ScanAddressTarget};

    fn payload(origin: AssetId, target: AssetId, website: Option<&str>, transaction_type: TransactionType) -> ScanTransactionPayload {
        ScanTransactionPayload {
            origin: ScanAddressTarget {
                asset_id: origin,
                address: "origin".into(),
            },
            target: ScanAddressTarget {
                asset_id: target,
                address: "target".into(),
            },
            website: website.map(str::to_string),
            transaction_type,
        }
    }

    #[test]
    fn test_scan_complete_requires_provider_results() {
        assert!(ScanClient::is_scan_complete(true, true, &[true, true]));
        assert!(!ScanClient::is_scan_complete(true, true, &[true, false]));
        assert!(!ScanClient::is_scan_complete(true, false, &[true]));
        assert!(!ScanClient::is_scan_complete(false, true, &[true, true]));
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
        let payload = payload(AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Ethereum), None, TransactionType::Transfer);

        assert!(ScanClient::token_asset_ids(&payload).is_empty());
    }

    #[test]
    fn test_same_token_is_looked_up_once() {
        let token = AssetId::from_token(Chain::SmartChain, "0x123");
        let payload = payload(token.clone(), token, None, TransactionType::Transfer);

        assert_eq!(ScanClient::token_asset_ids(&payload).len(), 1);
    }

    #[test]
    fn test_swap_looks_up_both_distinct_token_assets() {
        let payload = payload(
            AssetId::from_token(Chain::Ethereum, "0x123"),
            AssetId::from_token(Chain::SmartChain, "0x456"),
            None,
            TransactionType::Swap,
        );

        assert_eq!(
            ScanClient::token_asset_ids(&payload),
            vec![AssetId::from_token(Chain::Ethereum, "0x123"), AssetId::from_token(Chain::SmartChain, "0x456"),]
        );
    }

    #[test]
    fn test_provider_targets_use_recipient_context() {
        let payload = payload(
            AssetId::from_chain(Chain::SmartChain),
            AssetId::from_token(Chain::SmartChain, "0x456"),
            Some("https://example.com"),
            TransactionType::Transfer,
        );

        assert_eq!(
            ScanClient::provider_targets(&payload),
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
    fn test_provider_targets_skip_poisoning_for_contract_calls() {
        let payload = payload(
            AssetId::from_chain(Chain::Ethereum),
            AssetId::from_token(Chain::SmartChain, "0x456"),
            None,
            TransactionType::Swap,
        );

        assert_eq!(ScanClient::provider_targets(&payload).1, None);
    }
}
