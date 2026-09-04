use gem_client::Target;

#[derive(Clone, Debug)]
pub enum TronTarget {
    GetBlock,
    GetBlockByNumber { number: u64 },
    GetTransactionInfoByBlockNumber { number: u64 },
    GetTransactionById { id: String },
    GetTransactionInfoById { id: String },
    TriggerConstantContract,
    ListWitnesses,
    GetChainParameters,
    GetAccount,
    GetAccountResource,
    GetReward,
    BroadcastTransaction,
    GetNowBlock,
}

impl Target for TronTarget {
    fn path(&self) -> String {
        match self {
            Self::GetBlock => "/wallet/getblock".to_string(),
            Self::GetBlockByNumber { number } => format!("/wallet/getblockbynum?num={number}"),
            Self::GetTransactionInfoByBlockNumber { number } => format!("/wallet/gettransactioninfobyblocknum?num={number}"),
            Self::GetTransactionById { id } => format!("/wallet/gettransactionbyid?value={id}"),
            Self::GetTransactionInfoById { id } => format!("/wallet/gettransactioninfobyid?value={id}"),
            Self::TriggerConstantContract => "/wallet/triggerconstantcontract".to_string(),
            Self::ListWitnesses => "/wallet/listwitnesses".to_string(),
            Self::GetChainParameters => "/wallet/getchainparameters".to_string(),
            Self::GetAccount => "/wallet/getaccount".to_string(),
            Self::GetAccountResource => "/wallet/getaccountresource".to_string(),
            Self::GetReward => "/wallet/getReward".to_string(),
            Self::BroadcastTransaction => "/wallet/broadcasttransaction".to_string(),
            Self::GetNowBlock => "/wallet/getnowblock".to_string(),
        }
    }
}
