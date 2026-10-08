use gem_client::Target;

#[derive(Clone, Debug)]
pub enum FastNearTarget {
    Block,
    Transfers,
    Transactions,
}

impl Target for FastNearTarget {
    fn path(&self) -> String {
        match self {
            Self::Block => "/v0/block".to_string(),
            Self::Transfers => "/v0/transfers".to_string(),
            Self::Transactions => "/v0/transactions".to_string(),
        }
    }
}
