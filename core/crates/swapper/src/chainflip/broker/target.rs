use gem_client::Target;

#[derive(Clone, Debug)]
pub enum BrokerTarget {
    Assets,
    Quotes,
    Rpc,
    DepositChannel,
    StatusByTransactionHash,
}

impl Target for BrokerTarget {
    fn path(&self) -> String {
        match self {
            Self::Assets => "/assets".to_string(),
            Self::Quotes => "/quotes-native".to_string(),
            Self::Rpc => "/rpc".to_string(),
            Self::DepositChannel => "/deposit_channel".to_string(),
            Self::StatusByTransactionHash => "/status-by-transaction-hash".to_string(),
        }
    }
}
