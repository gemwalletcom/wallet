use primitives::PerpetualAccountMode;
use serde::{Deserialize, Serialize};
use serde_serializers::f64::deserialize_f64_from_str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserAbstractionMode {
    Default,
    Disabled,
    DexAbstraction,
    UnifiedAccount,
    PortfolioMargin,
}

impl From<UserAbstractionMode> for PerpetualAccountMode {
    fn from(value: UserAbstractionMode) -> Self {
        match value {
            UserAbstractionMode::Default | UserAbstractionMode::Disabled | UserAbstractionMode::DexAbstraction => Self::Standard,
            UserAbstractionMode::UnifiedAccount | UserAbstractionMode::PortfolioMargin => Self::Unified,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    pub name: String,
    pub address: String,
    pub valid_until: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "role", rename_all = "camelCase")]
pub enum UserRole {
    Agent {
        data: AgentOwner,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgentOwner {
    pub user: String,
}

impl UserRole {
    pub fn owner(self, signer: &str) -> String {
        match self {
            Self::Agent { data } => data.user,
            Self::Other => signer.to_string(),
        }
    }
}

pub(crate) struct AgentApproval {
    pub(crate) approval_required: bool,
    pub(crate) name: String,
    pub(crate) address: String,
    pub(crate) private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFee {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub user_cross_rate: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub user_spot_cross_rate: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub active_referral_discount: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerUpdate {
    pub time: u64,
    pub hash: String,
    pub delta: LedgerDelta,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorHistoryUpdate {
    pub time: u64,
    pub hash: String,
    pub delta: DelegatorHistoryDelta,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorHistoryDelta {
    pub c_deposit: Option<DelegatorAmountDelta>,
    pub delegate: Option<DelegatorDelegateDelta>,
    pub withdrawal: Option<DelegatorWithdrawalDelta>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorAmountDelta {
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorDelegateDelta {
    pub amount: String,
    pub is_undelegate: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatorWithdrawalDelta {
    pub amount: String,
    pub phase: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LedgerDelta {
    Send {
        nonce: u64,
    },
    SpotTransfer {
        nonce: u64,
    },
    Withdraw {
        nonce: u64,
    },
    CStakingTransfer {
        token: String,
        amount: String,
        is_deposit: bool,
    },
    #[serde(other)]
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_owner() {
        let signer = "0xd864a8c0ba6f7a3008ddf148e1194813ba3842d2";
        for (response, owner) in [
            (r#"{"role":"agent","data":{"user":"0xf5421efce6a6fede2ab38baa02e9e87bf16ef534"}}"#, "0xf5421efce6a6fede2ab38baa02e9e87bf16ef534"),
            (r#"{"role":"user"}"#, signer),
            (r#"{"role":"missing"}"#, signer),
            (r#"{"role":"subAccount","data":{"master":"0xf5421efce6a6fede2ab38baa02e9e87bf16ef534"}}"#, signer),
        ] {
            assert_eq!(serde_json::from_str::<UserRole>(response).unwrap().owner(signer), owner);
        }
    }
}
