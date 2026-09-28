use primitives::asset_score::AssetRank;
use primitives::{ChainAddress, ScanTransactionPayload, ScanType, TransactionType};

use super::model::{ScanDetection, ScanFinding, ScanPlan, ScanTargets, TransactionScanInput};
use super::subject::{ScanSubject, scan_subjects};
use crate::{AddressPoisoningTarget, AddressTarget, WebsiteTarget};

pub fn plan_transaction_scan(input: &TransactionScanInput) -> ScanPlan {
    let payload = &input.payload;
    let is_target_verified = input.addresses.iter().any(|address| address.is_verified_for(payload.target.asset_id.chain, &payload.target.address));
    let subjects = scan_subjects(payload);
    let cached = cached_detections(input, &subjects, is_target_verified);
    let mut detections = local_detections(input);
    detections.extend(cached.iter().cloned());

    let has_website = subjects.iter().any(|subject| subject.scan_type == ScanType::Website);
    let is_resolved = detections.iter().any(|detection| detection.is_enforced) || (is_target_verified && !has_website);
    let is_skipped = |scan_type: ScanType| (is_target_verified && is_address_type(scan_type)) || cached.iter().any(|detection| detection.scan_type == scan_type);
    let targets = provider_targets(payload, &subjects).filter(|_| !is_resolved).map(|targets| targets.retain(|scan_type| !is_skipped(scan_type)));
    let safe: Vec<ScanType> = match &targets {
        Some(targets) => ScanType::all().into_iter().filter(|scan_type| targets.contains(*scan_type) && input.safe.contains(scan_type)).collect(),
        None => vec![],
    };

    ScanPlan {
        subjects,
        detections,
        is_memo_required: input.addresses.iter().any(|address| address.is_memo_required == Some(true)),
        targets: targets.map(|targets| targets.retain(|scan_type| !safe.contains(&scan_type))),
        safe,
    }
}

fn is_address_type(scan_type: ScanType) -> bool {
    matches!(scan_type, ScanType::Address | ScanType::AddressPoisoning)
}

fn local_detections(input: &TransactionScanInput) -> Vec<ScanDetection> {
    let addresses = input
        .addresses
        .iter()
        .filter(|address| address.is_malicious == Some(true))
        .map(|address| ScanDetection::local(ScanType::Address, ScanFinding::Address(ChainAddress::new(address.chain, address.address.clone())), address.address.clone(), "manual", true));
    let is_asset_enforced = input.enforced.contains(&ScanType::Asset);
    let assets = input
        .assets
        .iter()
        .filter(|asset| asset.score.rank <= AssetRank::Spam.threshold())
        .map(|asset| ScanDetection::local(ScanType::Asset, ScanFinding::Asset(asset.asset.id.clone()), asset.asset.id.to_string(), "spam", is_asset_enforced));
    addresses.chain(assets).collect()
}

fn cached_detections(input: &TransactionScanInput, subjects: &[ScanSubject], is_target_verified: bool) -> Vec<ScanDetection> {
    subjects
        .iter()
        .filter(|subject| !(is_target_verified && is_address_type(subject.scan_type)))
        .filter_map(|subject| {
            let verdict = input.verdicts.iter().find(|verdict| subject.matches(verdict))?;
            Some(ScanDetection::provider(subject, verdict.provider, verdict.reason.clone(), input.enforced.contains(&subject.scan_type), true))
        })
        .collect()
}

