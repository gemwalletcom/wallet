use gem_client::Target;

#[derive(Clone, Debug)]
pub enum DidTarget {
    Records,
}

impl Target for DidTarget {
    fn path(&self) -> String {
        match self {
            Self::Records => "/v2/account/records".to_string(),
        }
    }
}
