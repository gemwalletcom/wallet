use std::error::Error;
use std::sync::Arc;

use config_keys::ConfigKey;
use gem_tracing::error_with_fields;
use primitives::{AddressDetails, AddressDetailsBalances, AddressType, Asset, AssetId, ChainAddress, ScanAddress, ScanType, ScanVerdict, VerificationStatus};
use storage::{AssetsRepository, Database, DatabaseError, ScanAddressesRepository, ScanDetectionsRepository};

use crate::ConfigCacher;
use crate::chain::ChainClient;

pub struct AddressDetailsClient {
    database: Database,
    config: Arc<ConfigCacher>,
    chain: ChainClient,
}

impl AddressDetailsClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>, chain: ChainClient) -> Self {
        Self { database, config, chain }
    }

    pub async fn get_address_details(&self, request: ChainAddress) -> Result<AddressDetails, Box<dyn Error + Send + Sync>> {
        let detection_max_age = self.config.get_duration(ConfigKey::ScanDetectionMaxAge).await?;
        let query = request.clone();
        let (assets, scan_addresses, verdicts) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let assets = client.get_assets(vec![AssetId::from(query.chain, Some(query.address.clone()))])?;
                let scan_addresses = client.get_scan_addresses(&[(query.chain, query.address.as_str())])?;
                let verdicts = client.get_scan_detections(vec![query.address.clone()], detection_max_age)?;
                Ok((assets, scan_addresses, verdicts))
            })
            .await?;
        let details = map_address_details(request, assets.into_iter().next(), scan_addresses.into_iter().next(), &verdicts);
        let balances = match details.address_type {
            AddressType::Validator => None,
            AddressType::Address | AddressType::Contract | AddressType::Asset | AddressType::Contact | AddressType::InternalWallet => self.get_balances(&details).await,
        };
        Ok(AddressDetails { balances, ..details })
    }

    async fn get_balances(&self, details: &AddressDetails) -> Option<AddressDetailsBalances> {
        let request = ChainAddress::new(details.chain, details.address.clone());
        match futures::try_join!(self.chain.get_balances_coin(request.clone()), self.chain.get_balances_staking(request)) {
            Ok((coin, staking)) => Some(AddressDetailsBalances { coin, staking }),
            Err(error) => {
                error_with_fields!("address details balances failed", &*error, chain = details.chain.as_ref());
                None
            }
        }
    }
}

fn map_address_details(request: ChainAddress, asset: Option<Asset>, scan_address: Option<ScanAddress>, verdicts: &[ScanVerdict]) -> AddressDetails {
    let is_flagged = verdicts.iter().any(|verdict| verdict.matches(ScanType::Address, Some(request.chain), &request.address));
    let scan_status = scan_address.as_ref().map(ScanAddress::verification_status);
    let status = match (is_flagged, scan_status, &asset) {
        (true, _, _) | (false, Some(VerificationStatus::Suspicious), _) => VerificationStatus::Suspicious,
        (false, _, Some(_)) => VerificationStatus::Verified,
        (false, Some(status), None) => status,
        (false, None, None) => VerificationStatus::Unverified,
    };
    let (name, address_type) = match (asset, scan_address) {
        (Some(asset), _) => (Some(asset.name), AddressType::Asset),
        (None, Some(scan_address)) => (scan_address.name, scan_address.address_type.unwrap_or(AddressType::Address)),
        (None, None) => (None, AddressType::Address),
    };
    AddressDetails {
        chain: request.chain,
        address: request.address,
        name,
        address_type,
        status,
        balances: None,
    }
}

#[cfg(test)]
mod tests {
    use std::slice;

    use super::map_address_details;
    use primitives::{AddressDetails, AddressType, Asset, Chain, ChainAddress, ScanAddress, ScanProvider, ScanType, ScanVerdict, VerificationStatus};

    #[test]
    fn test_map_address_details() {
        let request = ChainAddress::new(Chain::Ethereum, "0x1".to_string());
        let verdict = ScanVerdict {
            scan_type: ScanType::Address,
            chain: Some(Chain::Ethereum),
            target: "0x1".to_string(),
            provider: ScanProvider::GoPlus,
            reason: None,
        };
        let asset = Asset {
            name: "Tether USD".to_string(),
            ..Asset::mock_erc20()
        };
        let router = ScanAddress::contract(Chain::Ethereum, "0x1", "Uniswap");
        let validator = ScanAddress {
            address_type: Some(AddressType::Validator),
            ..ScanAddress::contract(Chain::Ethereum, "0x1", "Stakin")
        };
        let fraudulent = ScanAddress {
            name: None,
            address_type: None,
            is_malicious: Some(true),
            is_verified: Some(false),
            ..ScanAddress::contract(Chain::Ethereum, "0x1", "")
        };
        let expected = |name: Option<&str>, address_type: AddressType, status: VerificationStatus| AddressDetails {
            name: name.map(str::to_string),
            address_type,
            status,
            balances: None,
            ..AddressDetails::mock()
        };

        assert_eq!(map_address_details(request.clone(), None, None, &[]), expected(None, AddressType::Address, VerificationStatus::Unverified));
        assert_eq!(
            map_address_details(request.clone(), Some(asset.clone()), Some(router.clone()), &[]),
            expected(Some("Tether USD"), AddressType::Asset, VerificationStatus::Verified)
        );
        assert_eq!(map_address_details(request.clone(), None, Some(router), &[]), expected(Some("Uniswap"), AddressType::Contract, VerificationStatus::Verified));
        assert_eq!(map_address_details(request.clone(), None, Some(validator), &[]), expected(Some("Stakin"), AddressType::Validator, VerificationStatus::Verified));
        assert_eq!(map_address_details(request.clone(), None, Some(fraudulent.clone()), &[]), expected(None, AddressType::Address, VerificationStatus::Suspicious));
        assert_eq!(
            map_address_details(request.clone(), Some(asset.clone()), Some(fraudulent), &[]),
            expected(Some("Tether USD"), AddressType::Asset, VerificationStatus::Suspicious)
        );
        assert_eq!(
            map_address_details(request.clone(), Some(asset), None, slice::from_ref(&verdict)),
            expected(Some("Tether USD"), AddressType::Asset, VerificationStatus::Suspicious)
        );
        assert_eq!(
            map_address_details(request, None, None, &[ScanVerdict { chain: Some(Chain::SmartChain), ..verdict }]),
            expected(None, AddressType::Address, VerificationStatus::Unverified)
        );
    }
}
