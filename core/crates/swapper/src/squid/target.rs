use gem_client::Target;

#[derive(Clone, Debug)]
pub enum SquidTarget {
    Status { transaction_id: String, from_chain_id: String },
}

impl Target for SquidTarget {
    fn path(&self) -> String {
        match self {
            Self::Status { transaction_id, from_chain_id } => format!("/v2/status?transactionId={transaction_id}&fromChainId={from_chain_id}"),
        }
    }
}
