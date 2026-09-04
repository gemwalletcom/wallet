use gem_client::Target;

#[derive(Clone, Debug)]
pub enum RelayTarget {
    Quote,
    Request { term: String },
    Requests { user: String, origin_chain_id: u64 },
    Chains,
}

impl Target for RelayTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote => "/quote/v2".to_string(),
            Self::Request { term } => format!("/requests/v3?term={term}"),
            Self::Requests { user, origin_chain_id } => format!("/requests/v3?user={user}&originChainId={origin_chain_id}"),
            Self::Chains => "/chains".to_string(),
        }
    }
}
