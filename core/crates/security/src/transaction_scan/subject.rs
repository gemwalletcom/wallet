use chain_primitives::checksum_address;
use primitives::{AssetId, Chain, ChainAddress, ScanProvider, ScanTransactionPayload, ScanType, ScanVerdict};
use url::Url;

use super::model::ScanFinding;

#[derive(Debug, Clone, PartialEq)]
pub struct ScanSubject {
    pub scan_type: ScanType,
    pub chain: Option<Chain>,
    pub target: String,
    pub finding: ScanFinding,
}

impl ScanSubject {
    pub fn cache_key(&self) -> String {
        match self.chain {
            Some(chain) => format!("{}:{}", chain.as_ref(), self.target),
            None => self.target.clone(),
        }
    }

    pub fn matches(&self, verdict: &ScanVerdict) -> bool {
        verdict.matches(self.scan_type, self.chain, &self.target)
    }

    pub fn verdict(&self, provider: ScanProvider, reason: Option<String>) -> ScanVerdict {
        ScanVerdict {
            scan_type: self.scan_type,
            chain: self.chain,
            target: self.target.clone(),
            provider,
            reason,
        }
    }
}

pub fn scan_subjects(payload: &ScanTransactionPayload) -> Vec<ScanSubject> {
    let chain = payload.target.asset_id.chain;
    let address = &payload.target.address;
    let target = checksum_address(address, chain);
    let addresses = [ScanType::Address, ScanType::AddressPoisoning].into_iter().filter(|_| !address.is_empty()).map(|scan_type| ScanSubject {
        scan_type,
        chain: Some(chain),
        target: target.clone(),
        finding: ScanFinding::Address(ChainAddress::new(chain, address.clone())),
    });
    let website = payload.website.clone().zip(website_host(payload)).map(|(website, host)| ScanSubject {
        scan_type: ScanType::Website,
        chain: None,
        target: host,
        finding: ScanFinding::Website(website),
    });
    addresses.chain(website).collect()
}

pub fn website_host(payload: &ScanTransactionPayload) -> Option<String> {
    Url::parse(payload.website.as_deref()?).ok()?.host_str().map(str::to_string)
}

pub fn token_asset_ids(payload: &ScanTransactionPayload) -> Vec<AssetId> {
    let mut targets = Vec::new();
    for asset_id in [&payload.origin.asset_id, &payload.target.asset_id] {
        if !asset_id.is_native() && !targets.contains(asset_id) {
            targets.push(asset_id.clone());
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use primitives::TransactionType;

    use super::*;

    fn payload(website: Option<&str>) -> ScanTransactionPayload {
        ScanTransactionPayload {
            website: website.map(str::to_string),
            ..ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::SmartChain), AssetId::from_chain(Chain::SmartChain))
        }
    }

    #[test]
    fn test_website_host_excludes_credentials_and_query_values() {
        assert_eq!(website_host(&payload(Some("https://user:password@example.com/path?token=secret#fragment"))), Some("example.com".into()));
        assert_eq!(website_host(&payload(Some("invalid website"))), None);
    }

    #[test]
    fn test_token_asset_ids() {
        let token = AssetId::from_token(Chain::SmartChain, "0x123");
        let other = AssetId::from_token(Chain::Ethereum, "0x456");

        assert!(token_asset_ids(&ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Ethereum))).is_empty());
        assert_eq!(token_asset_ids(&ScanTransactionPayload::mock_with_assets(token.clone(), token.clone())), vec![token.clone()]);
        assert_eq!(token_asset_ids(&ScanTransactionPayload::mock_with_assets(token.clone(), other.clone())), vec![token, other]);
    }

    #[test]
    fn test_scan_subjects() {
        let subjects = scan_subjects(&payload(Some("https://example.com/path")));

        assert_eq!(subjects.iter().map(|subject| subject.scan_type).collect::<Vec<_>>(), vec![ScanType::Address, ScanType::AddressPoisoning, ScanType::Website]);
        assert_eq!(subjects[0].cache_key(), "smartchain:target");
        assert_eq!(subjects[2].cache_key(), "example.com");
        assert_eq!(subjects[2].finding, ScanFinding::Website("https://example.com/path".to_string()));
    }

    #[test]
    fn test_scan_subjects_checksum_evm_targets() {
        let subject = |address: &str| {
            let mut payload = payload(None);
            payload.target.address = address.to_string();
            scan_subjects(&payload).remove(0)
        };
        let lower = subject("0x938915fd4b7c188a211113ed655ae1f18c334146");

        assert_eq!(lower.target, "0x938915Fd4b7C188A211113Ed655ae1F18c334146");
        assert_eq!(subject("0x938915FD4B7C188A211113ED655AE1F18C334146").target, lower.target);
        assert_eq!(lower.finding, ScanFinding::Address(ChainAddress::new(Chain::SmartChain, "0x938915fd4b7c188a211113ed655ae1f18c334146".to_string())));
    }

    #[test]
    fn test_scan_subjects_keep_non_evm_targets() {
        let mut payload = ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::Solana), AssetId::from_chain(Chain::Solana));
        payload.target.address = "So11111111111111111111111111111111111111112".to_string();

        assert_eq!(scan_subjects(&payload)[0].target, "So11111111111111111111111111111111111111112");
    }

    #[test]
    fn test_scan_subjects_skip_empty_address() {
        let mut payload = ScanTransactionPayload {
            transaction_type: TransactionType::SmartContractCall,
            ..payload(None)
        };
        payload.target.address = String::new();

        assert!(scan_subjects(&payload).is_empty());
    }

    #[test]
    fn test_subject_matches_verdict() {
        let subject = &scan_subjects(&payload(None))[0];

        assert!(subject.matches(&subject.verdict(ScanProvider::HashDit, None)));
        assert!(!subject.matches(&scan_subjects(&payload(None))[1].verdict(ScanProvider::HashDit, None)));
    }
}
