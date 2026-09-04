use gem_client::Target;

#[derive(Clone, Debug)]
pub enum NearDataTarget {
    Block { number: u64 },
}

impl Target for NearDataTarget {
    fn path(&self) -> String {
        match self {
            Self::Block { number } => format!("/v0/block/{number}"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FastNearTarget {
    Transfers,
    Transactions,
}

impl Target for FastNearTarget {
    fn path(&self) -> String {
        match self {
            Self::Transfers => "/v0/transfers".to_string(),
            Self::Transactions => "/v0/transactions".to_string(),
        }
    }
}
