use gem_client::Target;

#[derive(Clone, Debug)]
pub enum PolkadotTarget {
    GetBalance { address: String },
    GetTransactionMaterial,
    EstimateFee,
    GetNodeVersion,
    GetBlockHead,
    GetBlocks { from: String, to: String },
    GetBlockHeader { block: String },
    GetBlock { number: i64 },
    SendTransaction,
}

impl Target for PolkadotTarget {
    fn path(&self) -> String {
        match self {
            Self::GetBalance { address } => format!("/v1/accounts/{address}/balance-info"),
            Self::GetTransactionMaterial => "/v1/transaction/material".to_string(),
            Self::EstimateFee => "/v1/transaction/fee-estimate".to_string(),
            Self::GetNodeVersion => "/v1/node/version".to_string(),
            Self::GetBlockHead => "/v1/blocks/head".to_string(),
            Self::GetBlocks { from, to } => format!("/v1/blocks?range={from}-{to}&noFees=true"),
            Self::GetBlockHeader { block } => format!("/v1/blocks/{block}/header"),
            Self::GetBlock { number } => format!("/v1/blocks/{number}"),
            Self::SendTransaction => "/v1/transaction".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(PolkadotTarget::GetBalance { address: "1abc".into() }.path(), "/v1/accounts/1abc/balance-info");
        assert_eq!(
            PolkadotTarget::GetBlocks {
                from: "100".into(),
                to: "110".into()
            }
            .path(),
            "/v1/blocks?range=100-110&noFees=true"
        );
        assert_eq!(PolkadotTarget::GetBlockHeader { block: "head".into() }.path(), "/v1/blocks/head/header");
        assert_eq!(PolkadotTarget::GetBlock { number: 42 }.path(), "/v1/blocks/42");
    }
}
