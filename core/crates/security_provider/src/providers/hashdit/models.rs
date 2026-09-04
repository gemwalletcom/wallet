use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SecurityRequest {
    pub(super) chain_id: &'static str,
    pub(super) address: String,
    pub(super) sync: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SecurityResponse {
    pub(super) data: SecurityData,
}

#[derive(Debug, Deserialize)]
pub(super) struct SecurityData {
    pub(super) overall_risk_level: RiskLevel,
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
}
