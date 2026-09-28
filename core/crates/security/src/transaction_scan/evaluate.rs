use primitives::{ScanOutcome, ScanSource, ScanTransaction, ScanType};

use super::check::ProviderCheck;
use super::model::{ScanDetection, ScanFinding, ScanPlan, TransactionScanInput};
use super::result::TransactionScanResult;

pub fn evaluate_transaction_scan(input: &TransactionScanInput, plan: ScanPlan, checks: Vec<ProviderCheck>) -> TransactionScanResult {
    let mut detections = plan.detections;
    let mut new_verdicts = Vec::new();
    for subject in &plan.subjects {
        let Some(check) = checks.iter().find(|check| check.scan_type == subject.scan_type && check.outcome == ScanOutcome::Malicious) else {
            continue;
        };
        let is_enforced = input.enforced.contains(&subject.scan_type);
        if is_enforced {
            new_verdicts.push(subject.verdict(check.provider, check.reason.clone()));
        }
        detections.push(ScanDetection::provider(subject, check.provider, check.reason.clone(), is_enforced, false));
    }

    let is_scan_complete = plan.targets.is_none() || {
        let completed = checks.iter().filter(|check| input.enforced.contains(&check.scan_type) && check.outcome != ScanOutcome::Error).count();
        let cached = plan.safe.iter().filter(|scan_type| input.enforced.contains(scan_type)).count();
        completed + cached >= input.required_successes
    };
    let new_safe = ScanType::all().into_iter().filter(|scan_type| scan_type.is_safe_cacheable() && is_clean(&checks, *scan_type)).collect();

    TransactionScanResult {
        scan: scan_transaction(&detections, plan.is_memo_required, is_scan_complete),
        source: if checks.is_empty() { ScanSource::Local } else { ScanSource::Remote },
        subjects: plan.subjects,
        detections,
        new_verdicts,
        safe: plan.safe,
        new_safe,
        checks,
    }
}

fn is_clean(checks: &[ProviderCheck], scan_type: ScanType) -> bool {
    let mut checks = checks.iter().filter(|check| check.scan_type == scan_type).peekable();
    checks.peek().is_some() && checks.all(|check| check.outcome == ScanOutcome::Clean)
}

