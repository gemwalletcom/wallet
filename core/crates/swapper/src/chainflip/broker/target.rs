use gem_client::Target;

#[derive(Clone, Debug)]
pub enum BrokerTarget {
    Assets,
    Quotes,
    Rpc,
}

impl Target for BrokerTarget {
    fn path(&self) -> String {
        match self {
            Self::Assets => "/assets".to_string(),
            Self::Quotes => "/quotes-native".to_string(),
            Self::Rpc => "/rpc".to_string(),
        }
    }
}
