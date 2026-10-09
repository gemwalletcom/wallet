use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use primitives::{AddressName, AddressType, Asset, AssetId, ChainAddress, VerificationStatus};

use super::repository::Repository;

#[derive(Clone)]
pub struct AddressNamesClient {
    repository: Arc<dyn Repository>,
}

impl AddressNamesClient {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn get_address_names(&self, requests: Vec<ChainAddress>) -> Result<Vec<AddressName>, Box<dyn Error + Send + Sync>> {
        let requests: Vec<ChainAddress> = requests.into_iter().filter(|request| !request.address.is_empty()).collect();
        if requests.is_empty() {
            return Ok(vec![]);
        }

        let asset_ids = requests.iter().map(|request| AssetId::from(request.chain, Some(request.address.clone()))).collect::<Vec<_>>();
        let (scan_rows, assets) = self.repository.get_address_name_records(requests.clone(), asset_ids).await?;
        let scan_names = scan_rows
            .into_iter()
            .filter_map(|scan_address| scan_address.address_name())
            .map(|name| (ChainAddress::new(name.chain, name.address.clone()), name))
            .collect::<HashMap<_, _>>();
        let asset_names = assets.into_iter().filter_map(asset_entry).collect::<HashMap<_, _>>();

        Ok(map_requests(requests, &scan_names, &asset_names))
    }
}

fn map_requests(requests: Vec<ChainAddress>, scan_names: &HashMap<ChainAddress, AddressName>, asset_names: &HashMap<ChainAddress, AddressName>) -> Vec<AddressName> {
    requests
        .into_iter()
        .filter_map(|request| asset_names.get(&request).or_else(|| scan_names.get(&request)).cloned())
        .scan(HashSet::new(), |seen, name| seen.insert(ChainAddress::new(name.chain, name.address.clone())).then_some(name))
        .collect()
}

fn asset_entry(asset: Asset) -> Option<(ChainAddress, AddressName)> {
    let address = asset.token_id()?.to_string();

    Some((
        ChainAddress::new(asset.chain(), address.clone()),
        AddressName {
            chain: asset.chain(),
            address,
            name: asset.name,
            address_type: AddressType::Contract,
            status: VerificationStatus::Verified,
            image_url: None,
        },
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::map_requests;
    use primitives::{AddressName, AddressType, Chain, ChainAddress, VerificationStatus};

    #[test]
    fn test_map_requests_prefers_asset_then_scan() {
        let asset_request = ChainAddress::new(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string());
        let scan_request = ChainAddress::new(Chain::Ethereum, "0x123".to_string());
        let asset_name = AddressName::mock("0xdAC17F958D2ee523a2206206994597C13D831ec7", "USDT", AddressType::Contract, VerificationStatus::Verified);
        let scan_name = AddressName::mock("0x123", "Scan Name", AddressType::Address, VerificationStatus::Unverified);

        let scan_names = HashMap::from([(asset_request.clone(), scan_name.clone()), (scan_request.clone(), scan_name.clone())]);
        let asset_names = HashMap::from([(asset_request.clone(), asset_name.clone())]);

        assert_eq!(map_requests(vec![asset_request, scan_request], &scan_names, &asset_names), vec![asset_name, scan_name]);
    }
}
