use primitives::{ScanSource, ScanTransaction, ScanType, ScanVerdict};

use super::check::ProviderCheck;
use super::model::ScanDetection;
use super::subject::ScanSubject;

pub struct TransactionScanResult {
    pub scan: ScanTransaction,
    pub source: ScanSource,
    pub subjects: Vec<ScanSubject>,
    pub detections: Vec<ScanDetection>,
    pub new_verdicts: Vec<ScanVerdict>,
    pub safe: Vec<ScanType>,
    pub new_safe: Vec<ScanType>,
    pub checks: Vec<ProviderCheck>,
}

impl TransactionScanResult {
    pub fn subject_target(&self, scan_type: ScanType) -> &str {
        self.subjects.iter().find(|subject| subject.scan_type == scan_type).map(|subject| subject.target.as_str()).unwrap_or_default()
    }
}
