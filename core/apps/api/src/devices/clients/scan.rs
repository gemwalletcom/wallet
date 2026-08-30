use cacher::{AccessTokenCacherClient, CacherClient};
use gem_client::ReqwestClient;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::{ScanTransaction, ScanTransactionPayload};
use rocket::futures::future;
use security_provider::providers::goplus::GoPlusProvider;
use security_provider::providers::hashdit::HashDitProvider;
use security_provider::{AddressTarget, ScanProvider, ScanResult, TokenTarget};
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::{Database, ScanAddressesRepository};

pub struct ScanProviderFactory {}

impl ScanProviderFactory {
    pub fn create_providers(settings: &Settings, cacher: CacherClient) -> Vec<Box<dyn ScanProvider + Send + Sync>> {
        let client = gem_client::builder().timeout(settings.security.timeout).build().unwrap();

        vec![
            Box::new(GoPlusProvider::new(
                ReqwestClient::new(settings.security.goplus.url.clone(), client.clone()),
                &settings.security.goplus.key.public,
                &settings.security.goplus.key.secret,
                Some(Arc::new(AccessTokenCacherClient::new(cacher, GoPlusProvider::<ReqwestClient>::NAME))),
            )),
            Box::new(HashDitProvider::new(
                ReqwestClient::new(settings.security.hashdit.url.clone(), client.clone()),
                &settings.security.hashdit.key.public,
                &settings.security.hashdit.key.secret,
            )),
        ]
    }
}

#[derive(Clone)]
pub struct ScanClient {
    database: Database,
    pub security_providers: Vec<Arc<dyn ScanProvider + Send + Sync>>,
}

impl ScanClient {
    pub fn new(database: Database, security_providers: Vec<Box<dyn ScanProvider + Send + Sync>>) -> Self {
        let security_providers = security_providers.into_iter().map(Arc::from).collect();
        Self { database, security_providers }
    }

    pub async fn get_scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let local_scan = self.get_scan_transaction_local(payload.clone())?;
        if local_scan.is_malicious {
            return Ok(local_scan);
        }

        let target = AddressTarget {
            chain: payload.origin.asset_id.chain,
            address: payload.origin.address.clone(),
        };
        let token_targets = Self::token_targets(&payload);
        let (address_scans, token_scans) = future::join(
            self.scan_address_providers(target),
            future::join_all(token_targets.into_iter().map(|target| self.scan_token_providers(target))),
        )
        .await;

        Ok(ScanTransaction {
            is_malicious: address_scans.iter().any(|scan| scan.is_malicious) || token_scans.iter().flatten().any(|scan| scan.is_malicious),
            is_memo_required: local_scan.is_memo_required,
        })
    }

    fn get_scan_transaction_local(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, Box<dyn Error + Send + Sync>> {
        let queries = [
            (payload.origin.asset_id.chain, payload.origin.address.as_str()),
            (payload.target.asset_id.chain, payload.target.address.as_str()),
        ];
        let addresses = self.database.scan_addresses()?.get_scan_addresses(&queries)?;
        let is_malicious = addresses.iter().any(|address| address.is_fraudulent);
        let is_memo_required = addresses.iter().any(|address| address.is_memo_required);

        Ok(ScanTransaction { is_malicious, is_memo_required })
    }

    fn token_targets(payload: &ScanTransactionPayload) -> Vec<TokenTarget> {
        let mut targets = Vec::new();
        for asset_id in [&payload.origin.asset_id, &payload.target.asset_id] {
            let Some(token_id) = &asset_id.token_id else {
                continue;
            };
            let target = TokenTarget {
                token_id: token_id.clone(),
                chain: asset_id.chain,
            };
            if !targets.iter().any(|value: &TokenTarget| value.chain == target.chain && value.token_id == target.token_id) {
                targets.push(target);
            }
        }
        targets
    }

    pub async fn scan_address_providers(&self, target: AddressTarget) -> Vec<ScanResult<AddressTarget>> {
        future::join_all(
            self.security_providers
                .iter()
                .filter(|provider| provider.supports_address_chain(target.chain))
                .map(|provider| async { (provider.name(), provider.scan_address(&target).await) }),
        )
        .await
        .into_iter()
        .filter_map(|(provider, result)| match result {
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

    async fn scan_token_providers(&self, target: TokenTarget) -> Vec<ScanResult<TokenTarget>> {
        future::join_all(
            self.security_providers
                .iter()
                .filter(|provider| provider.supports_token_chain(target.chain))
                .map(|provider| async { (provider.name(), provider.scan_token(&target).await) }),
        )
        .await
        .into_iter()
        .filter_map(|(provider, result)| match result {
            Ok(result) => {
                info_with_fields!(
                    "security scan result",
                    kind = "token",
                    provider = result.provider.as_str(),
                    chain = result.target.chain.as_ref(),
                    token_id = result.target.token_id.as_str(),
                    malicious = result.is_malicious,
                    reason = result.reason.as_deref().unwrap_or_default()
                );
                Some(result)
            }
            Err(error) => {
                error_with_fields!(
                    "security scan failed",
                    error.as_ref(),
                    kind = "token",
                    provider = provider,
                    chain = target.chain.as_ref(),
                    token_id = target.token_id.as_str()
                );
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
    fn test_native_assets_do_not_create_token_targets() {
        let payload = payload(AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Ethereum), TransactionType::Transfer);

        assert!(ScanClient::token_targets(&payload).is_empty());
    }

    #[test]
    fn test_same_token_from_and_to_is_scanned_once() {
        let token = AssetId::from_token(Chain::SmartChain, "0x123");
        let payload = payload(token.clone(), token, TransactionType::Transfer);

        assert_eq!(ScanClient::token_targets(&payload).len(), 1);
    }

    #[test]
    fn test_swap_scans_both_distinct_token_assets() {
        let payload = payload(
            AssetId::from_token(Chain::Ethereum, "0x123"),
            AssetId::from_token(Chain::SmartChain, "0x456"),
            TransactionType::Swap,
        );

        assert_eq!(
            ScanClient::token_targets(&payload),
            vec![
                TokenTarget {
                    token_id: "0x123".into(),
                    chain: Chain::Ethereum,
                },
                TokenTarget {
                    token_id: "0x456".into(),
                    chain: Chain::SmartChain,
                },
            ]
        );
    }
}
