use gem_client::Target;

#[derive(Clone, Debug)]
pub enum SquidTarget {
    Route,
    Status { transaction_id: String, from_chain_id: String },
}

impl Target for SquidTarget {
    fn path(&self) -> String {
        match self {
            Self::Route => "/v2/route".to_string(),
            Self::Status { transaction_id, from_chain_id } => format!("/v2/status?transactionId={transaction_id}&fromChainId={from_chain_id}"),
        }
    }
}
