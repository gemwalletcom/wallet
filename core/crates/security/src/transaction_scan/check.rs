use std::error::Error;
use std::time::Duration;

use gem_tracing::DurationMs;
use primitives::{ScanOutcome, ScanProvider, ScanType};
use serde::Serialize;

use crate::{ScanPendingError, ScanResult};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderCheck {
    #[serde(skip)]
    pub provider: ScanProvider,
    #[serde(skip)]
    pub scan_type: ScanType,
    pub outcome: ScanOutcome,
    pub latency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ProviderCheck {
    pub fn new<T>(provider: ScanProvider, scan_type: ScanType, result: Result<ScanResult<T>, Box<dyn Error + Send + Sync>>, latency: Duration) -> Self {
        let (outcome, reason, error) = match result {
            Ok(result) if result.is_malicious => (ScanOutcome::Malicious, result.reason, None),
            Ok(result) => (ScanOutcome::Clean, result.reason, None),
            Err(error) if error.is::<ScanPendingError>() => (ScanOutcome::Pending, Some(error.to_string()), None),
            Err(error) => (ScanOutcome::Error, None, Some(error.to_string())),
        };
        Self {
            provider,
            scan_type,
            outcome,
            latency: DurationMs(latency).to_string(),
            reason,
            error,
        }
    }
}

#[cfg(test)]
impl ProviderCheck {
    pub fn mock(provider: ScanProvider, scan_type: ScanType, outcome: ScanOutcome) -> Self {
        let result = match outcome {
            ScanOutcome::Clean | ScanOutcome::Malicious => Ok(ScanResult {
                target: (),
                is_malicious: outcome == ScanOutcome::Malicious,
                reason: (outcome == ScanOutcome::Malicious).then(|| "phishing".to_string()),
                provider: provider.as_ref().to_string(),
            }),
            ScanOutcome::Pending => Err(ScanPendingError {
                provider: provider.as_ref().to_string(),
                poll_after: 10,
            }
            .into()),
            ScanOutcome::Error => Err("timeout".into()),
        };
        Self::new(provider, scan_type, result, Duration::from_millis(100))
    }
}
