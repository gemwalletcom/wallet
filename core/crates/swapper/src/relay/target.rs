use gem_client::Target;

#[derive(Clone, Debug)]
pub enum RelayTarget {
    Quote,
    Request { term: String },
    Chains,
}

impl Target for RelayTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote => "/quote/v2".to_string(),
            Self::Request { term } => format!("/requests/v3?term={term}"),
            Self::Chains => "/chains".to_string(),
        }
    }
}
