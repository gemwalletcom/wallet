use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SecurityRequest {
    pub(super) chain_id: &'static str,
    pub(super) address: String,
    pub(super) sync: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SolanaTokenSecurityRequest {
    pub(super) address: String,
    pub(super) sync: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AddressPoisoningRequest {
    pub(super) chain_id: &'static str,
    pub(super) address: String,
    pub(super) user_address: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DomainSecurityRequest {
    pub(super) url: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status")]
pub(super) enum SecurityResponse {
    #[serde(rename = "ok")]
    Complete { data: SecurityData },
    #[serde(rename = "in progress")]
    InProgress {
        #[serde(rename = "pollAfter")]
        poll_after: u64,
    },
}

impl SecurityResponse {
    pub(super) fn into_data(self) -> Result<SecurityData, String> {
        match self {
            Self::Complete { data } => Ok(data),
            Self::InProgress { poll_after } => Err(format!("HashDit scan is in progress; retry after {poll_after} seconds")),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct SecurityData {
    pub(super) overall_risk_level: RiskLevel,
}

#[derive(Debug, Deserialize)]
pub(super) struct AddressPoisoningResponse {
    pub(super) data: AddressPoisoningData,
}

#[derive(Debug, Deserialize)]
pub(super) struct AddressPoisoningData {
    pub(super) target_address: AddressPoisoningResult,
}

#[derive(Debug, Deserialize)]
pub(super) struct AddressPoisoningResult {
    pub(super) is_poisoning: BinaryFlag,
}

#[derive(Debug, Deserialize)]
pub(super) struct DomainSecurityResponse {
    pub(super) data: DomainSecurityData,
}

#[derive(Debug, Deserialize)]
pub(super) struct DomainSecurityData {
    pub(super) has_result: bool,
    pub(super) risk_level: i32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) enum BinaryFlag {
    #[serde(rename = "0")]
    Clear,
    #[serde(rename = "1")]
    Risk,
}

impl BinaryFlag {
    pub(super) fn is_risk(&self) -> bool {
        match self {
            Self::Clear => false,
            Self::Risk => true,
        }
    }
}

impl DomainSecurityData {
    pub(super) fn is_malicious(&self) -> Result<bool, String> {
        if !self.has_result {
            return Err("HashDit domain scan has no result".to_string());
        }
        match self.risk_level {
            -1..=1 => Ok(false),
            2..=5 => Ok(true),
            value => Err(format!("Unsupported HashDit domain risk level: {value}")),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub(super) enum RiskLevel {
    #[serde(rename = "No Obvious Risk")]
    NoObvious,
    #[serde(rename = "Low Risk")]
    Low,
    #[serde(rename = "Medium Risk")]
    Medium,
    #[serde(rename = "High Risk")]
    High,
    #[serde(rename = "Significant Risk")]
    Significant,
}

impl RiskLevel {
    pub(super) fn is_malicious(&self) -> bool {
        matches!(self, Self::Medium | Self::High | Self::Significant)
    }

    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::NoObvious => "No Obvious Risk",
            Self::Low => "Low Risk",
            Self::Medium => "Medium Risk",
            Self::High => "High Risk",
            Self::Significant => "Significant Risk",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_levels() {
        assert!(!RiskLevel::NoObvious.is_malicious());
        assert!(!RiskLevel::Low.is_malicious());
        assert!(RiskLevel::Medium.is_malicious());
        assert!(RiskLevel::High.is_malicious());
        assert!(RiskLevel::Significant.is_malicious());
    }

    #[test]
    fn test_domain_risk_levels() {
        for risk_level in -1..=1 {
            assert_eq!(DomainSecurityData { has_result: true, risk_level }.is_malicious(), Ok(false));
        }
        for risk_level in 2..=5 {
            assert_eq!(DomainSecurityData { has_result: true, risk_level }.is_malicious(), Ok(true));
        }
        assert!(DomainSecurityData { has_result: false, risk_level: 0 }.is_malicious().is_err());
        assert!(DomainSecurityData { has_result: false, risk_level: 5 }.is_malicious().is_err());
        assert!(DomainSecurityData { has_result: true, risk_level: 6 }.is_malicious().is_err());
    }
}