fn scan_transaction(detections: &[ScanDetection], is_memo_required: bool, is_scan_complete: bool) -> ScanTransaction {
    let mut malicious_addresses = Vec::new();
    let mut malicious_assets = Vec::new();
    let mut malicious_website = None;
    for detection in detections.iter().filter(|detection| detection.is_enforced) {
        match &detection.finding {
            ScanFinding::Address(address) if !malicious_addresses.contains(address) => malicious_addresses.push(address.clone()),
            ScanFinding::Asset(asset_id) if !malicious_assets.contains(asset_id) => malicious_assets.push(asset_id.clone()),
            ScanFinding::Website(website) => malicious_website = Some(website.clone()),
            ScanFinding::Address(_) | ScanFinding::Asset(_) => {}
        }
    }
    ScanTransaction {
        is_malicious: Some(!malicious_addresses.is_empty() || !malicious_assets.is_empty() || malicious_website.is_some()),
        is_memo_required: Some(is_memo_required),
        is_scan_complete,
        malicious_addresses: Some(malicious_addresses),
        malicious_assets: Some(malicious_assets),
        malicious_website,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use primitives::{AssetId, Chain, ChainAddress, ScanProvider, ScanTransactionPayload, ScanVerdict, TransactionType};

    use super::*;
    use crate::transaction_scan::plan_transaction_scan;

    fn input(transaction_type: TransactionType, website: Option<&str>) -> TransactionScanInput {
        TransactionScanInput::mock(ScanTransactionPayload {
            transaction_type,
            website: website.map(str::to_string),
            ..ScanTransactionPayload::mock_with_assets(AssetId::from_chain(Chain::SmartChain), AssetId::from_chain(Chain::SmartChain))
        })
    }

    fn evaluate(input: &TransactionScanInput, checks: Vec<ProviderCheck>) -> TransactionScanResult {
        evaluate_transaction_scan(input, plan_transaction_scan(input), checks)
    }

    #[test]
    fn test_enforced_malicious_address_is_blocked_and_stored() {
        let input = input(TransactionType::Transfer, None);

        let result = evaluate(
            &input,
            vec![
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Malicious),
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::AddressPoisoning, ScanOutcome::Clean),
            ],
        );

        assert_eq!(result.scan.is_malicious, Some(true));
        assert!(result.scan.is_scan_complete);
        assert_eq!(result.scan.malicious_addresses, Some(vec![ChainAddress::new(Chain::SmartChain, "target".to_string())]));
        assert_eq!(result.source, ScanSource::Remote);
        assert_eq!(
            result.new_verdicts,
            vec![ScanVerdict {
                scan_type: ScanType::Address,
                chain: Some(Chain::SmartChain),
                target: "target".to_string(),
                provider: ScanProvider::HashDit,
                reason: Some("phishing".to_string()),
            }]
        );
        assert_eq!(result.detections[0].target, "target");
        assert_eq!(result.detections[0].reason.as_deref(), Some("phishing"));
        assert!(!result.detections[0].is_cached);
    }

    #[test]
    fn test_malicious_website_returns_url_and_stores_host() {
        let input = input(TransactionType::SmartContractCall, Some("https://bnbdaily.finance/path"));

        let result = evaluate(&input, vec![ProviderCheck::mock(ScanProvider::HashDit, ScanType::Website, ScanOutcome::Malicious)]);

        assert_eq!(result.scan.malicious_website.as_deref(), Some("https://bnbdaily.finance/path"));
        assert_eq!(result.new_verdicts[0].chain, None);
        assert_eq!(result.new_verdicts[0].target, "bnbdaily.finance");
    }

    #[test]
    fn test_dry_run_finding_is_logged_but_not_returned_or_stored() {
        let mut input = input(TransactionType::SmartContractCall, Some("https://example.com"));
        input.enforced.remove(&ScanType::Website);

        let result = evaluate(
            &input,
            vec![
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Clean),
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Website, ScanOutcome::Malicious),
            ],
        );

        assert_eq!(result.scan.is_malicious, Some(false));
        assert_eq!(result.scan.malicious_website, None);
        assert!(result.new_verdicts.is_empty());
        assert_eq!(
            result.detections.iter().filter(|detection| !detection.is_enforced).map(|detection| detection.scan_type).collect::<Vec<_>>(),
            vec![ScanType::Website]
        );
    }

    #[test]
    fn test_scan_complete_counts_enforced_successes() {
        let mut input = input(TransactionType::Transfer, Some("https://example.com"));
        input.required_successes = 2;
        input.enforced.remove(&ScanType::Website);
        let checks = || {
            vec![
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Clean),
                ProviderCheck::mock(ScanProvider::GoPlus, ScanType::Address, ScanOutcome::Error),
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Website, ScanOutcome::Clean),
            ]
        };
        assert!(!evaluate(&input, checks()).scan.is_scan_complete);

        input.enforced.insert(ScanType::Website);
        assert!(evaluate(&input, checks()).scan.is_scan_complete);
    }

    #[test]
    fn test_local_result_is_complete() {
        let input = input(TransactionType::StakeDelegate, None);

        let result = evaluate(&input, vec![]);

        assert!(result.scan.is_scan_complete);
        assert_eq!(result.scan.is_malicious, Some(false));
        assert_eq!(result.source, ScanSource::Local);
    }

    #[test]
    fn test_new_safe_requires_every_check_clean() {
        let input = input(TransactionType::SmartContractCall, Some("https://example.com"));

        let result = evaluate(
            &input,
            vec![
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Clean),
                ProviderCheck::mock(ScanProvider::GoPlus, ScanType::Address, ScanOutcome::Error),
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Website, ScanOutcome::Clean),
            ],
        );

        assert_eq!(result.new_safe, vec![ScanType::Website]);
    }

    #[test]
    fn test_malicious_type_is_not_safe() {
        let input = input(TransactionType::SmartContractCall, None);

        let result = evaluate(
            &input,
            vec![
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Clean),
                ProviderCheck::mock(ScanProvider::GoPlus, ScanType::Address, ScanOutcome::Malicious),
            ],
        );

        assert!(result.new_safe.is_empty());
    }

    #[test]
    fn test_cached_safe_counts_as_completed() {
        let mut input = input(TransactionType::Transfer, None);
        input.safe = HashSet::from([ScanType::Address]);

        let result = evaluate(&input, vec![ProviderCheck::mock(ScanProvider::HashDit, ScanType::AddressPoisoning, ScanOutcome::Error)]);

        assert!(result.scan.is_scan_complete);
        assert_eq!(result.safe, vec![ScanType::Address]);
    }

    #[test]
    fn test_pending_check_is_complete_but_not_safe() {
        let input = input(TransactionType::SmartContractCall, None);

        let result = evaluate(&input, vec![ProviderCheck::mock(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Pending)]);

        assert!(result.scan.is_scan_complete);
        assert!(result.new_safe.is_empty());
        assert!(result.checks.iter().all(|check| check.error.is_none()));
    }

    #[test]
    fn test_provider_error_target() {
        let input = input(TransactionType::Transfer, None);

        let result = evaluate(
            &input,
            vec![
                ProviderCheck::mock(ScanProvider::Tronscan, ScanType::Address, ScanOutcome::Error),
                ProviderCheck::mock(ScanProvider::HashDit, ScanType::AddressPoisoning, ScanOutcome::Clean),
            ],
        );

        assert_eq!(result.subject_target(ScanType::Address), "target");
        assert_eq!(result.checks[0].error.as_deref(), Some("timeout"));
    }
}