fn provider_targets(payload: &ScanTransactionPayload, subjects: &[ScanSubject]) -> Option<ScanTargets> {
    let has_subject = |scan_type: ScanType| subjects.iter().any(|subject| subject.scan_type == scan_type);
    let address = has_subject(ScanType::Address).then(|| AddressTarget {
        chain: payload.target.asset_id.chain,
        address: payload.target.address.clone(),
    });
    let poisoning = match payload.transaction_type {
        TransactionType::Transfer | TransactionType::TransferNFT => address.clone().map(|target| AddressPoisoningTarget {
            target,
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
    let website = payload.website.clone().filter(|_| has_subject(ScanType::Website)).map(|website| WebsiteTarget { website });
    Some(ScanTargets { address, poisoning, website })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use primitives::asset_score::AssetRank;
    use primitives::{AssetBasic, AssetId, Chain, ScanAddress, ScanProvider, ScanTransactionPayload, ScanVerdict};

    use super::*;
    use crate::{AddressPoisoningTarget, AddressTarget, WebsiteTarget};

    fn payload(transaction_type: TransactionType, website: Option<&str>) -> ScanTransactionPayload {
        ScanTransactionPayload {
            transaction_type,
            website: website.map(str::to_string),
            ..ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::SmartChain), AssetId::from_chain(Chain::SmartChain))
        }
    }

    fn verified(address: &str) -> ScanAddress {
        ScanAddress::contract(Chain::SmartChain, address, "Verified")
    }

    fn verdict(scan_type: ScanType, chain: Option<Chain>, target: &str) -> ScanVerdict {
        ScanVerdict {
            scan_type,
            chain,
            target: target.to_string(),
            provider: ScanProvider::HashDit,
            reason: Some("phishing".to_string()),
        }
    }

    fn spam_asset(asset_id: AssetId) -> AssetBasic {
        let mut asset = AssetBasic::mock_with_price(Chain::SmartChain, 1.0, 0.0);
        asset.asset.id = asset_id;
        asset.score.rank = AssetRank::Spam.threshold();
        asset
    }

    #[test]
    fn test_plan_skips_cached_safe_types() {
        let mut input = TransactionScanInput::mock(payload(TransactionType::Transfer, Some("https://example.com")));
        input.safe = HashSet::from([ScanType::Address]);

        let plan = plan_transaction_scan(&input);
        let targets = plan.targets.unwrap();

        assert_eq!(plan.safe, vec![ScanType::Address]);
        assert_eq!(targets.address, None);
        assert!(targets.poisoning.is_some());
        assert!(targets.website.is_some());
    }

    #[test]
    fn test_plan_verified_target_ignores_cached_safe_address() {
        let mut input = TransactionScanInput::mock(payload(TransactionType::SmartContractCall, Some("https://example.com")));
        input.addresses = vec![verified("target")];
        input.safe = HashSet::from([ScanType::Address]);

        assert!(plan_transaction_scan(&input).safe.is_empty());
    }

    #[test]
    fn test_plan_scans_recipient_targets() {
        let plan = plan_transaction_scan(&TransactionScanInput::mock(payload(TransactionType::Transfer, Some("https://example.com"))));
        let address = AddressTarget {
            chain: Chain::SmartChain,
            address: "target".to_string(),
        };

        assert_eq!(
            plan.targets,
            Some(ScanTargets {
                address: Some(address.clone()),
                poisoning: Some(AddressPoisoningTarget {
                    target: address,
                    user_address: "origin".to_string(),
                }),
                website: Some(WebsiteTarget { website: "https://example.com".to_string() }),
            })
        );
    }

    #[test]
    fn test_plan_skips_poisoning_for_contract_calls() {
        let targets = plan_transaction_scan(&TransactionScanInput::mock(payload(TransactionType::Swap, None))).targets.unwrap();

        assert!(targets.address.is_some());
        assert!(targets.poisoning.is_none());
    }

    #[test]
    fn test_plan_skips_empty_address() {
        let mut payload = payload(TransactionType::SmartContractCall, Some("https://example.com"));
        payload.target.address = String::new();

        let targets = plan_transaction_scan(&TransactionScanInput::mock(payload)).targets.unwrap();

        assert_eq!(targets.address, None);
        assert_eq!(targets.poisoning, None);
        assert!(targets.website.is_some());
    }

    #[test]
    fn test_plan_skips_invalid_website() {
        let targets = plan_transaction_scan(&TransactionScanInput::mock(payload(TransactionType::SmartContractCall, Some("invalid website")))).targets.unwrap();

        assert_eq!(targets.website, None);
        assert!(targets.address.is_some());
    }

    #[test]
    fn test_plan_skips_staking() {
        let plan = plan_transaction_scan(&TransactionScanInput::mock(payload(TransactionType::StakeDelegate, Some("https://example.com"))));

        assert_eq!(plan.targets, None);
    }

    #[test]
    fn test_plan_verified_target_without_website_is_resolved() {
        let mut input = TransactionScanInput::mock(payload(TransactionType::Transfer, None));
        input.addresses = vec![verified("target")];

        assert_eq!(plan_transaction_scan(&input).targets, None);
    }

    #[test]
    fn test_plan_verified_target_still_scans_website() {
        let mut input = TransactionScanInput::mock(payload(TransactionType::SmartContractCall, Some("https://bnbdaily.finance/")));
        input.addresses = vec![verified("target")];

        let targets = plan_transaction_scan(&input).targets.unwrap();

        assert_eq!(targets.address, None);
        assert_eq!(targets.poisoning, None);
        assert_eq!(
            targets.website,
            Some(WebsiteTarget {
                website: "https://bnbdaily.finance/".to_string()
            })
        );
    }

    #[test]
    fn test_plan_flagged_address_is_resolved() {
        let mut flagged = verified("target");
        flagged.is_malicious = Some(true);
        let mut input = TransactionScanInput::mock(payload(TransactionType::Transfer, None));
        input.addresses = vec![flagged];

        let plan = plan_transaction_scan(&input);

        assert_eq!(plan.targets, None);
        assert_eq!(plan.detections[0].reason.as_deref(), Some("manual"));
    }

    #[test]
    fn test_plan_enforced_verdict_is_resolved() {
        let mut input = TransactionScanInput::mock(payload(TransactionType::Transfer, None));
        input.verdicts = vec![verdict(ScanType::Address, Some(Chain::SmartChain), "target")];

        let plan = plan_transaction_scan(&input);

        assert_eq!(plan.targets, None);
        assert!(plan.detections[0].is_cached);
        assert_eq!(plan.detections[0].provider, Some(ScanProvider::HashDit));
        assert_eq!(plan.detections[0].reason.as_deref(), Some("phishing"));
    }

    #[test]
    fn test_plan_ignores_verdict_for_other_chain() {
        let mut input = TransactionScanInput::mock(payload(TransactionType::Transfer, None));
        input.verdicts = vec![verdict(ScanType::Address, Some(Chain::Ethereum), "target")];

        let plan = plan_transaction_scan(&input);

        assert!(plan.detections.is_empty());
        assert!(plan.targets.unwrap().address.is_some());
    }

    #[test]
    fn test_plan_dry_run_verdict_skips_only_its_type() {
        let mut input = TransactionScanInput::mock(payload(TransactionType::Transfer, Some("https://example.com")));
        input.enforced.remove(&ScanType::Website);
        input.verdicts = vec![verdict(ScanType::Website, None, "example.com")];

        let plan = plan_transaction_scan(&input);
        let targets = plan.targets.unwrap();

        assert!(!plan.detections[0].is_enforced);
        assert_eq!(targets.website, None);
        assert!(targets.address.is_some());
    }

    #[test]
    fn test_plan_spam_asset() {
        let token = AssetId::from_token(Chain::SmartChain, "0x123");
        let mut input = TransactionScanInput::mock(ScanTransactionPayload::mock_with_assets(token.clone(), token.clone()));
        input.assets = vec![spam_asset(token)];
        assert_eq!(plan_transaction_scan(&input).targets, None);

        input.enforced.remove(&ScanType::Asset);
        let plan = plan_transaction_scan(&input);
        assert!(!plan.detections[0].is_enforced);
        assert!(plan.targets.is_some());
    }
}
