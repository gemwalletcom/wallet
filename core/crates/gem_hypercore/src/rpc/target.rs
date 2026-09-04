use gem_client::Target;

#[derive(Clone, Debug)]
pub enum HyperCoreTarget {
    Info,
    Exchange,
}

impl Target for HyperCoreTarget {
    fn path(&self) -> String {
        match self {
            Self::Info => "/info".to_string(),
            Self::Exchange => "/exchange".to_string(),
        }
    }
}
