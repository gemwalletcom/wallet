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
