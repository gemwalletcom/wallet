use primitives::AssetId;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub sequence: u64,
    pub balances: Vec<Balance>,
    pub thresholds: Thresholds,
    pub signers: Vec<Signer>,
}

impl Account {
    pub fn has_trustline(&self, asset_id: &AssetId) -> bool {
        let Some((issuer, code)) = asset_id.split_sub_token_parts() else {
            return false;
        };
        self.balances
            .iter()
            .any(|balance| balance.asset_issuer.as_deref() == Some(issuer.as_str()) && balance.asset_code.as_deref() == Some(code.as_str()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub med_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signer {
    pub key: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEmpty {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    pub asset_code: Option<String>,
    pub asset_issuer: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, asset_constants::STELLAR_USDC_ASSET_ID};

    #[test]
    fn test_has_trustline() {
        let account: Account = serde_json::from_str(
            r#"{"sequence":"1","balances":[{"balance":"1.0","asset_type":"credit_alphanum4","asset_code":"USDC","asset_issuer":"GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN"},{"balance":"2.0","asset_type":"native"}],"thresholds":{"med_threshold":0},"signers":[]}"#,
        )
        .unwrap();

        assert!(account.has_trustline(&STELLAR_USDC_ASSET_ID));
        assert!(!account.has_trustline(&AssetId::from_token(Chain::Stellar, "GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP2::USDC")));
        assert!(!account.has_trustline(&AssetId::from_chain(Chain::Stellar)));
    }
}
